#[macro_use]
extern crate byoc_benchmarks;
use byoc::Array;
use byoc_benchmarks::{MicroBenchmark, MicroBenchmarkArgs};
use std::fs::File;

fn main() {
    let mut args = MicroBenchmarkArgs::default("Array");

    let array = &mut Array::<(usize, usize)>::new(args.capacity);

    microbenchmark!(array, args.bench, &mut args.file, args.header);
}
