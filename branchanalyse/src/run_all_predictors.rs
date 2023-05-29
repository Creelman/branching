use rayon::prelude::*;
use serde::Serialize;
use branchlib::simulator::{Simulator, StandardSimulator, TrainingSplitSimulator};
use branchlib::strategies::always::AlwaysTaken;
use branchlib::strategies::gshare::GShare;
use branchlib::strategies::profiled::StaticPredictorTrainer;
use branchlib::strategies::twobit::TwoBit;

#[derive(Serialize)]
pub struct AllPredictorsRecord {
    pub table_size: usize,
    pub always: f64,
    pub twobit: f64,
    pub gshare_max_history: f64,
    pub gshare_best: f64,
    pub gshare_median: f64,
    pub gshare_worst: f64,
    pub profiled: f64
}

pub fn run_all_predictors(trace: &[u8]) -> Vec<AllPredictorsRecord> {
    (9usize..=16).into_par_iter().map(|table_exponent| {
        let table_size: usize = 1 << table_exponent;
        let num_index_bits = table_size.trailing_zeros() as u64;
        let mut gshares: Vec<f64> = (0..num_index_bits)
            .into_par_iter()
            .map(|i| StandardSimulator::new(GShare::new(table_size, i)).simulate(trace).to_accuracy())
            .collect();
        gshares.sort_by(|a, b| a.partial_cmp(b).unwrap().reverse());
        AllPredictorsRecord {
            table_size,
            always: StandardSimulator::new(AlwaysTaken::default()).simulate(trace).to_accuracy(),
            twobit: StandardSimulator::new(TwoBit::new(table_size)).simulate(trace).to_accuracy(),
            gshare_max_history: StandardSimulator::new(GShare::new(table_size, num_index_bits)).simulate(trace).to_accuracy(),
            gshare_best: *gshares.first().unwrap(),
            gshare_median: gshares[gshares.len() / 2],
            gshare_worst: *gshares.last().unwrap(),
            profiled: {
                let mut sim = TrainingSplitSimulator::new(StaticPredictorTrainer::new(table_size), 1.0);
                sim.train(trace);
                let p = sim.get_predictor();
                let mut sim = StandardSimulator::new(p);
                sim.simulate(trace).to_accuracy()
            }
        }
    }).collect()
}