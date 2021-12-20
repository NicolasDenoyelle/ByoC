use byoc::{BuildingBlock, GetMut, Prefetch};
use rand::SeedableRng;
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::time::Instant;
use std::vec::IntoIter;

fn shuffled(n: usize, seed: u8) -> IntoIter<usize> {
    let mut keys: Vec<usize> = (0..n).collect();
    let mut rng = rand::rngs::StdRng::from_seed([seed; 32]);
    rand::seq::SliceRandom::shuffle(keys.as_mut_slice(), &mut rng);
    keys.into_iter()
}

#[derive(Clone, Copy)]
pub enum MicroBenchmark {
    Push,
    PushMultiple,
    Pop,
    PopMultiple,
    Take,
    TakeMultiple,
    GetMut,
}

impl MicroBenchmark {
    pub fn print_result(line: String, file: &mut Option<File>) {
        match file {
            None => println!("{}", line),
            Some(f) => writeln!(f, "{}", line).unwrap(),
        }
    }

    pub fn header(self) -> &'static str {
        match self {
            MicroBenchmark::Push => {
                "# container container.count nanoseconds"
            }
            MicroBenchmark::PushMultiple => {
                "# container push.count nanoseconds"
            }
            MicroBenchmark::Pop => {
                "# container container.count nanoseconds"
            }
            MicroBenchmark::PopMultiple => {
                "# container pop.count nanoseconds"
            }
            MicroBenchmark::Take => {
                "# container container.count nanoseconds"
            }
            MicroBenchmark::TakeMultiple => {
                "# container take.count nanoseconds"
            }
            MicroBenchmark::GetMut => {
                "# container container.count nanoseconds"
            }
        }
    }

    pub fn bench_push<'a, C>(
        name: &str,
        container: &mut C,
        file: &mut Option<File>,
    ) where
        C: BuildingBlock<'a, usize, usize>,
    {
        let n = container.capacity();
        drop(container.flush());

        for key in shuffled(n, 0) {
            let t = Instant::now();
            container.push(vec![(key, 0)]);
            let t = t.elapsed().as_nanos();
            MicroBenchmark::print_result(
                format!("{} {} {}", name, container.count(), t),
                file,
            );
        }
    }

    pub fn bench_push_multiple<'a, C>(
        name: &str,
        container: &mut C,
        file: &mut Option<File>,
    ) where
        C: BuildingBlock<'a, usize, usize>,
    {
        let n = container.capacity();

        for n in 1..n + 1 {
            drop(container.flush());
            let keys = shuffled(n, 0);
            let values = shuffled(n, 1);
            let insert: Vec<(usize, usize)> = keys.zip(values).collect();
            let t = Instant::now();
            let n = n - container.push(insert).len();
            let t = t.elapsed().as_nanos();
            MicroBenchmark::print_result(
                format!("{} {} {}", name, n, t),
                file,
            );
        }
    }

    pub fn bench_pop<'a, C>(
        name: &str,
        container: &mut C,
        file: &mut Option<File>,
    ) where
        C: BuildingBlock<'a, usize, usize>,
    {
        let n = container.capacity();
        drop(container.flush());

        let keys = shuffled(n + 1, 0);
        let values = shuffled(n + 1, 1);
        let insert: Vec<(usize, usize)> = keys.zip(values).collect();
        container.push(insert);

        loop {
            let t = Instant::now();
            let mut out = container.pop(1);
            let t = t.elapsed().as_nanos();
            match out.pop() {
                Some(_) => {
                    MicroBenchmark::print_result(
                        format!("{} {} {}", name, container.count(), t),
                        file,
                    );
                }
                None => break,
            }
        }
    }

    pub fn bench_pop_multiple<'a, C>(
        name: &str,
        container: &mut C,
        file: &mut Option<File>,
    ) where
        C: BuildingBlock<'a, usize, usize>,
    {
        let n = container.capacity();
        for n in 1..n + 1 {
            drop(container.flush());
            let keys = shuffled(n, 0);
            let values = shuffled(n, 1);
            let insert: Vec<(usize, usize)> = keys.zip(values).collect();
            container.push(insert);
            let t = Instant::now();
            let n = container.pop(n).len();
            let t = t.elapsed().as_nanos();
            MicroBenchmark::print_result(
                format!("{} {} {}", name, n, t),
                file,
            );
        }
    }

    pub fn bench_take<'a, C>(
        name: &str,
        container: &mut C,
        file: &mut Option<File>,
    ) where
        C: BuildingBlock<'a, usize, usize>,
    {
        let n = container.capacity();
        drop(container.flush());

        let keys = shuffled(n, 0);
        let values = shuffled(n, 1);
        let insert: Vec<(usize, usize)> = keys.zip(values).collect();
        container.push(insert);

        for i in shuffled(n, 2) {
            let t = Instant::now();
            container.take(&i);
            let t = t.elapsed().as_nanos();
            MicroBenchmark::print_result(
                format!("{} {} {}", name, container.count(), t),
                file,
            );
        }
    }

    pub fn bench_take_multiple<'a, C>(
        name: &str,
        container: &mut C,
        file: &mut Option<File>,
    ) where
        C: BuildingBlock<'a, usize, usize> + Prefetch<'a, usize, usize>,
    {
        let n = container.capacity();

        for i in 1..n + 1 {
            drop(container.flush());
            let keys = shuffled(n, 0);
            let values = shuffled(n, 1);
            let insert: Vec<(usize, usize)> = keys.zip(values).collect();
            container.push(insert);
            let mut keys = shuffled(i, 2).collect();
            let t = Instant::now();
            let n = container.take_multiple(&mut keys).len();
            let t = t.elapsed().as_nanos();
            MicroBenchmark::print_result(
                format!("{} {} {}", name, n, t),
                file,
            );
        }
    }

    pub fn bench_get_mut<'a, C, W>(
        name: &str,
        container: &mut C,
        file: &mut Option<File>,
    ) where
        W: Deref<Target = usize> + DerefMut,
        C: BuildingBlock<'a, usize, usize> + GetMut<usize, usize, W>,
    {
        let n = container.capacity();

        for n in 1..n + 1 {
            drop(container.flush());
            let keys = shuffled(n, 0);
            let values = shuffled(n, 1);
            let insert: Vec<(usize, usize)> = keys.zip(values).collect();
            container.push(insert);

            for k in 0..n {
                let t = Instant::now();
                let mut v = unsafe { container.get_mut(&k).unwrap() };
                *v += 1;
                drop(v);
                let t = t.elapsed().as_nanos();
                MicroBenchmark::print_result(
                    format!("{} {} {}", name, n, t),
                    file,
                );
            }
        }
    }
}

#[macro_export]
macro_rules! microbenchmark {
    ($container:ident, $bench: expr, $file: expr, $header: expr) => {
        let mut file: &mut Option<File> = $file;
        let name: &str = stringify!($container).as_ref();
        let bench: MicroBenchmark = $bench;
        let header: bool = $header;

        if header {
            MicroBenchmark::print_result(
                String::from(bench.header()),
                &mut file,
            );
        }

        match bench {
            MicroBenchmark::Push => {
                MicroBenchmark::bench_push(name, $container, file);
            }
            MicroBenchmark::PushMultiple => {
                MicroBenchmark::bench_push_multiple(
                    name, $container, $file,
                );
            }
            MicroBenchmark::Pop => {
                MicroBenchmark::bench_pop(name, $container, file);
            }
            MicroBenchmark::PopMultiple => {
                MicroBenchmark::bench_pop_multiple(name, $container, file);
            }
            MicroBenchmark::Take => {
                MicroBenchmark::bench_take(name, $container, file);
            }
            MicroBenchmark::TakeMultiple => {
                MicroBenchmark::bench_take_multiple(
                    name, $container, file,
                );
            }
            MicroBenchmark::GetMut => {
                MicroBenchmark::bench_get_mut(name, $container, file);
            }
        }
    };
}
