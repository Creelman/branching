# Branching

*High performance library and command line tools for analysing the performance of [branch predictors](https://en.wikipedia.org/wiki/Branch_predictor)*

The simulator and implementations are optimised for performance; single-threaded it can process around 6.5GB/s of traces, and the analysis program automatically parallelises experiments - note that this will use all available processing power, use `RAYON_NUM_THREADS=N` to limit parallelism to `N` threads. 

## Provided Strategies
* Always taken
* Two-bit
* GShare, with variable history bits
* Profiled static

## Crate Structure
There are two library crates and two executable crates.

* Branchlib is a library implementing the core branch prediction functionality, including...
  * Traits for prediction strategies
  * Simulation harnesses
  * Implementations for common strategies
  * Input format parsing
* Branchcli contains some common command line parsing definitions for the simulation and analysis binaries
* Branchsim is a command line tool which can be used to simulate a particular strategy on a given input file
* Branchanalyse is a command line tool which can run many simulators on many files and return results as CSV data

