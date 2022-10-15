use super::ByteStream;
use crate::stream::{IOStructMut, Stream, StreamFactory};
use crate::utils::get::LifeTimeGuard;
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

impl<K, V, F> Get<K, V> for ByteStream<(K, V), F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    F: StreamFactory,
{
    type Target = StreamCell<V>;

    fn get(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.stream.iter().filter_map(|s| s.as_ref()).find_map(|s| {
            s.iter().find_map(|item| {
                let (k, item) = item.unwrap();
                if &k == key {
                    Some(LifeTimeGuard::new(StreamCell { item }))
                } else {
                    None
                }
            })
        })
    }
}

impl<K, V, F> GetMut<K, V> for ByteStream<(K, V), F>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize,
    F: StreamFactory,
{
    type Target = StreamMutCell<K, V, F::Stream>;

    fn get_mut(&mut self, key: &K) -> Option<LifeTimeGuard<Self::Target>> {
        self.stream
            .iter_mut()
            .filter_map(|s| s.as_mut())
            .find_map(|s| {
                s.iter_mut().find_map(|item| {
                    let (k, _) = &*item;
                    if k == key {
                        Some(LifeTimeGuard::new(StreamMutCell {
                            item,
                            unused: PhantomData,
                        }))
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
