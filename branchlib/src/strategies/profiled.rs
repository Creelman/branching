use crate::strategies::{BranchPredictionStrategy, BranchPredictionTrainer};

#[derive(Debug)]
pub struct TrainedStaticPredictor {
    table: Vec<bool>
}

pub struct StaticPredictorTrainer {
    table: Vec<i64>
}

impl StaticPredictorTrainer {
    pub fn new(tablesize: usize) -> Self {
        Self {
            table: vec![0; tablesize]
        }
    }
}

impl BranchPredictionTrainer for StaticPredictorTrainer {
    type Output = TrainedStaticPredictor;

    fn add_example(&mut self, program_counter: u64, _target_address: u64, actual_result: bool) {
        let addr = program_counter as usize & (self.table.len() - 1);
        self.table[addr] += match actual_result {
            true => 1,
            false => -1
        }
    }

    fn to_predictor(&self) -> Self::Output {
        TrainedStaticPredictor {
            table: self.table.iter().map(|a| *a >= 0).collect()
        }
    }
}

impl BranchPredictionStrategy for TrainedStaticPredictor {
    fn predict_and_update(&mut self, program_counter: u64, _target_address: u64, _actual_result: bool) -> bool {
        let addr = program_counter as usize & (self.table.len() - 1);
        self.table[addr]
    }
}