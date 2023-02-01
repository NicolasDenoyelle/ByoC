pub mod action;
pub mod executor;
pub mod generator;
pub mod initializer;
pub mod utils;

/// Generic interface for executing a benchmark.
pub trait Benchmark<ValidationError> {
    fn initialize();
    fn run() -> utils::csv::Record;
    fn validate() -> Result<(), ValidationError> {
        Ok(())
    }
}
