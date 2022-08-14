#[macro_use]
extern crate byoc_benchmarks;
use byoc::stream::VecStream;
use byoc::Compressor;
use byoc_benchmarks::{MicroBenchmark, MicroBenchmarkArgs};
use std::fs::File;

fn main() {
    let app = MicroBenchmarkArgs::base_app("Compressor").about("Run a microbenchmark for Compressor BuildingBlock.\nCompressor is built on top of a (in memory) vector stream and does not account for the cost of writing to disk.");
    let mut args = MicroBenchmarkArgs::build(app).0;
    let compressor = &mut Compressor::new(VecStream::new(), args.capacity);

    microbenchmark!(compressor, args.bench, &mut args.file, args.header);
}
