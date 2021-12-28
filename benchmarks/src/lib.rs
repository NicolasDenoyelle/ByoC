#[macro_use]
extern crate clap;

#[macro_use]
mod microbenchmarks;
pub use microbenchmarks::MicroBenchmark;

mod microbenchmarks_args;
pub use microbenchmarks_args::MicroBenchmarkArgs;
