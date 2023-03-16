use std::fs::File;
use std::path::{PathBuf};
use clap::Parser;
use clap::Subcommand;
use memmap2::{Advice, Mmap};
use branchlib::predictor::BranchPredictor;
use branchlib::simulator::Simulator;
use branchlib::strategies::always::AlwaysTaken;
use branchlib::strategies::twobit::TwoBit;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg()]
    trace: PathBuf,

    #[command(subcommand)]
    strategy: Strategies,
}

#[derive(Subcommand, Clone, Debug)]
enum Strategies {
    #[command(about, name = "always")]
    Always,
    #[command(about = "Two-bit predictor with a given table size", name = "tb")]
    TwoBit { tablesize: usize },
    #[command(about = "GShare predictor", name = "gshare")]
    GShare,
    #[command(about = "Branch predictor based on profiling", name = "profile")]
    Profiled,
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
            simulate!(AlwaysTaken::new(), data)
        }
        Strategies::TwoBit { tablesize: 512 } => {
            simulate!(TwoBit::<512>::new(), data)
        }
        Strategies::TwoBit { tablesize: 1024 } => {
            simulate!(TwoBit::<1024>::new(), data)
        }
        Strategies::TwoBit { tablesize: 2048 } => {
            simulate!(TwoBit::<2048>::new(), data)
        }
        Strategies::TwoBit { tablesize: 4096 } => {
            simulate!(TwoBit::<4096>::new(), data)
        }
        Strategies::TwoBit { .. } => {
            return Err(format!("Twobit is only supported for table sizes of 512, 1024, 2048, or 4096"));
        }
        Strategies::GShare => { todo!() }
        Strategies::Profiled => { todo!() }
    };
    println!("Total Lines: {}, Hits: {}, Percentage: {}", results.total_hits, results.total_hits, results.total_hits as f64 / results.total_predictions as f64);
    Ok(())
}

