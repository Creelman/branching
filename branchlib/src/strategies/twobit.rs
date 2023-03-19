use crate::strategies::BranchPredictionStrategy;

// Indexed by current state, values are prediction, next state if false, next state if true
const STATE_MACHINE: [[u8; 3]; 4] = [
    [0, 0, 1],
    [0, 0, 2],
    [1, 1, 3],
    [1, 2, 3]
];

#[derive(Debug)]
pub struct TwoBit {
    size: usize,
    states: Vec<u8>
}

impl TwoBit {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            states: vec![0; size]
        }
    }
}

impl BranchPredictionStrategy for TwoBit {
    fn predict_and_update(&mut self, program_counter: u64, _target_address: u64, actual_result: bool) -> bool {
        // Constant optimised away
        let addressing_bitmask = self.size - 1;
        let program_counter = program_counter as usize;
        // Get the current state
        let current_state_info = STATE_MACHINE[self.states[program_counter & addressing_bitmask] as usize];
        // Update the current state for this branch based on actual value
        self.states[program_counter & (addressing_bitmask)] = current_state_info[1 + actual_result as usize];
        // Return prediction
        current_state_info[0] == 1
    }
}