#[macro_use]
extern crate byoc_benchmarks;
use byoc::utils::policy::FIFO;
use byoc::{Array, Policy};
use byoc_benchmarks::{MicroBenchmark, MicroBenchmarkArgs};
use std::fs::File;

fn main() {
    let mut args = MicroBenchmarkArgs::default("policy");

    let policy = &mut Policy::<_, usize, _, _>::new(
        Array::<(usize, _)>::new(args.capacity),
        FIFO::new(),
    );

    microbenchmark!(policy, args.bench, &mut args.file, args.header);
}
