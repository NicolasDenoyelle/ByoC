#[macro_use]
extern crate byoc_benchmarks;
use byoc::{Array, Associative};
use byoc_benchmarks::{MicroBenchmark, MicroBenchmarkArgs};
use clap::Arg;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;

macro_rules! run {
    ($n: literal, $capacity: expr, $args: expr) => {{
        let arrays = [(); $n].map(|_| Array::new($capacity));
        let mut associative =
            Associative::new(arrays, DefaultHasher::new());
        let associative = &mut associative;

        microbenchmark!(
            associative,
            $args.bench,
            &mut $args.file,
            $args.header
        );
    }};
}

fn main() {
    let app = MicroBenchmarkArgs::base_app("Associative")
        .about("Run a microbenchmark for Associative BuildingBlock.");

    let nsets =
				Arg::with_name("num-sets")
				.short("n")	
				.takes_value(true)
				.default_value("1")
				.help("A power of two for the number of sets in the associative container. `num-sets` must be a divisor of `capacity`.");
    let app = app.arg(nsets);

    let capacity_error = clap::Error::with_description(
        "`capacity` must be a multiple of `num-sets`.",
        clap::ErrorKind::InvalidValue,
    );
    let num_set_error = clap::Error::with_description(
        "`num_set` exceeds maximum value.",
        clap::ErrorKind::InvalidValue,
    );

    let (mut args, matches) = MicroBenchmarkArgs::build(app);

    let num_sets: u32 = matches
        .value_of("num-sets")
        .unwrap()
        .parse::<u32>()
        .expect("Invalid format for arg 'num-sets'");
    let num_sets = 2usize.pow(num_sets);

    if args.capacity < num_sets || args.capacity.rem_euclid(num_sets) != 0
    {
        capacity_error.exit();
    }

    let array_capacity = args.capacity / num_sets;
    match num_sets {
        2 => run!(2, array_capacity, args),
        4 => run!(4, array_capacity, args),
        8 => run!(8, array_capacity, args),
        16 => run!(16, array_capacity, args),
        32 => run!(32, array_capacity, args),
        64 => run!(64, array_capacity, args),
        128 => run!(128, array_capacity, args),
        256 => run!(256, array_capacity, args),
        512 => run!(512, array_capacity, args),
        1024 => run!(1024, array_capacity, args),
        2048 => run!(2048, array_capacity, args),
        4096 => run!(4096, array_capacity, args),
        8192 => run!(8192, array_capacity, args),
        _ => num_set_error.exit(),
    };
}
