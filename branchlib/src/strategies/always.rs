use crate::strategies::BranchPredictionStrategy;

#[derive(Debug, Default)]
pub struct AlwaysTaken {}

impl BranchPredictionStrategy for AlwaysTaken {
    fn predict_and_update(&mut self, _program_counter: u64, _target_address: u64, _actual_result: bool) -> bool {
        true
    }
}