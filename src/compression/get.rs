use super::Compressed;
use crate::stream::Stream;
use crate::{Get, GetMut};
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Deref, DerefMut};
use std::vec::Vec;

/// Simple struct wrapping a local copy of the value in a
/// `Compressed` building block.
pub struct CompressedCell<V> {
    value: V,
}

impl<V: Ord> Deref for CompressedCell<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Struct wrapping a mutable local copy of the value in a
/// `Compressed` building block.
///
/// The local copy gets written back into the underlying compressed stream
/// when this structure is dropped.
/// The memory footprint of this the total amount of elements in the
/// compressed stream. If you hold several cells of the same compressed,
/// the footprint is multiplied by the amount of cells.
///
/// ## Safety:
///
/// On top of the memory footprint, if multiple cells of the same
/// `Compressed` live and are modified in the same scope, only the last
/// one dropped will be committed back to the compressed stream.
pub struct CompressedMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    stream: Compressed<(K, V), S>,
    elements: Vec<(K, V)>,
    index: usize,
    is_written: bool,
}

impl<K, V, S> Deref for CompressedMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.elements.get(self.index).unwrap().1
    }
}

impl<K, V, S> DerefMut for CompressedMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_written = true;
        &mut self.elements.get_mut(self.index).unwrap().1
    }
}

impl<K, V, S> Drop for CompressedMutCell<K, V, S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
    S: Stream,
{
    fn drop(&mut self) {
        if !self.is_written {
            return;
        }

        self.stream
            .write(&self.elements)
            .expect("Could not write new elements to Compressed");
    }
}

impl<K, V, S> Get<K, V, CompressedCell<V>> for Compressed<(K, V), S>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize + Ord,
    S: Stream,
{
    unsafe fn get(&self, key: &K) -> Option<CompressedCell<V>> {
        // Read elements into memory.
        match self.read() {
            Err(_) => None,
            Ok(v) => v.into_iter().find_map(|(k, v)| {
                if &k == key {
                    Some(CompressedCell { value: v })
                } else {
                    None
                }
            }),
        }
    }
}

impl<K, V, S> GetMut<K, V, CompressedMutCell<K, V, S>>
    for Compressed<(K, V), S>
where
    K: DeserializeOwned + Serialize + Eq,
    V: DeserializeOwned + Serialize + Ord,
    S: Stream,
{
    unsafe fn get_mut(
        &mut self,
        key: &K,
    ) -> Option<CompressedMutCell<K, V, S>> {
        // Read elements into memory.
        let v = match self.read() {
            Err(_) => return None,
            Ok(v) => v,
        };

        // Find index of matching key.
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

        // Return cell
        Some(CompressedMutCell {
            stream: self.shallow_copy(),
            elements: v,
            index: i,
            is_written: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Compressed;
    use crate::stream::VecStream;
    use crate::tests::{test_get, test_get_mut};

    #[test]
    fn get() {
        for i in [0usize, 10usize, 100usize] {
            test_get(Compressed::new(VecStream::new(), i));
            test_get_mut(Compressed::new(VecStream::new(), i));
        }
    }
}
