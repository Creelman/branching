use clap::Subcommand;

#[derive(Subcommand, Clone, Debug)]
pub enum Strategy {
    #[command(about = "Static branch predictor which assumes a branch is always taken", name = "always")]
    Always,
    #[command(about = "Two-bit predictor with a given table size", name = "twobit")]
    TwoBit { tablesize: usize },
    #[command(about = "GShare predictor", name = "gshare")]
    GShare { tablesize: usize, history_bits: u64 },
    #[command(about = "GShare predictor, best accuracy from all variations of address and history bits", name = "gsharebest")]
    GShareBest { tablesize: usize },
    #[command(about = "Static branch predictor which profiles the program to find the most common path for various branches, \n\
    then builds a table for the most common paths for various branches", name = "profiled")]
    Profiled { tablesize: usize, split: f64 },
}

