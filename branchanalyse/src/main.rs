mod run_all_predictors;

use std::error::Error;
use std::fs::File;
use std::io::stdout;
use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use csv::Writer;
use memmap2::{Advice, Mmap};
use branchcli::Strategy;
use branchlib::simulator::{SimulationResults, StandardSimulator};
use branchlib::strategies::BranchPredictionStrategy;
use crate::run_all_predictors::run_all_predictors;


#[derive(Parser, Debug)]
#[command(version, about = "Branch prediction outputs for analysis")]
pub struct Args {
    #[command(subcommand)]
    command: CommandType,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CommandType {
    Traces {
        traces: PathBuf,
        #[command(subcommand)]
        predictor: Strategy,
    },
    AllPredictors {
        trace: PathBuf,
    },
    GShareHistoryRange {

    }
}


fn main() -> Result<(), String> {
    let args = Args::parse();
    match args.command {
        CommandType::Traces { traces, predictor } => {

        }
        CommandType::AllPredictors { trace } => {
            let file = File::open(trace)
                .map_err(|e| format!("Could not open the trace file: {e}"))?;
            let mmap = unsafe {
                let m = Mmap::map(&file)
                    .map_err(|e| format!("Could not memory map the trace file: {e}"))?;
                m.advise(Advice::Sequential).expect("Memory mapping error");
                m
            };
            let data = mmap.as_ref();
            let res = run_all_predictors(data);
            let mut writer = Writer::from_writer(stdout());
            res.into_iter().for_each(|a| {
                writer.serialize(a).expect("CSV serialisation error");
            })
        }
        CommandType::GShareHistoryRange { .. } => {}
    };
    Ok(())
}

