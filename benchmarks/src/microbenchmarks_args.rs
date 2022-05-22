use crate::MicroBenchmark;
use clap::{App, Arg, ArgGroup, ArgMatches};
use std::fs::{File, OpenOptions};

pub struct MicroBenchmarkArgs {
    pub bench: MicroBenchmark,
    pub capacity: usize,
    pub file: Option<File>,
    pub header: bool,
}

impl MicroBenchmarkArgs {
    fn opt_arg<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        let g = ArgGroup::with_name("bench")
            .arg("push")
            .arg("push_multiple")
            .arg("pop")
            .arg("pop_multiple")
            .arg("take")
            .arg("take_multiple")
            .arg("get_mut")
            .required(true)
            .multiple(false);

        let push = Arg::with_name("push").help("Create a random set of key/value pairs of the same length as container capacity and measure insertion of each key/value pair one by one.");
        let push_multiple = Arg::with_name("push_multiple").help("Create random sets of key/value pairs of increasing lengths and measure the time of their insertion in an empty container.");
        let pop = Arg::with_name("pop").help("Fill a container with a random set of key/value pairs and measure the time to pop each element out one by one.");
        let pop_multiple = Arg::with_name("pop_multiple").help("Fill a container with a random set of key/value pairs and measure the time to pop an increasing number of elements out of the full container.");
        let take = Arg::with_name("take").help("Fill a container with a random set of key/value pairs and measure the time to take each key/value pair out in a random order.");
        let take_multiple = Arg::with_name("take_multiple").help("Fill a container with a random set key/value pairs and measure the time to take an increasing number of random key/value pairs out of the container.");
        let get_mut = Arg::with_name("get_mut").help("Fill a container with random sets of key/value pairs of increasing lengths and measure the time to read, write and commit each key/value pair.");

        app.arg(push)
            .arg(push_multiple)
            .arg(pop)
            .arg(pop_multiple)
            .arg(take)
            .arg(take_multiple)
            .arg(get_mut)
            .group(g)
    }

    fn from_arg<'a>(args: &'a ArgMatches<'a>) -> MicroBenchmark {
        match args.value_of("bench").unwrap() {
            "push" => MicroBenchmark::Push,
            "push_multiple" => MicroBenchmark::PushMultiple,
            "pop" => MicroBenchmark::Pop,
            "pop_multiple" => MicroBenchmark::PopMultiple,
            "take" => MicroBenchmark::Take,
            "take_multiple" => MicroBenchmark::TakeMultiple,
            "get_mut" => MicroBenchmark::GetMut,
            &_ => panic!("Unexpected benchmark name."),
        }
    }

    pub fn base_app<'a, 'b>(app_name: &str) -> App<'a, 'b> {
        let app = App::new(app_name)
            .version(crate_version!())
            .author(crate_authors!());
        let capacity_arg = Arg::with_name("capacity")
            .short("c")
            .help("Container capacity in number of key.value pairs.")
            .takes_value(true)
            .required(false);
        let file_arg = Arg::with_name("output-file")
            .short("o")
            .help(
                "File where to write results. If not provided, results
are written to stdout.",
            )
            .takes_value(true)
            .required(false);
        let header_arg = Arg::with_name("with-header")
            .short("t")
            .help("Weather or not to print benchmark header.")
            .takes_value(false)
            .required(false);

        MicroBenchmarkArgs::opt_arg(app)
            .arg(capacity_arg)
            .arg(file_arg)
            .arg(header_arg)
    }

    pub fn build<'a, 'b>(
        app: App<'a, 'b>,
    ) -> (Self, clap::ArgMatches<'a>) {
        let matches = app.get_matches();

        let margs = MicroBenchmarkArgs {
            bench: MicroBenchmarkArgs::from_arg(&matches),
            capacity: if let Some(c) = matches.value_of("capacity") {
                c.parse::<usize>()
                    .expect("Invalid format for arg 'capacity'")
            } else {
                1000usize
            },
            file: matches.value_of("output-file").map(|f| {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(f)
                    .unwrap()
            }),
            header: matches.is_present("with-header"),
        };

        (margs, matches)
    }

    pub fn default(app_name: &str) -> Self {
        let about = format!(
            "Run a microbenchmark for {} BuildingBlock.",
            app_name
        );
        let app =
            MicroBenchmarkArgs::base_app(app_name).about(about.as_ref());
        MicroBenchmarkArgs::build(app).0
    }
}