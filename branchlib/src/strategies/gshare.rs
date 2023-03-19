use crate::strategies::BranchPredictionStrategy;
use crate::strategies::twobit::TwoBit;

#[derive(Debug)]
pub struct GShare {
    twobit: TwoBit,
    global_history: u64,
    history_shift: u64,
    history_mask: u64,
    address_mask: u64,
}

impl GShare {
    pub fn new(size: usize, history_bits: u64, address_bits: u64) -> Self {
        Self {
            twobit: TwoBit::new(size),
            global_history: 0,
            history_shift: size.trailing_zeros() as u64 - history_bits,
            history_mask: 1u64.wrapping_shl(history_bits as u32).wrapping_sub(1),
            address_mask: 1u64.wrapping_shl(address_bits as u32).wrapping_sub(1),
        }
    }
}

impl BranchPredictionStrategy for GShare {
    fn predict_and_update(&mut self, program_counter: u64, target_address: u64, actual_result: bool) -> bool {
        let address = (program_counter & self.address_mask) ^ ((self.global_history << self.history_shift) & self.history_mask);
        let res = self.twobit.predict_and_update(address, target_address, actual_result);
        self.global_history <<= 1;
        self.global_history |= actual_result as u64;
        res
    }
}