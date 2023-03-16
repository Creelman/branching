use crate::strategies::BranchPredictionStrategy;

// This being its own struct was less useful than I'd anticipated
pub struct BranchPredictor<S: BranchPredictionStrategy> {
    strategy: S,
}

impl <S: BranchPredictionStrategy> BranchPredictor<S> {
    pub fn new(strategy: S) -> Self {
        Self {
            strategy
        }
    }

    // true if predicted taken, false otherwise
    pub fn predict_and_update(&mut self, program_counter: u64, target_address: u64, actual_result: bool) -> bool {
        self.strategy.predict_and_update(program_counter, target_address, actual_result)
    }
}