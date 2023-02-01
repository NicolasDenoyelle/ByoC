use super::Executor;
use crate::generator::ActionGenerator;
use crate::utils::csv::Record;
use byoc::{BuildingBlock, Get, GetMut, Profiler};

pub struct GenericActionExecutor<Container, AG: ActionGenerator> {
    container: Profiler<Container>,
    actions: AG,
}

impl<Container, AG: ActionGenerator> GenericActionExecutor<Container, AG> {
    pub fn new(container: Container, action_generator: AG) -> Self {
        Self {
            container: Profiler::new(container),
            actions: action_generator,
        }
    }
}

impl<
        'a,
        'b: 'a,
        K: 'b,
        V: 'b,
        Container: BuildingBlock<'a, 'b, K, V> + Get<K, V> + GetMut<K, V>,
        AG: ActionGenerator<KeyType = K, ValueType = V> + Clone,
    > Executor for GenericActionExecutor<Container, AG>
{
    fn run(&mut self) -> Record {
        self.container.reset();

        for action in self.actions.clone().into_iter() {
            action.run(&mut self.container);
        }

        let (contains_num, contains_ns) = self.container.contains_stats();
        let (take_num, take_ns, take_size) = self.container.take_stats();
        let (pop_num, pop_ns, pop_size) = self.container.pop_stats();
        let (push_num, push_ns) = self.container.push_stats();
        let (flush_num, flush_ns, flush_size) = self.container.pop_stats();
        let (flush_iter_num, flush_iter_ns) =
            self.container.flush_iter_stats();
        let (get_num, get_ns) = self.container.get_stats();
        let (get_mut_num, get_mut_ns) = self.container.get_mut_stats();
        let (hit_num, hit_ns) = self.container.hit_stats();
        let (miss_num, miss_ns) = self.container.miss_stats();

        let mut record = Record::new();
        record.insert(
            String::from("contains_num"),
            format!("{}", contains_num),
        );
        record.insert(
            String::from("contains_num"),
            format!("{}", contains_num),
        );
        record.insert(
            String::from("contains_ns"),
            format!("{}", contains_ns),
        );
        record.insert(String::from("take_num,"), format!("{}", take_num));
        record.insert(String::from("take_ns,"), format!("{}", take_ns));
        record.insert(String::from("take_size"), format!("{}", take_size));
        record.insert(String::from("pop_num,"), format!("{}", pop_num));
        record.insert(String::from("pop_ns,"), format!("{}", pop_ns));
        record.insert(String::from("pop_size"), format!("{}", pop_size));
        record.insert(String::from("push_num,"), format!("{}", push_num));
        record.insert(String::from("push_ns"), format!("{}", push_ns));
        record
            .insert(String::from("flush_num,"), format!("{}", flush_num));
        record.insert(String::from("flush_ns,"), format!("{}", flush_ns));
        record
            .insert(String::from("flush_size"), format!("{}", flush_size));
        record.insert(
            String::from("flush_iter_num,"),
            format!("{}", flush_iter_num),
        );
        record.insert(
            String::from("flush_iter_ns"),
            format!("{}", flush_iter_ns),
        );
        record.insert(String::from("get_num,"), format!("{}", get_num));
        record.insert(String::from("get_ns"), format!("{}", get_ns));
        record.insert(
            String::from("get_mut_num,"),
            format!("{}", get_mut_num),
        );
        record
            .insert(String::from("get_mut_ns"), format!("{}", get_mut_ns));
        record.insert(String::from("hit_num,"), format!("{}", hit_num));
        record.insert(String::from("hit_ns"), format!("{}", hit_ns));
        record.insert(String::from("miss_num,"), format!("{}", miss_num));
        record.insert(String::from("miss_ns"), format!("{}", miss_ns));

        record
    }
}
