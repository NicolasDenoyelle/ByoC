use crate::private::io_vec::{IOResult, IOVec};
use crate::private::set::MinSet;
use crate::utils::stream::Stream as Streamable;
use crate::utils::stream::StreamFactory;
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};

pub struct Stream<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Streamable,
    F: StreamFactory<S> + Clone,
{
    factory: F,
    vec: IOVec<T, S>,
    capacity: usize,
    chunk_size: usize,
}

impl<T, S, F> Stream<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Streamable,
    F: StreamFactory<S> + Clone,
{
    pub fn new(
        mut factory: F,
        capacity: usize,
        chunk_size: usize,
    ) -> IOResult<Self> {
        let store = factory.create();

        Ok(Stream {
            factory: factory,
            vec: IOVec::new(store, chunk_size),
            capacity: capacity,
            chunk_size: chunk_size,
        })
    }

    /// Returns the chunk size that fits this `value`.
    /// This is the next power of two above the byte size occupied
    /// by the serialized `value`.
    /// This function panics if [`bincode`](../../bincode/index.html) cannot
    /// compute the [serialized size](../../bincode/fn.serialized_size.html)
    /// of this item.
    pub fn chunk_size(value: &T) -> usize {
        let mut n = bincode::serialized_size(value).unwrap() as usize;
        let mut i = 0usize;

        loop {
            let s = n << 1usize;
            if (s >> 1usize) == n {
                n = s;
                i += 1;
            } else {
                break;
            }
        }
        1usize + (!0usize >> i)
    }
}

impl<'a, K, V, S, F> BuildingBlock<'a, K, V> for Stream<(K, V), S, F>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: 'a + Streamable,
    F: StreamFactory<S> + Clone,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        match self.vec.len() {
            Ok(s) => s,
            Err(_) => 0usize,
        }
    }

    fn contains(&self, key: &K) -> bool {
        self.vec.iter().any(|s| &(*s).0 == key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        // Get indexes of matching keys.
        match self.vec.iter().enumerate().find_map(|(i, s)| {
            if &(*s).0 == key {
                Some(i)
            } else {
                None
            }
        }) {
            None => None,
            Some(i) => match self.vec.swap_remove(i) {
                Err(_) => panic!(),
                Ok(None) => panic!(),
                Ok(Some(v)) => Some(v),
            },
        }
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut set = MinSet::new(n);

        // Stream values and save only the top n ones with their index.
        for e in
            self.vec.iter().enumerate().map(|(i, s)| (s.unwrap().1, i))
        {
            set.push(e);
        }

        // Filter values to only keep index and sort index.
        let mut indexes: Vec<usize> =
            set.into_iter().map(|(_, i)| i).collect();
        indexes.sort();

        let mut ret = Vec::with_capacity(indexes.len());
        // Removes keys with swap remove from the end.
        // Position of other matching elements is not impacted
        // by the swap.
        for i in indexes.into_iter().rev() {
            match self.vec.swap_remove(i) {
                Err(_) => return ret,
                Ok(None) => return ret,
                Ok(Some(v)) => ret.push(v),
            }
        }
        ret
    }

    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        let vlen = match self.vec.len() {
            Ok(len) => len,
            Err(_) => return Vec::new(),
        };

        let n = std::cmp::min(values.len(), self.capacity - vlen);

        if n > 0 {
            let mut out = values.split_off(n);
            match self.vec.append(&mut values) {
                Ok(_) => out,
                Err(_) => {
                    values.append(&mut out);
                    values
                }
            }
        } else {
            values
        }
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let store = self.factory.create();
        let vec = IOVec::new(store, self.chunk_size);

        let vec = std::mem::replace(&mut self.vec, vec);
        Box::new(vec.into_iter())
    }
}
