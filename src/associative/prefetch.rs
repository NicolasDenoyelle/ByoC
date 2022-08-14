use super::Associative;
use crate::{BuildingBlock, Prefetch};
use std::hash::{Hash, Hasher};

impl<'a, K, V, H, C> Prefetch<'a, K, V> for Associative<C, H>
where
    K: 'a + Clone + Hash,
    V: 'a + Ord,
    C: BuildingBlock<'a, K, V> + Prefetch<'a, K, V>,
    H: Hasher + Clone,
{
    fn prefetch(&mut self, keys: Vec<K>) {
        let mut set_keys: Vec<Vec<K>> =
            Vec::with_capacity(self.containers.len());
        for _ in 0..self.containers.len() {
            set_keys.push(Vec::with_capacity(keys.len()));
        }

        for k in keys.into_iter() {
            set_keys[self.set(k.clone())].push(k);
        }

        for c in self.containers.iter_mut().rev() {
            c.prefetch(set_keys.pop().unwrap());
        }
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut ret = Vec::with_capacity(keys.len());

        // Rearrange keys per set.
        let mut set_keys: Vec<Vec<K>> =
            Vec::with_capacity(self.containers.len());
        for _ in 0..self.containers.len() {
            set_keys.push(Vec::with_capacity(keys.len()));
        }
        for k in keys.drain(0..keys.len()) {
            set_keys[self.set(k.clone())].push(k);
        }

        // Take from each bucket.
        for (c, keys) in
            self.containers.iter_mut().zip(set_keys.iter_mut())
        {
            ret.append(&mut c.take_multiple(keys));
        }

        // Put the remaining keys back in the input keys.
        for mut sk in set_keys.into_iter() {
            keys.append(&mut sk);
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::Associative;
    use crate::tests::test_prefetch;
    use crate::Array;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn prefetch() {
        test_prefetch(Associative::new(
            vec![Array::new(5); 10],
            DefaultHasher::new(),
        ));
    }
}
