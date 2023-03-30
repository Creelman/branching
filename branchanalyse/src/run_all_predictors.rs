use rayon::prelude::*;
use serde::Serialize;
use branchlib::predictor::BranchPredictor;
use branchlib::simulator::{Simulator, StandardSimulator};
use branchlib::strategies::always::AlwaysTaken;
use branchlib::strategies::gshare::GShare;
use branchlib::strategies::twobit::TwoBit;

#[derive(Serialize)]
pub struct AllPredictorsRecord {
    table_size: usize,
    always: f64,
    twobit: f64,
    gshare_best: f64,
    gshare_median: f64,
    gshare_worst: f64,
}

pub fn run_all_predictors(trace: &[u8]) -> Vec<AllPredictorsRecord> {
    let mut table_size: usize = 512;
    let mut res = Vec::new();
    while table_size <= 65536 {
        let num_index_bits = table_size.trailing_zeros();
        let mut gshares: Vec<f64> = (0..num_index_bits as u64)
            .into_par_iter()
            .map(|i| StandardSimulator::new(BranchPredictor::new(GShare::new(table_size, i))).simulate(trace).to_accuracy())
            .collect();
        gshares.sort_by(|a, b| a.partial_cmp(b).unwrap());
        res.push(AllPredictorsRecord {
            table_size,
            always: StandardSimulator::new(BranchPredictor::new(AlwaysTaken::default())).simulate(trace).to_accuracy(),
            twobit: StandardSimulator::new(BranchPredictor::new(TwoBit::new(table_size))).simulate(trace).to_accuracy(),
            gshare_best: *gshares.first().unwrap(),
            gshare_median: gshares[gshares.len() / 2],
            gshare_worst: *gshares.last().unwrap()
        });
        table_size *= 2;
    }
    res
}