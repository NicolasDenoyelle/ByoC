use super::Compressor;
use crate::internal::set::MinSet;
use crate::stream::Stream;
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Deref, DerefMut};

impl<'a, K, V, S> BuildingBlock<'a, K, V> for Compressor<'a, (K, V), S>
where
    K: 'a + Serialize + DeserializeOwned + Eq,
    V: 'a + Serialize + DeserializeOwned + Ord,
    S: Stream<'a>,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        *self.count.as_ref().deref()
    }

    fn contains(&self, key: &K) -> bool {
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return false,
            Ok(v) => v,
        };

        v.iter().any(|(k, _)| k == key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        // Read elements into memory.
        let mut v: Vec<(K, V)> = match self.read() {
            Err(_) => return None,
            Ok(v) => v,
        };

        // Look for matching key.
        let i = match v.iter().enumerate().find_map(|(i, (k, _))| {
            if k == key {
                Some(i)
            } else {
                None
            }
        }) {
            None => return None,
            Some(i) => i,
        };

        // Remove element from vector and rewrite vector to stream.
        let ret = v.swap_remove(i);
        match self.write(&v) {
            Ok(_) => Some(ret),
            Err(_) => None,
        }
    }

    #[allow(clippy::type_complexity)]
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(n);

        // Read elements into memory (twice).
        let (mut v1, v2): (Vec<(K, V)>, Vec<(K, V)>) =
            match (self.read(), self.read()) {
                (Ok(v1), Ok(v2)) => (v1, v2),
                _ => return out,
            };

        // Look for max values.
        let mut victims = MinSet::new(n);
        for x in v2.into_iter().enumerate().map(|(i, (_, v))| (v, i)) {
            victims.push(x);
        }

        let mut victims: Vec<usize> =
            victims.into_iter().map(|(_, i)| i).collect();
        victims.sort_unstable();

        // Make a vector of max values.
        for i in victims.into_iter().rev() {
            out.push(v1.swap_remove(i));
        }

        // Rewrite vector without popped elements to stream.
        match self.write(&v1) {
            Ok(_) => out,
            Err(_) => Vec::new(),
        }
    }

    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        // Read elements into memory.
        let mut v: Vec<(K, V)> = match self.read() {
            Err(_) => return values,
            Ok(v) => v,
        };

        // Insert only fitting elements.
        let n = std::cmp::min(self.capacity - v.len(), values.len());
        let out = values.split_off(n);
        if n > 0 {
            v.append(&mut values);
        }

        // Rewrite vector to stream.
        match self.write(&v) {
            Ok(_) => out,
            Err(_) => panic!("Could not write new elements to Compressor"),
        }
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        // Read elements into memory.
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return Box::new(std::iter::empty()),
            Ok(v) => v,
        };

        if self.stream.resize(0).is_err() {
            return Box::new(std::iter::empty());
        }

        *self.count.as_mut().deref_mut() = 0;
        Box::new(v.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::Compressor;
    use crate::stream::VecStream;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        for i in [0usize, 10usize, 100usize] {
            test_building_block(Compressor::new(VecStream::new(), i));
        }
    }
}
