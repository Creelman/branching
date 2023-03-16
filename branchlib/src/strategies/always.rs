use crate::strategies::BranchPredictionStrategy;

pub struct AlwaysTaken {}

impl AlwaysTaken {
    pub fn new() -> Self {
        Self {}
    }
}

impl BranchPredictionStrategy for AlwaysTaken {
    fn predict_and_update(&mut self, _program_counter: u64, _target_address: u64, _actual_result: bool) -> bool {
        true
    }
}