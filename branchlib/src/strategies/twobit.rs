use crate::strategies::BranchPredictionStrategy;

// Indexed by current state, values are prediction, next state if false, next state if true
const STATE_MACHINE: [[u8; 3]; 4] = [
    [0, 0, 1],
    [0, 0, 2],
    [1, 1, 3],
    [1, 2, 3]
];

pub struct TwoBit<const T: usize> {
    states: [u8; T]
}

impl <const T: usize> TwoBit<T> {
    pub fn new() -> Self {
        Self {
            states: [0; T]
        }
    }
}

impl <const T: usize> BranchPredictionStrategy for TwoBit<T> {
    fn predict_and_update(&mut self, program_counter: u64, _target_address: u64, actual_result: bool) -> bool {
        let addressing_bitmask = T - 1;
        let program_counter = program_counter as usize;
        let current_state_info = STATE_MACHINE[self.states[program_counter & addressing_bitmask] as usize];
        self.states[program_counter & (addressing_bitmask as usize)] = current_state_info[1 + actual_result as usize];
        current_state_info[0] == 1
    }
}