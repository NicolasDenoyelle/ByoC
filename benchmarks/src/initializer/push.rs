use super::Initializer;
use crate::generator::KeyValuePairGenerator;
use crate::utils::iter::CollectIterator;
use byoc::BuildingBlock;

#[derive(Clone)]
pub struct PushInitializer<I> {
    key_value_pair_generator: I,
    batch_size: usize,
}

impl<I> PushInitializer<I> {
    pub fn new(key_value_pair_generator: I) -> Self {
        Self {
            key_value_pair_generator,
            batch_size: 1024,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl<K, V, B, G> Initializer<B> for PushInitializer<G>
where
    G: KeyValuePairGenerator<KeyType = K, ValueType = V>,
    B: BuildingBlock<K, V>,
{
    fn initialize(self, initializee: &mut B) {
        drop(initializee.flush());

        let iter = self.key_value_pair_generator.into_iter();
        let iter = CollectIterator::new(iter, self.batch_size);
        for kv_vec in iter {
            drop(initializee.push(kv_vec));
        }
    }
}
