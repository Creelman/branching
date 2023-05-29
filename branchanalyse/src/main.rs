mod run_all_predictors;
mod gshare_history_range;
mod all_traces;

use std::fs;
use std::fs::{File, FileType};
use std::io::stdout;
use std::path::{PathBuf};
use clap::{Parser, Subcommand};
use csv::Writer;
use memmap2::{Mmap};
use rayon::prelude::*;
use branchlib::simulator::{LINE_SIZE, Simulator, StandardSimulator, TrainingSplitSimulator};
use branchlib::strategies::always::AlwaysTaken;
use branchlib::strategies::BranchPredictionStrategy;
use branchlib::strategies::gshare::GShare;
use branchlib::strategies::profiled::StaticPredictorTrainer;
use branchlib::strategies::twobit::TwoBit;
use crate::all_traces::AllTracesResult;
use crate::gshare_history_range::gshare_history_range;
use crate::run_all_predictors::{AllPredictorsRecord, run_all_predictors};


#[derive(Parser, Debug)]
#[command(version, about = "Branch prediction outputs for analysis")]
pub struct Args {
    #[command(subcommand)]
    command: CommandType,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Strategy {
    Always,
    TwoBit,
    GShare,
    Profiled,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CommandType {
    Traces {
        traces: PathBuf,
        #[command(subcommand)]
        predictor: Strategy,
    },
    AllPredictors {
        trace: PathBuf
    },
    Combined {
        traces: PathBuf
    },
    GShareHistoryRange {
        trace: PathBuf
    },
}


fn main() -> Result<(), String> {
    let args = Args::parse();
    match args.command {
        CommandType::Traces { traces, predictor } => {
            let x = run_traces(traces, &predictor)?;
            let mut writer = Writer::from_writer(stdout());
            x.into_iter().for_each(|a| {
                writer.serialize(a).expect("CSV serialisation error");
            })
        }
        CommandType::AllPredictors { trace } => {
            let mmap = mmap_file(trace)?;
            let data = mmap.as_ref();
            let res = run_all_predictors(data);
            let mut writer = Writer::from_writer(stdout());
            res.into_iter().for_each(|a| {
                writer.serialize(a).expect("CSV serialisation error");
            })
        }
        CommandType::GShareHistoryRange { trace } => {
            let mmap = mmap_file(trace)?;
            let data = mmap.as_ref();
            let res = gshare_history_range(data);
            let mut writer = Writer::from_writer(stdout());
            res.into_iter().for_each(|a| {
                writer.serialize(a).expect("CSV Serialisation error")
            })
        }
        CommandType::Combined { traces } => {
            let files = fs::read_dir(traces).map_err(|e| format!("Couldn't read directory: {e}"))?;
            let mut x: Vec<AllPredictorsRecord> = (9usize..=16).into_iter().map(|i| {
                AllPredictorsRecord {
                    table_size: 1 << i,
                    always: 0.0,
                    twobit: 0.0,
                    gshare_max_history: 0.0,
                    gshare_best: 0.0,
                    gshare_median: 0.0,
                    gshare_worst: 0.0,
                    profiled: 0.0,
                }
            }).collect();
            let mut total_lines = 0u64;

            for file in files {
                let file = file.map_err(|e| String::from("Couldn't open file"))?.path();
                let mmap = mmap_file(file.clone())?;
                let data = mmap.as_ref();
                let file_lines = (data.len() / LINE_SIZE) as u64;
                total_lines += file_lines;
                let res = run_all_predictors(data);
                x = x.iter().zip(res).map(|(a, b)| {
                    assert!(a.table_size == b.table_size);
                    AllPredictorsRecord {
                        table_size: a.table_size,
                        always: a.always + file_lines as f64 * b.always,
                        twobit: a.twobit + file_lines as f64 * b.twobit,
                        gshare_max_history: a.gshare_max_history + file_lines as f64 * b.gshare_max_history,
                        gshare_best: a.gshare_best + file_lines as f64 * b.gshare_best,
                        gshare_median: a.gshare_median + file_lines as f64 * b.gshare_median,
                        gshare_worst: a.gshare_worst + file_lines as f64 * b.gshare_worst,
                        profiled: a.profiled + file_lines as f64 * b.profiled,
                    }
                }).collect()
            }
            let mut writer = Writer::from_writer(stdout());
            let total_lines = total_lines as f64;
            x
                .into_iter()
                .map(|a| {
                    AllPredictorsRecord {
                        table_size: a.table_size,
                        always: a.always / total_lines,
                        twobit: a.twobit / total_lines,
                        gshare_max_history: a.gshare_max_history / total_lines,
                        gshare_best: a.gshare_best / total_lines,
                        gshare_median: a.gshare_median / total_lines,
                        gshare_worst: a.gshare_worst / total_lines,
                        profiled: a.profiled / total_lines,
                    }
                })
                .for_each(|a| {
                    writer.serialize(a).expect("CSV Serialisation error")
                })
        }
    };
    Ok(())
}

fn run_traces(traces: PathBuf, predictor: &Strategy) -> Result<Vec<AllTracesResult>, String> {
    let files = fs::read_dir(traces).map_err(|e| format!("Couldn't read directory: {e}"))?;
    let mut x: Vec<AllTracesResult> = Vec::new();
    for file in files {
        let file = file.map_err(|_| String::from("Couldn't open file"))?.path();
        let mmap = mmap_file(file.clone())?;
        let data = mmap.as_ref();
        let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
        x.append(&mut (9usize..=16).into_par_iter().map(|table_exponent| {
            let table_size: usize = 1 << table_exponent;
            let num_index_bits = table_size.trailing_zeros();
            let accuracy = match predictor {
                Strategy::Always => {
                    StandardSimulator::new(AlwaysTaken::default()).simulate(data).to_accuracy()
                }
                Strategy::TwoBit => {
                    StandardSimulator::new(TwoBit::new(table_size)).simulate(data).to_accuracy()
                }
                Strategy::GShare => {
                    StandardSimulator::new(GShare::new(table_size, num_index_bits as u64)).simulate(data).to_accuracy()
                }
                Strategy::Profiled => {
                    let mut sim = TrainingSplitSimulator::new(StaticPredictorTrainer::new(table_size), 1.0);
                    sim.train(data);
                    let p = sim.get_predictor();
                    let mut sim = StandardSimulator::new(p);
                    sim.simulate(data).to_accuracy()
                }
            };
            AllTracesResult {
                table_size,
                trace: file_name.clone(),
                accuracy,
            }
        }).collect::<Vec<AllTracesResult>>());
    }
    Ok(x)
}

fn mmap_file(path: PathBuf) -> Result<Mmap, String> {
    let file = File::open(path)
        .map_err(|e| format!("Could not open the trace file: {e}"))?;
    unsafe {
        let m = Mmap::map(&file)
            .map_err(|e| format!("Could not memory map the trace file: {e}"))?;
        Ok(m)
    }
}
