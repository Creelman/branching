use hex_simd::Out;
use crate::predictor::BranchPredictor;
use crate::strategies::{BranchPredictionStrategy, BranchPredictionTrainer};

const PROGRAM_COUNTER_OFFSET: usize = 0;
const ADDRESS_LENGTH: usize = 16;
const SEPARATOR_LENGTH: usize = 1;
const BINARY_OFFSET: usize = 1;
const TARGET_ADDRESS_OFFSET: usize = PROGRAM_COUNTER_OFFSET + ADDRESS_LENGTH + SEPARATOR_LENGTH;
const BRANCH_KIND_OFFSET: usize = TARGET_ADDRESS_OFFSET + ADDRESS_LENGTH + SEPARATOR_LENGTH;
const IS_DIRECT_OFFSET: usize = BRANCH_KIND_OFFSET + BINARY_OFFSET + SEPARATOR_LENGTH;
const IS_CONDITIONAL_OFFSET: usize = IS_DIRECT_OFFSET + BINARY_OFFSET + SEPARATOR_LENGTH;
const IS_TAKEN_OFFSET: usize = IS_CONDITIONAL_OFFSET + BINARY_OFFSET + SEPARATOR_LENGTH;
const LINE_ENDING_LENGTH: usize = 1;
pub const LINE_SIZE: usize = IS_TAKEN_OFFSET + BINARY_OFFSET + LINE_ENDING_LENGTH;

pub trait Simulator {
    fn simulate(&mut self, trace: &[u8]) -> &SimulationResults;
    fn get_results(&self) -> &SimulationResults;
}

#[derive(Debug)]
pub struct StandardSimulator<S: BranchPredictionStrategy> {
    predictor: BranchPredictor<S>,
    results: SimulationResults,
}

#[derive(Debug, Default, Clone)]
pub struct SimulationResults {
    pub total_predictions: u64,
    pub total_hits: u64,
}

impl SimulationResults {
    pub fn to_accuracy(&self) -> f64 {
        self.total_hits as f64 / self.total_predictions as f64
    }
}

impl<S: BranchPredictionStrategy> StandardSimulator<S> {
    pub fn new(predictor: BranchPredictor<S>) -> Self {
        Self {
            predictor,
            results: SimulationResults::default(),
        }
    }
}

impl<S: BranchPredictionStrategy> Simulator for StandardSimulator<S> {
    fn simulate(&mut self, trace: &[u8]) -> &SimulationResults {
        // Check we're line-aligned
        debug_assert_eq!(trace.len() % LINE_SIZE, 0);
        let mut offset: usize = 0;
        while offset < trace.len() {
            let line = &trace[offset..offset + LINE_SIZE];
            offset += LINE_SIZE;
            debug_assert_eq!(line[LINE_SIZE - 1], b'\n');
            let conditional = line[IS_CONDITIONAL_OFFSET] == b'1';
            if !conditional {
                continue;
            }
            let _branch_kind = line[BRANCH_KIND_OFFSET];
            let _direct = line[IS_DIRECT_OFFSET] == b'1';
            let program_counter = parse_address((&line[PROGRAM_COUNTER_OFFSET..PROGRAM_COUNTER_OFFSET + ADDRESS_LENGTH]).try_into().unwrap());
            let target_address = parse_address((&line[TARGET_ADDRESS_OFFSET..TARGET_ADDRESS_OFFSET + ADDRESS_LENGTH]).try_into().unwrap());
            let taken = line[IS_TAKEN_OFFSET] == b'1';
            self.results.total_predictions += 1;
            if self.predictor.predict_and_update(program_counter, target_address, taken) == taken {
                self.results.total_hits += 1;
            }
        }
        &self.results
    }

    fn get_results(&self) -> &SimulationResults {
        &self.results
    }
}

pub struct TrainingSplitSimulator<T: BranchPredictionTrainer> {
    trainer: T,
    split: f64,
    results: SimulationResults,
}

impl<T: BranchPredictionTrainer> TrainingSplitSimulator<T> {
    pub fn new(trainer: T, split: f64) -> Self {
        Self {
            trainer,
            split: split.clamp(0.0, 1.0),
            results: SimulationResults::default(),
        }
    }

    pub fn train(&mut self, trace: &[u8]) {
        // Check we're line-aligned
        debug_assert_eq!(trace.len() % LINE_SIZE, 0);
        let mut offset: usize = 0;
        while offset < trace.len() {
            let line = &trace[offset..offset + LINE_SIZE];
            debug_assert_eq!(trace[LINE_SIZE - 1], b'\n');
            offset += LINE_SIZE;
            let conditional = line[IS_CONDITIONAL_OFFSET] == b'1';
            if !conditional { continue; }
            let program_counter = parse_address((&line[PROGRAM_COUNTER_OFFSET..PROGRAM_COUNTER_OFFSET + ADDRESS_LENGTH]).try_into().unwrap());
            let target_address = parse_address((&line[TARGET_ADDRESS_OFFSET..TARGET_ADDRESS_OFFSET + ADDRESS_LENGTH]).try_into().unwrap());
            let _branch_kind = line[BRANCH_KIND_OFFSET];
            let _direct = line[IS_DIRECT_OFFSET] == b'1';
            let taken = line[IS_TAKEN_OFFSET] == b'1';
            self.trainer.add_example(program_counter, target_address, taken);
        }
    }

    pub fn get_predictor(&self) -> T::Output {
        self.trainer.to_predictor()
    }
}

impl<T> Simulator for TrainingSplitSimulator<T>
    where
        T: BranchPredictionTrainer,
{
    fn simulate(&mut self, trace: &[u8]) -> &SimulationResults {
        // Split the trace along a line size
        let mut split_point = (self.split * trace.len() as f64) as usize;
        split_point -= split_point % LINE_SIZE;
        let training_set = &trace[..split_point];
        let test_set = &trace[split_point..];
        self.train(training_set);
        self.results = StandardSimulator::new(BranchPredictor::new(self.get_predictor())).simulate(test_set).clone();
        &self.results
    }

    fn get_results(&self) -> &SimulationResults {
        &self.results
    }
}

pub fn parse_address(hex: &[u8; ADDRESS_LENGTH]) -> u64 {
    let mut arr: [u8; 8] = [0; 8];
    hex_simd::decode(hex, Out::from_slice(&mut arr)).unwrap();
    u64::from_be_bytes(arr)
}