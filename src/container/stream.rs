use crate::container::Container;
use crate::utils::io::{IOResult, IOVec, Resize};
use crate::utils::set::MinSet;
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Seek, Write};

pub struct Stream<T, S>
where
    T: DeserializeOwned + Serialize,
    S: Read + Write + Seek + Resize + Clone,
{
    make_stream: fn() -> IOResult<S>,
    vec: IOVec<T, S>,
    capacity: usize,
    chunk_size: usize,
}

impl<T, S> Stream<T, S>
where
    T: DeserializeOwned + Serialize,
    S: Read + Write + Seek + Resize + Clone,
{
    pub fn new(
        capacity: usize,
        chunk_size: usize,
        make_stream: fn() -> IOResult<S>,
    ) -> IOResult<Self> {
        let store = match make_stream() {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        Ok(Stream {
            make_stream: make_stream,
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

impl<'a, K, V, S> Container<'a, K, V> for Stream<(K, V), S>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: 'a + Read + Write + Seek + Resize + Clone,
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

    fn take<'b>(
        &'b mut self,
        key: &'b K,
    ) -> Box<dyn Iterator<Item = (K, V)> + 'b> {
        // Get indexes of matching keys.
        let mut indexes: Vec<usize> = self
            .vec
            .iter()
            .enumerate()
            .filter_map(
                |(i, s)| if &(*s).0 == key { Some(i) } else { None },
            )
            .collect();
        // Sort in ascending order
        indexes.sort();

        // Iterator removes keys with swap remove from the end.
        // Position of other matching elements is not impacted
        // by the swap.
        Box::new(indexes.into_iter().rev().map(move |i| {
            match self.vec.swap_remove(i) {
                Err(_) => panic!(),
                Ok(None) => panic!(),
                Ok(Some(v)) => v,
            }
        }))
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
        for i in indexes {
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
        let store = match (self.make_stream)() {
            Ok(s) => s,
            Err(_) => return Box::new(std::iter::empty()),
        };
        let vec = IOVec::new(store, self.chunk_size);

        let vec = std::mem::replace(&mut self.vec, vec);
        Box::new(vec.into_iter())
    }
}
