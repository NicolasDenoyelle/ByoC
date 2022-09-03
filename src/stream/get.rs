use super::ByteStream;
use crate::stream::{IOStructMut, Stream, StreamFactory};
use crate::{Get, GetMut};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// In memory read-only representation of a key/value pair in a stream.
///
/// `StreamCell` can be dereferenced into the actual value inside the
/// stream.
pub struct StreamCell<V> {
    item: V,
}

impl<V> Deref for StreamCell<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.item
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
    unused: PhantomData<S>,
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

impl<K, V, F, S> Get<K, V, StreamCell<V>> for ByteStream<(K, V), S, F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S>,
{
    /// Get value inside a `Stream`. The value is wrapped inside a
    /// [`StreamCell`](struct.StreamCell.html). The `StreamCell` can
    /// further be dereferenced into a value reference.
    ///
    /// ## Safety:
    ///
    /// The return value inside the `StreamCell` is a copy of
    /// the value inside the stream. Ideally, the stream should not
    /// be updated or disappear while the returned `StreamCell` is still
    /// in use. If the stream is modified, the value inside the `StreamCell`
    /// may no longer accurately represent the value inside the stream.
    ///
    /// ## Example:
    ///
    /// ```
    /// use byoc::{BuildingBlock, Get};
    /// use byoc::Stream;
    /// use byoc::utils::stream::VecStreamFactory;
    ///
    /// // Make a stream and populate it.
    /// // Array with 3 elements capacity.
    /// let mut c = Stream::new(VecStreamFactory{}, 1);
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
    unsafe fn get(&self, key: &K) -> Option<StreamCell<V>> {
        self.stream.iter().filter_map(|s| s.as_ref()).find_map(|s| {
            s.iter().find_map(|item| {
                let (k, item) = item.unwrap();
                if &k == key {
                    Some(StreamCell { item })
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
    /// Get a mutable value inside a `Stream`. The value is wrapped
    /// inside a [`StreamMutCell`](struct.StreamMutCell.html).
    /// The `StreamMutCell` can further be dereferenced into a value
    /// reference.
    ///
    /// ## Safety:
    ///
    /// The return value inside the `StreamMutCell` is a copy of
    /// the value inside the stream. If the value is modified, it is
    /// written back to the stream at the same position.
    /// The stream should not be updated or disappear while the
    /// returned `StreamMutCell` is still in use. If the latter happens,
    /// all the subsequent uses of this container are undefined behavior.
    ///
    /// ## Example:
    ///
    /// ```
    /// use byoc::{BuildingBlock, Get, GetMut};
    /// use byoc::Stream;
    /// use byoc::utils::stream::VecStreamFactory;
    ///
    /// // Make a stream and populate it.
    /// let mut c = Stream::new(VecStreamFactory{}, 1);
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
        self.stream
            .iter_mut()
            .filter_map(|s| s.as_mut())
            .find_map(|s| {
                s.iter_mut().find_map(|item| {
                    let (k, _) = &*item;
                    if k == key {
                        Some(StreamMutCell {
                            item,
                            unused: PhantomData,
                        })
                    } else {
                        None
                    }
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::ByteStream;
    use crate::stream::VecStreamFactory;
    use crate::tests::{test_get, test_get_mut};

    #[test]
    fn get() {
        for i in [0usize, 10usize, 100usize] {
            test_get(ByteStream::new(VecStreamFactory {}, i));
            test_get_mut(ByteStream::new(VecStreamFactory {}, i));
        }
    }
}
