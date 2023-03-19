use std::fs::File;
use std::path::{PathBuf};
use clap::Parser;
use clap::Subcommand;
use memmap2::{Advice, Mmap};
use branchlib::predictor::BranchPredictor;
use branchlib::simulator::{SimulationResults, Simulator, Trainer};
use branchlib::strategies::always::AlwaysTaken;
use branchlib::strategies::gshare::GShare;
use branchlib::strategies::twobit::TwoBit;
use branchlib::strategies::profiled::StaticPredictorTrainer;
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about = "Branch prediction simulator")]
pub struct Args {
    #[arg()]
    trace: PathBuf,

    #[command(subcommand)]
    strategy: Strategies,
}

#[derive(Subcommand, Clone, Debug)]
enum Strategies {
    #[command(about = "Static branch predictor which assumes a branch is always taken", name = "always")]
    Always,
    #[command(about = "Two-bit predictor with a given table size", name = "twobit")]
    TwoBit { tablesize: usize },
    #[command(about = "GShare predictor", name = "gshare")]
    GShare { tablesize: usize, address_bits: u64, history_bits: u64 },
    #[command(about = "GShare predictor, best accuracy from all variations of address and history bits", name = "gsharebest")]
    GShareBest { tablesize: usize },
    #[command(about = "Static branch predictor which profiles the program to find the most common path for various branches, \n\
    then builds a table for the most common paths for various branches", name = "profiled")]
    Profiled { tablesize: usize, split: f64 },
}

macro_rules! simulate {
    ($strategy:expr, $data:expr) => {
        {
            let predictor = BranchPredictor::new($strategy);
            let mut simulator = Simulator::new(predictor);
            simulator.simulate($data);
            simulator.get_results().clone()
        }
    };
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
        Strategies::Always => {
            simulate!(AlwaysTaken::default(), data)
        }
        Strategies::TwoBit { tablesize } => {
            simulate!(TwoBit::new(tablesize), data)
        }
        Strategies::GShare {
            tablesize, address_bits, history_bits
        } => {
            simulate!(GShare::new(tablesize, history_bits, address_bits), data)
        }
        Strategies::GShareBest {
            tablesize
        } => {
            gshare_best(tablesize, data)
        }
        Strategies::Profiled { tablesize, .. } => {
            let mut trainer = Trainer::new(StaticPredictorTrainer::new(tablesize));
            trainer.train(data);
            simulate!(trainer.get_predictor(), data)
        }
    };
    println!("Total Lines: {}, Hits: {}, Percentage: {}", results.total_predictions, results.total_hits, (results.total_hits as f64 / results.total_predictions as f64) * 100.0);
    Ok(())
}

fn gshare_best(tablesize: usize, data: &[u8]) -> SimulationResults {
    let x = tablesize.trailing_zeros() as u64;
    let mut sims: Vec<Simulator<GShare>> = Vec::new();
    for address_bits in 32..=32 {
        for history_bits in 0..x {
            sims.push(Simulator::new(BranchPredictor::new(GShare::new(tablesize, history_bits, address_bits))));
        }
    }
    // Process all configurations in parallel. Accessing various parts of the mmap in parallel
    // doesn't seem to cause any major performance issues
    let best = sims.par_iter_mut()
        .map(|sim| {
            let x = sim.simulate(data).clone();
            (sim, x)
        })
        .max_by(|a, b| a.1.total_hits.cmp(&b.1.total_hits))
        .unwrap();
    println!("Best: {:?}", best.0);
    best.1
}
