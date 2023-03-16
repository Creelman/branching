use hex_simd::Out;
use crate::predictor::BranchPredictor;
use crate::strategies::BranchPredictionStrategy;

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
const LINE_SIZE: usize = IS_TAKEN_OFFSET + BINARY_OFFSET + LINE_ENDING_LENGTH;

pub struct Simulator<S: BranchPredictionStrategy> {
    predictor: BranchPredictor<S>,
    results: SimulationResults,
}

#[derive(Default, Clone)]
pub struct SimulationResults {
    pub total_predictions: u64,
    pub total_hits: u64,
}

impl<S: BranchPredictionStrategy> Simulator<S> {
    pub fn new(predictor: BranchPredictor<S>) -> Self {
        Self {
            predictor,
            results: SimulationResults::default(),
        }
    }

    pub fn simulate(&mut self, trace: &[u8]) {
        // Check we're line-aligned
        debug_assert_eq!(trace.len() % LINE_SIZE, 0);
        let mut offset: usize = 0;
        while offset < trace.len() {
            self.results.total_predictions += 1;
            let line = &trace[offset..offset + LINE_SIZE];
            debug_assert_eq!(trace[LINE_SIZE - 1], b'\n');
            let program_counter = parse_address((&line[PROGRAM_COUNTER_OFFSET..PROGRAM_COUNTER_OFFSET + ADDRESS_LENGTH]).try_into().unwrap());
            let target_address = parse_address((&line[TARGET_ADDRESS_OFFSET..TARGET_ADDRESS_OFFSET + ADDRESS_LENGTH]).try_into().unwrap());
            let _branch_kind = line[BRANCH_KIND_OFFSET];
            let _direct = line[IS_DIRECT_OFFSET] == b'1';
            let _conditional = line[IS_CONDITIONAL_OFFSET] == b'1';
            let taken = line[IS_TAKEN_OFFSET] == b'1';

            if self.predictor.predict_and_update(program_counter, target_address, taken) == taken {
                self.results.total_hits += 1;
            }
            offset += LINE_SIZE;
        }
    }

    pub fn get_results(&self) -> &SimulationResults {
        &self.results
    }
}

pub fn parse_address(hex: &[u8; ADDRESS_LENGTH]) -> u64 {
    let mut arr: [u8; 8] = [0; 8];
    hex_simd::decode(hex, Out::from_slice(&mut arr)).unwrap();
    u64::from_be_bytes(arr)
}