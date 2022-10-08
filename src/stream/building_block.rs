use super::ByteStream;
use crate::internal::kmin::KMin;
use crate::stream::{IOVec, StreamFactory};
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, K, V, F> BuildingBlock<'a, K, V> for ByteStream<(K, V), F>
where
    K: 'a + DeserializeOwned + Serialize + Ord,
    V: 'a + DeserializeOwned + Serialize + Ord,
    F: 'a + StreamFactory,
{
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn size(&self) -> usize {
        self.stream
            .iter()
            .map(|option_io_vec| match option_io_vec {
                None => 0usize,
                Some(io_vec) => io_vec.size().unwrap_or(0usize),
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

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut ret = Vec::with_capacity(keys.len());
        keys.sort();
        for stream in self.stream.iter_mut().filter_map(|s| s.as_ref()) {
            for (k, v) in stream.iter().map(|x| x.unwrap()) {
                if let Ok(i) = keys.binary_search(&k) {
                    ret.push((k, v));
                    keys.remove(i);
                }
            }
        }
        ret
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
        let mut set = KMin::new(n);

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
    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut out = Vec::<(K, V)>::with_capacity(values.len());

        let mut total_size = 0usize;
        for s in self.stream.iter() {
            match s {
                None => continue,
                Some(io_vec) => match io_vec.size() {
                    Ok(s) => total_size += s,
                    Err(_) => return values,
                },
            }
        }

        for value in values.into_iter() {
            let size = match bincode::serialized_size(&value) {
                Err(_) => {
                    out.push(value);
                    continue;
                }
                Ok(s) => s as usize,
            };

            let (i, chunk_size) = Self::chunk_size(size);
            if chunk_size + total_size > self.capacity {
                out.push(value);
                continue;
            }

            if self.stream[i].is_none() {
                let store = self.factory.create();
                self.stream[i] = Some(IOVec::new(store, chunk_size));
            }

            let mut value = vec![value];
            match self.stream[i].as_mut().unwrap().append(&mut value) {
                Ok(_) => total_size += chunk_size,
                Err(_) => out.push(value.pop().unwrap()),
            }
        }
        out
    }

    #[allow(clippy::needless_collect)]
    // Collect is needed to take everything out of the container.
    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let stream: Vec<IOVec<(K, V), F::Stream>> = self
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
            test_building_block(
                ByteStream::new(VecStreamFactory {}, i),
                true,
            );
        }
    }
}
