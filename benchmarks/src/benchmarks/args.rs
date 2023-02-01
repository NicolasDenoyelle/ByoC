use crate::benchmarks::{Action, Benchmark, Initializer, Key, Value};
use byoc::{BuildingBlock, GetMut};
use clap::{Arg, ArgMatches, Command};
use std::iter::Iterator;
use std::ops::DerefMut;

#[derive(Copy, Clone)]
pub struct BenchmarkArgs {
    pub num_iterations: usize,
    pub capacity: usize,
    push_weight: f32,
    take_weight: f32,
    get_mut_weight: f32,
    push_or_get_mut_weight: f32,
    push_or_take_weight: f32,
    get_mut_or_push_weight: f32,
    initializer: Initializer,
}

impl BenchmarkArgs {
    pub fn run<'a, P, T, G, U, C>(
        self,
        container: C,
        push_iter: P,
        take_iter: T,
        get_mut_iter: G,
    ) where
        P: Iterator<Item = Key>,
        T: Iterator<Item = Key>,
        G: Iterator<Item = Key>,
        U: 'a + DerefMut<Target = Value>,
        C: BuildingBlock<'a, Key, Value> + GetMut<Key, Value, U>,
    {
        let mut bench =
            Benchmark::new(container, push_iter, take_iter, get_mut_iter);
        bench.set_weight(Action::Push, self.push_weight);
        bench.set_weight(Action::Take, self.take_weight);
        bench.set_weight(Action::GetMut, self.get_mut_weight);
        bench
            .set_weight(Action::PushOrGetMut, self.push_or_get_mut_weight);
        bench.set_weight(Action::PushOrTake, self.push_or_take_weight);
        bench
            .set_weight(Action::GetMutOrPush, self.get_mut_or_push_weight);
        bench.initialize(self.initializer);
        bench.run(self.num_iterations);
    }

    pub fn from_app(app: Command) -> (Self, ArgMatches) {
        let matches = app.get_matches();
        (
            BenchmarkArgs {
                num_iterations: matches
                    .value_of("num-iterations")
                    .unwrap()
                    .parse()
                    .expect("Invalid format for arg 'num-iterations'"),
                capacity: matches
                    .value_of("capacity")
                    .unwrap()
                    .parse()
                    .expect("Invalid format for arg 'capacity'"),
                push_weight: matches
                    .value_of("push-weight")
                    .unwrap()
                    .parse()
                    .expect("Invalid format for arg 'push-weight'"),
                take_weight: matches
                    .value_of("take-weight")
                    .unwrap()
                    .parse()
                    .expect("Invalid format for arg 'take-weight'"),
                get_mut_weight: matches
                    .value_of("get-mut-weight")
                    .unwrap()
                    .parse()
                    .expect("Invalid format for arg 'get-mut-weight'"),
                push_or_get_mut_weight: matches
                    .value_of("push-or-get-mut-weight")
                    .unwrap()
                    .parse()
                    .expect(
                        "Invalid format for arg 'push-or-get-mut-weight'",
                    ),
                push_or_take_weight: matches
                    .value_of("push-or-take-weight")
                    .unwrap()
                    .parse()
                    .expect(
                        "Invalid format for arg 'push-or-take-weight'",
                    ),
                get_mut_or_push_weight: matches
                    .value_of("get-mut-or-push-weight")
                    .unwrap()
                    .parse()
                    .expect(
                        "Invalid format for arg 'get-mut-or-push-weight'",
                    ),
                initializer: matches
                    .value_of("initializer")
                    .unwrap()
                    .into(),
            },
            matches,
        )
    }

    pub fn new(app_name: &str) -> Self {
        Self::from_app(Self::app(app_name)).0
    }

    pub fn app<'a>(app_name: &str) -> Command<'a> {
        let push =
						Arg::new("push-weight")
						.long("push-weight")
						.default_value("1.0")
						.help("Set the probability weight of this action being performed on next action iteration. This action will try to push a key and associated value in the container.");
        let take =
						Arg::new("take-weight")
						.long("take-weight")
						.default_value("1.0")
						.help("Set the probability weight of this action being performed on next action iteration. This action will try to take a key and associated value out of the container.");
        let get_mut =
						Arg::new("get-mut-weight")
						.long("get-mut-weight")
						.default_value("1.0")
						.help("Set the probability weight of this action being performed on next action iteration. This action will try to read a value from the container.");
        let push_or_get_mut =
						Arg::new("push-or-get-mut-weight")
						.long("push-or-get-mut-weight")
						.default_value("1.0")
						.help("Set the probability weight of this action being performed on next action iteration. This action will check if a key is in the container. If the key is not found, it is pushed in the container, else it is read from the container.");
        let push_or_take =
						Arg::new("push-or-take-weight")
						.long("push-or-take-weight")
						.default_value("1.0")
						.help("Set the probability weight of this action being performed on next action iteration. This action will check if a key is in the container. If the key is not found, it is pushed in the container, else it is taken out of the container.");
        let get_mut_or_push =
						Arg::new("get-mut-or-push-weight")
						.long("get-mut-or-push-weight")
						.default_value("1.0")
						.help("Set the probability weight of this action being performed on next action iteration. This action will try to read an element from the container and push matching key if it was not found.");
        let num_iterations = Arg::new("num-iterations")
            .long("num-iterations")
            .short('n')
            .default_value("1073741824")
            .help("The amount of actions to run on the container.");
        let capacity = Arg::new("capacity")
            .long("capacity")
            .short('c')
            .default_value("268435456")
            .help("The amount of actions to run on the container.");
        let initializer = Arg::new("initializer")
            .long("initializer")
            .default_value(Initializer::RANDOM_UNIFORM_PREFIX)
            .help(Initializer::HELP_STR);

        Command::new(app_name)
            .version(crate_version!())
            .author(crate_authors!())
            .arg(push)
            .arg(take)
            .arg(get_mut)
            .arg(push_or_get_mut)
            .arg(push_or_take)
            .arg(get_mut_or_push)
            .arg(num_iterations)
            .arg(capacity)
            .arg(initializer)
    }
}
