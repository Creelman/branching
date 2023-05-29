use std::fs::File;
use std::path::{PathBuf};
use clap::Parser;
use memmap2::{Advice, Mmap};
use branchlib::simulator::{SimulationResults, StandardSimulator, TrainingSplitSimulator};
use branchlib::strategies::always::AlwaysTaken;
use branchlib::strategies::gshare::GShare;
use branchlib::strategies::twobit::TwoBit;
use branchlib::strategies::profiled::StaticPredictorTrainer;
use branchlib::simulator::Simulator;
use rayon::prelude::*;
use branchcli::Strategy;
use branchlib::strategies::BranchPredictionStrategy;

#[derive(Parser, Debug)]
#[command(version, about = "Branch prediction simulator")]
pub struct Args {
    #[arg()]
    trace: PathBuf,

    #[command(subcommand)]
    strategy: Strategy,
}

fn simulate<T: BranchPredictionStrategy>(strategy: T, data: &[u8]) -> SimulationResults {
    StandardSimulator::new(strategy).simulate(data).clone()
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let file = File::open(args.trace)
        .map_err(|e| format!("Could not open the trace file: {e}"))?;
    let mmap = unsafe {
        let m = Mmap::map(&file)
            .map_err(|e| format!("Could not memory map the trace file: {e}"))?;
        m.advise(Advice::Sequential).expect("Memory mapping error");
        m
    };
    let data = mmap.as_ref();
    let results = match args.strategy {
        Strategy::Always => {
            simulate(AlwaysTaken::default(), data)
        }
        Strategy::TwoBit { tablesize } => {
            simulate(TwoBit::new(tablesize), data)
        }
        Strategy::GShare {
            tablesize, history_bits
        } => {
            simulate(GShare::new(tablesize, history_bits), data)
        }
        Strategy::GShareBest {
            tablesize
        } => {
            gshare_best(tablesize, data)
        }
        Strategy::Profiled { tablesize, split } => {
            let mut trainer = TrainingSplitSimulator::new(StaticPredictorTrainer::new(tablesize), split);
            trainer.simulate(data).clone()
        }
    };
    println!("Total Lines: {}, Hits: {}, Percentage: {}", results.total_predictions, results.total_hits, (results.total_hits as f64 / results.total_predictions as f64) * 100.0);
    Ok(())
}

fn gshare_best(tablesize: usize, data: &[u8]) -> SimulationResults {
    let x = tablesize.trailing_zeros() as u64;
    let mut sims: Vec<StandardSimulator<GShare>> = Vec::new();
    for history_bits in 0..x {
        sims.push(StandardSimulator::new(GShare::new(tablesize, history_bits)));
    }
    // Process all configurations in parallel. Accessing various parts of the mmap in parallel
    // doesn't seem to cause any major performance issues, despite advising sequential accesses
    let (_, res) = sims.par_iter_mut()
        .map(|sim| {
            let x = sim.simulate(data).clone();
            (sim, x)
        })
        .max_by(|a, b| a.1.total_hits.cmp(&b.1.total_hits))
        .unwrap();
    res
}
