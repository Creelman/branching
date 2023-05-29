use rayon::prelude::*;
use serde::Serialize;
use branchlib::simulator::{Simulator, StandardSimulator};
use branchlib::strategies::gshare::GShare;

#[derive(Serialize)]
pub struct GShareHistoryRangeResult {
    table_size: usize,
    history_length: u64,
    accuracy: f64
}

pub fn gshare_history_range(trace: &[u8]) -> Vec<GShareHistoryRangeResult> {
    (9usize..=16).into_par_iter().flat_map(|table_exponent| {
        let table_size: usize = 1 << table_exponent;
        let num_index_bits = table_size.trailing_zeros();
        (0..=num_index_bits as u64)
            .into_par_iter()
            .map(|history_length| {
                let accuracy = StandardSimulator::new(GShare::new(table_size, history_length))
                    .simulate(trace)
                    .to_accuracy();
                GShareHistoryRangeResult {
                    table_size,
                    history_length,
                    accuracy,
                }
            })
            .collect::<Vec<GShareHistoryRangeResult>>()
    }).collect()
}