pub type Key = u64;
pub type Value = u64;

mod benchmark;
pub use benchmark::Benchmark;

mod args;
pub use args::BenchmarkArgs;

mod initializer;
pub use initializer::Initializer;

mod action;
pub use action::Action;

pub mod pattern;
mod set;
