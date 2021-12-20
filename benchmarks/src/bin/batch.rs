#[macro_use]
extern crate byoc_benchmarks;
use byoc::{Array, Batch};
use byoc_benchmarks::{MicroBenchmark, MicroBenchmarkArgs};
use clap::Arg;
use std::fs::File;

fn main() {
    let app = MicroBenchmarkArgs::base_app("Batch")
        .about("Run a microbenchmark for Batch BuildingBlock.");
    let num =
	Arg::with_name("num-batches")
	.short("n")	
	.takes_value(true)
	.default_value("1")
	.help("The number of batches to build. `num-batches` must be a divisor of `capacity`.");
    let num_error = clap::Error::with_description(
        "`num-batches` must be a divisor of `capacity`.",
        clap::ErrorKind::InvalidValue,
    );
    let app = app.arg(num);

    let (mut args, matches) = MicroBenchmarkArgs::build(app);
    let num_batches = matches
        .value_of("num-batches")
        .unwrap()
        .parse::<usize>()
        .expect("Invalid format for arg 'num-batches'");
    if args.capacity.rem_euclid(num_batches) != 0 {
        num_error.exit();
    }

    let batch_capacity = args.capacity / num_batches;
    let mut batch = Batch::new();
    for _ in 0..num_batches {
        batch.append(Array::new(batch_capacity));
    }
    let batch = &mut batch;

    microbenchmark!(batch, args.bench, &mut args.file, args.header);
}
