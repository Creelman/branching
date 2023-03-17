pub mod always;
pub mod twobit;
pub mod gshare;
pub mod profiled;

pub trait BranchPredictionStrategy {
    fn predict_and_update(&mut self, program_counter: u64, target_address: u64, actual_result: bool) -> bool;
}

pub trait BranchPredictionTrainer {
    type Output: BranchPredictionStrategy;

    fn add_example(&mut self, program_counter: u64, target_address: u64, actual_result: bool);
    fn to_predictor(self) -> Self::Output;
}