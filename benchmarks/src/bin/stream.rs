#[macro_use]
extern crate byoc_benchmarks;
use byoc::stream::TempFileStreamFactory;
use byoc::Stream;
use byoc_benchmarks::{MicroBenchmark, MicroBenchmarkArgs};
use std::fs::File;

fn main() {
    let mut args = MicroBenchmarkArgs::default("Stream");
    let stream = &mut Stream::new(TempFileStreamFactory {}, args.capacity);
    microbenchmark!(stream, args.bench, &mut args.file, args.header);
}
