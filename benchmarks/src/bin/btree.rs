#[macro_use]
extern crate byoc_benchmarks;
use byoc::BTree;
use byoc_benchmarks::{MicroBenchmark, MicroBenchmarkArgs};
use std::fs::File;

fn main() {
    let mut args = MicroBenchmarkArgs::default("BTree");

    let btree = &mut BTree::<usize, usize>::new(args.capacity);

    microbenchmark!(btree, args.bench, &mut args.file, args.header);
}
