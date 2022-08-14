use super::ByteStream;
use crate::internal::io_vec::IOVec;
use crate::internal::set::MinSet;
use crate::stream::{Stream, StreamFactory};
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, K, V, S, F> BuildingBlock<'a, K, V>
    for ByteStream<'a, (K, V), S, F>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: 'a + Stream<'a>,
    F: StreamFactory<S>,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn count(&self) -> usize {
        self.stream
            .iter()
            .map(|s| match s {
                None => 0usize,
                Some(s) => s.len().unwrap_or(0usize),
            })
            .sum()
    }

    fn contains(&self, key: &K) -> bool {
        self.stream.iter().any(|s| {
            if let Some(s) = s {
                s.iter().any(|kv| &(*kv).0 == key)
            } else {
                false
            }
        })
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        // Get indexe of matching key container and matching key position.
        let ij =
            self.stream.iter().enumerate().find_map(|(i, s)| match s {
                None => None,
                Some(s) => s.iter().enumerate().find_map(|(j, kv)| {
                    if &(*kv).0 == key {
                        Some((i, j))
                    } else {
                        None
                    }
                }),
            });

        match ij {
            None => None,
            Some((i, j)) => {
                match self.stream[i].as_mut().unwrap().swap_remove(j) {
                    Err(_) => panic!(),
                    Ok(None) => panic!(),
                    Ok(Some(v)) => Some(v),
                }
            }
        }
    }

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// This building block implements the trait
    /// [`Ordered`](trait.Ordered.html), which means that
    /// the highest values are popped out. This is implemented by
    /// reading all values in the stream and keeping the `n` highest values
    /// with there position index in a btree structure.
    /// Once the stream has been entirely walked, indexes of victims
    /// are sorted in descending order and the corrseponding victims
    /// are removed one by one, by swapping them with element at the
    /// end of the stream then popping out the last element.
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut set = MinSet::new(n);

        // Stream values and save only the top n ones with their index.
        for (i, s) in self.stream.iter().enumerate() {
            if let Some(s) = s.as_ref() {
                for (j, kv) in s.iter().enumerate() {
                    set.push((kv.unwrap().1, i, j));
                }
            }
        }

        // Filter values to only keep index and sort index.
        let mut indexes: Vec<(usize, usize)> =
            set.into_iter().map(|(_, i, j)| (i, j)).collect();
        indexes.sort_unstable();

        let mut ret = Vec::with_capacity(indexes.len());
        // Removes keys with swap remove from the end.
        // Position of other matching elements is not impacted
        // by the swap.
        for (i, j) in indexes.into_iter().rev() {
            match self.stream[i].as_mut().unwrap().swap_remove(j) {
                Err(_) => return ret,
                Ok(None) => return ret,
                Ok(Some(kv)) => ret.push(kv),
            }
        }
        ret
    }

    /// Insert key/value pairs in the container. If the container cannot
    /// store all the values, the last input values not fitting in are
    /// returned.
    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        let n = std::cmp::min(values.len(), self.capacity - self.count());

        if n > 0 {
            let mut out = values.split_off(n);
            for value in values.into_iter() {
                let size = match bincode::serialized_size(&value) {
                    Err(_) => {
                        out.push(value);
                        continue;
                    }
                    Ok(s) => s as usize,
                };

                let (i, chunk_size) = Self::chunk_size(size);
                if self.stream[i].is_none() {
                    let store = self.factory.create();
                    self.stream[i] = Some(IOVec::new(store, chunk_size));
                }
                let mut value = vec![value];
                match self.stream[i].as_mut().unwrap().append(&mut value) {
                    Ok(_) => {}
                    Err(_) => out.push(value.pop().unwrap()),
                }
            }
            out
        } else {
            values
        }
    }

    #[allow(clippy::needless_collect)]
    // Collect is needed to take everything out of the container.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let stream: Vec<IOVec<(K, V), S>> = self
            .stream
            .iter_mut()
            .filter_map(|opt| opt.take())
            .collect();
        Box::new(stream.into_iter().flat_map(|v| v.into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::ByteStream;
    use crate::stream::VecStreamFactory;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        for i in [0usize, 10usize, 100usize] {
            test_building_block(ByteStream::new(VecStreamFactory {}, i));
        }
    }
}
