use crate::policy::Ordered;
use crate::private::io_vec::{IOStruct, IOStructMut, IOVec};
use crate::private::set::MinSet;
use crate::utils::stream::Stream as Streamable;
use crate::utils::stream::StreamFactory;
use crate::{BuildingBlock, Get};
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// Stream for any chunk size
//------------------------------------------------------------------------//

pub struct Stream<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Streamable,
    F: StreamFactory<S> + Clone,
{
    factory: F,
    streams: Vec<Option<IOVec<T, S>>>,
    capacity: usize,
}

impl<T, S, F> Stream<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Streamable,
    F: StreamFactory<S> + Clone,
{
    pub fn new(factory: F, capacity: usize) -> Self {
        let max_streams = 8 * std::mem::size_of::<usize>();
        let mut streams =
            Vec::<Option<IOVec<T, S>>>::with_capacity(max_streams);
        for _ in 0..max_streams {
            streams.push(None)
        }

        Stream {
            factory: factory,
            streams: streams,
            capacity: capacity,
        }
    }

    /// Returns the position of the most significant byte
    /// starting from the left and associated power of two.
    /// The power of two is the size of the chunk that will hold the
    /// serialized value of the `size` provided as input.
    fn chunk_size(mut size: usize) -> (usize, usize) {
        let mut i = 0usize;

        loop {
            let s = size << 1usize;
            if (s >> 1usize) == size {
                size = s;
                i += 1;
            } else {
                break;
            }
        }
        (i, 1usize + (!0usize >> i))
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
        self.streams
            .iter()
            .map(|s| match s {
                None => 0usize,
                Some(s) => match s.len() {
                    Ok(s) => s,
                    Err(_) => 0usize,
                },
            })
            .sum()
    }

    fn contains(&self, key: &K) -> bool {
        self.streams.iter().any(|s| {
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
            self.streams.iter().enumerate().find_map(|(i, s)| match s {
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
                match self.streams[i].as_mut().unwrap().swap_remove(j) {
                    Err(_) => panic!(),
                    Ok(None) => panic!(),
                    Ok(Some(v)) => Some(v),
                }
            }
        }
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut set = MinSet::new(n);

        // Stream values and save only the top n ones with their index.
        for (i, s) in self.streams.iter().enumerate() {
            if let Some(s) = s.as_ref() {
                for (j, kv) in s.iter().enumerate() {
                    set.push((kv.unwrap().1, i, j));
                }
            }
        }

        // Filter values to only keep index and sort index.
        let mut indexes: Vec<(usize, usize)> =
            set.into_iter().map(|(_, i, j)| (i, j)).collect();
        indexes.sort();

        let mut ret = Vec::with_capacity(indexes.len());
        // Removes keys with swap remove from the end.
        // Position of other matching elements is not impacted
        // by the swap.
        for (i, j) in indexes.into_iter().rev() {
            match self.streams[i].as_mut().unwrap().swap_remove(j) {
                Err(_) => return ret,
                Ok(None) => return ret,
                Ok(Some(kv)) => ret.push(kv),
            }
        }
        ret
    }

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
                if self.streams[i].is_none() {
                    let store = self.factory.create();
                    self.streams[i] = Some(IOVec::new(store, chunk_size));
                }
                let mut value = vec![value];
                match self.streams[i].as_mut().unwrap().append(&mut value)
                {
                    Ok(_) => {}
                    Err(_) => out.push(value.pop().unwrap()),
                }
            }
            out
        } else {
            values
        }
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let streams: Vec<IOVec<(K, V), S>> = self
            .streams
            .iter_mut()
            .filter_map(|opt| opt.take())
            .collect();
        Box::new(streams.into_iter().flat_map(|v| v.into_iter()))
    }
}

impl<K, V: Ord, S, F> Ordered<V> for Stream<(K, V), S, F>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
    S: Streamable,
    F: StreamFactory<S> + Clone,
{
}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

pub struct StreamCell<K, V> {
    item: IOStruct<(K, V)>,
}

impl<K, V> Deref for StreamCell<K, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.item.deref().1
    }
}

pub struct StreamCellMut<K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Streamable,
{
    item: IOStructMut<(K, V), S>,
}

impl<K, V, S> Deref for StreamCellMut<K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Streamable,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.item.deref().1
    }
}

impl<K, V, S> DerefMut for StreamCellMut<K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Streamable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item.deref_mut().1
    }
}

impl<K, V, F, S> Get<K, V, StreamCell<K, V>, StreamCellMut<K, V, S>>
    for Stream<(K, V), S, F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    S: Streamable,
    F: StreamFactory<S> + Clone,
{
    unsafe fn get(&self, key: &K) -> Option<StreamCell<K, V>> {
        self.streams
            .iter()
            .filter_map(|s| s.as_ref())
            .find_map(|s| {
                s.iter().find_map(|item| {
                    let (k, _) = &*item;
                    if k == key {
                        Some(StreamCell { item: item })
                    } else {
                        None
                    }
                })
            })
    }

    unsafe fn get_mut(
        &mut self,
        key: &K,
    ) -> Option<StreamCellMut<K, V, S>> {
        self.streams
            .iter_mut()
            .filter_map(|s| s.as_mut())
            .find_map(|s| {
                s.iter_mut().find_map(|item| {
                    let (k, _) = &*item;
                    if k == key {
                        Some(StreamCellMut { item: item })
                    } else {
                        None
                    }
                })
            })
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::Stream;
    use crate::policy::tests::test_ordered;
    use crate::tests::{test_building_block, test_get};
    use crate::utils::stream::VecStreamFactory;

    #[test]
    fn building_block() {
        for i in vec![0, 10, 100] {
            test_building_block(Stream::new(VecStreamFactory {}, i));
        }
    }

    #[test]
    fn ordered() {
        for i in vec![0, 10, 100] {
            test_ordered(Stream::new(VecStreamFactory {}, i));
        }
    }

    #[test]
    fn get() {
        for i in vec![0, 10, 100] {
            test_get(Stream::new(VecStreamFactory {}, i));
        }
    }
}
