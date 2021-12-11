use crate::container::stream::io_vec::{IOStruct, IOStructMut, IOVec};
use crate::container::stream::{Stream, StreamFactory};
use crate::private::set::MinSet;
use crate::{BuildingBlock, Get, GetMut, Ordered, Prefetch};
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Deref, DerefMut};

//------------------------------------------------------------------------//
// ByteStream for any chunk size
//------------------------------------------------------------------------//

/// Key/value storage on a byte stream.
///
/// `ByteStream` is a [`BuildingBlock`](../trait.BuildingBlock.html)
/// implementation storing key/value pairs together in a byte stream.
/// The byte stream of a `ByteStream` can be any kind of byte stream
/// implementing the trait
/// [`Stream`](trait.Stream.html) such as a
/// [file](struct.FileStream.html) or a
/// [vector](struct.VecStream.html).
///
/// The `ByteStream` building block behaves similarly as the
/// [`Array`](struct.Array.html) building block implementation.
/// In fact key/value pairs are stored in a set of vectors abstraction
/// implemented over a byte stream.
///
/// Key/value pairs of this building block are serialized/deserialized
/// into bytes inside a buffer. For a given kay/value pair, the size of
/// the corresponding buffer is the closest power of two fitting the
/// serialized pair. Given a chunk size, the chunk is stored in a vector
/// (or byte stream) of chunks of the same size.
///
/// Byte streams are generated from a structure implementing the trait
/// [`StreamFactory`](trait.StreamFactory.html), such as
/// [`VecStreamFactory`](struct.VecStreamFactory.html).
///
/// ## Examples
///
/// ```
/// use cache::BuildingBlock;
/// use cache::container::ByteStream;
/// use cache::container::stream::vec_stream::VecStreamFactory;
///
/// // Array with 3 elements capacity.
/// let mut c = ByteStream::new(VecStreamFactory{}, 3);
///
/// // BuildingBlock as room for 3 elements and returns an empty vector.
/// // No element is rejected.
/// assert!(c.push(vec![(1, 4), (2, 2), (3, 3)]).pop().is_none());
///
/// // Stream is full and pops extra inserted value (all values here).
/// let (key, _) = c.push(vec![(4, 12)]).pop().unwrap();
/// assert_eq!(key, 4);
///
/// // Stream pops elements in order of the highest values.
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, 1);
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, 3);
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, 2);
/// ```
pub struct ByteStream<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S>,
{
    factory: F,
    streams: Vec<Option<IOVec<T, S>>>,
    capacity: usize,
}

impl<T, S, F> ByteStream<T, S, F>
where
    T: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S>,
{
    /// Create a new `ByteStream` building block with a set `capacity`.
    /// Key/value pairs of this building block will be stored on byte
    /// streams generated by a
    /// [`factory`](trait.StreamFactory.html).
    pub fn new(factory: F, capacity: usize) -> Self {
        let max_streams = 8 * std::mem::size_of::<usize>();
        let mut streams =
            Vec::<Option<IOVec<T, S>>>::with_capacity(max_streams);
        for _ in 0..max_streams {
            streams.push(None)
        }

        ByteStream {
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

impl<'a, K, V, S, F> BuildingBlock<'a, K, V> for ByteStream<(K, V), S, F>
where
    K: 'a + DeserializeOwned + Serialize + Eq,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: 'a + Stream,
    F: StreamFactory<S>,
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

    /// Remove up to `n` values from the container.
    /// If less than `n` values are stored in the container,
    /// the returned vector contains all the container values and
    /// the container is left empty.
    /// This building block implements the trait
    /// [`Ordered`](../policy/trait.Ordered.html), which means that
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

impl<K, V: Ord, S, F> Ordered<V> for ByteStream<(K, V), S, F>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S>,
{
}

//------------------------------------------------------------------------//
// Get trait implementation
//------------------------------------------------------------------------//

/// In memory read-only representation of a key/value pair in a stream.
///
/// `StreamCell` can be dereferenced into the actual value inside the
/// stream.
pub struct StreamCell<K, V> {
    item: IOStruct<(K, V)>,
}

impl<K, V> Deref for StreamCell<K, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.item.deref().1
    }
}

/// In memory read-write representation of a key/value pair in a stream.
///
/// `StreamMutCell` can be dereferenced into the actual value inside the
/// stream. If the value inside a `StreamMutCell` is modified via a call
/// to `deref_mut()`, then the key/value pair is written back to the
/// stream it comes from when the `StreamMutCell` is destroyed.
pub struct StreamMutCell<K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Stream,
{
    item: IOStructMut<(K, V), S>,
}

impl<K, V, S> Deref for StreamMutCell<K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Stream,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.item.deref().1
    }
}

impl<K, V, S> DerefMut for StreamMutCell<K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Stream,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item.deref_mut().1
    }
}

impl<K, V, F, S> Get<K, V, StreamCell<K, V>> for ByteStream<(K, V), S, F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S>,
{
    /// Get value inside a `ByteStream`. The value is wrapped inside a
    /// [`StreamCell`](struct.StreamCell.html). The `StreamCell` can
    /// further be dereferenced into a value reference.
    ///
    /// ## Safety:
    ///
    /// The return value inside the `StreamCell` is a copy of
    /// the value inside the stream. Ideally, the stream should not
    /// be updated or disapear while the returned `StreamCell` is still
    /// in use. If the stream is modified, the value inside the `StreamCell`
    /// may no longer accurately represent the value inside the stream.
    ///
    /// ## Example:
    ///
    /// ```
    /// use cache::{BuildingBlock, Get};
    /// use cache::container::ByteStream;
    /// use cache::container::stream::vec_stream::VecStreamFactory;
    ///
    /// // Make a stream and populate it.
    /// // Array with 3 elements capacity.
    /// let mut c = ByteStream::new(VecStreamFactory{}, 1);
    /// c.push(vec![(1,1)]);
    ///
    /// // Get the value inside the vector.
    /// let v = unsafe { c.get(&1).unwrap() };
    ///
    /// // Replace with another value.
    /// c.flush();
    /// c.push(vec![(2,2)]);
    ///
    /// // Val is not updated to the content of the stream.
    /// assert!(*v == 1);
    /// ```
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
}

impl<K, V, F, S> GetMut<K, V, StreamMutCell<K, V, S>>
    for ByteStream<(K, V), S, F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S>,
{
    /// Get a mutable value inside a `ByteStream`. The value is wrapped
    /// inside a [`StreamMutCell`](struct.StreamMutCell.html).
    /// The `StreamMutCell` can further be dereferenced into a value
    /// reference.
    ///
    /// ## Safety:
    ///
    /// The return value inside the `StreamMutCell` is a copy of
    /// the value inside the stream. If the value is modified, it is
    /// written back to the stream at the same position.
    /// The stream should not be updated or disapear while the
    /// returned `StreamMutCell` is still in use. If the latter happens,
    /// all the subsequent uses of this container are undefined behavior.
    ///
    /// ## Example:
    ///
    /// ```
    /// use cache::{BuildingBlock, Get, GetMut};
    /// use cache::container::ByteStream;
    /// use cache::container::stream::vec_stream::VecStreamFactory;
    ///
    /// // Make a stream and populate it.
    /// let mut c = ByteStream::new(VecStreamFactory{}, 1);
    /// c.push(vec![(1,1)]);
    ///
    /// // Get the value inside the vector.
    /// let mut v = unsafe { c.get_mut(&1).unwrap() };
    /// *v = 3;
    /// drop(v);
    ///
    /// // Check it is indeed updated:
    /// let v = unsafe { c.get(&1).unwrap() };
    /// assert_eq!(*v, 3);
    /// ```
    unsafe fn get_mut(
        &mut self,
        key: &K,
    ) -> Option<StreamMutCell<K, V, S>> {
        self.streams
            .iter_mut()
            .filter_map(|s| s.as_mut())
            .find_map(|s| {
                s.iter_mut().find_map(|item| {
                    let (k, _) = &*item;
                    if k == key {
                        Some(StreamMutCell { item: item })
                    } else {
                        None
                    }
                })
            })
    }
}

//------------------------------------------------------------------------//
// Prefetch Trait Implementation
//------------------------------------------------------------------------//

impl<'a, K, V, S, F> Prefetch<'a, K, V> for ByteStream<(K, V), S, F>
where
    K: 'a + DeserializeOwned + Serialize + Ord,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: 'a + Stream,
    F: StreamFactory<S>,
{
    fn prefetch(&mut self, _keys: Vec<K>) {}
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut ret = Vec::with_capacity(keys.len());
        keys.sort();
        for stream in self.streams.iter_mut().filter_map(|s| s.as_ref()) {
            for (k, v) in stream.iter().map(|x| x.unwrap()) {
                match keys.binary_search(&k) {
                    Ok(i) => {
                        ret.push((k, v));
                        keys.remove(i);
                    }
                    Err(_) => {}
                }
            }
        }
        ret
    }
}

//------------------------------------------------------------------------//
//  Tests
//------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::ByteStream;
    use crate::container::stream::vec_stream::VecStreamFactory;
    use crate::policy::tests::test_ordered;
    use crate::tests::{
        test_building_block, test_get, test_get_mut, test_prefetch,
    };

    #[test]
    fn building_block() {
        for i in vec![0, 10, 100] {
            test_building_block(ByteStream::new(VecStreamFactory {}, i));
        }
    }

    #[test]
    fn ordered() {
        for i in vec![0, 10, 100] {
            test_ordered(ByteStream::new(VecStreamFactory {}, i));
        }
    }

    #[test]
    fn get() {
        for i in vec![0, 10, 100] {
            test_get(ByteStream::new(VecStreamFactory {}, i));
            test_get_mut(ByteStream::new(VecStreamFactory {}, i));
        }
    }

    #[test]
    fn prefetch() {
        for i in vec![0, 10, 100] {
            test_prefetch(ByteStream::new(VecStreamFactory {}, i));
        }
    }
}
