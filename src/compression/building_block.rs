use super::Compressed;
use crate::stream::Stream;
use crate::utils::size::find_cut_at_size;
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};

impl<K, V, S> BuildingBlock<K, V> for Compressed<(K, V), S>
where
    K: Serialize + DeserializeOwned + Ord,
    V: Serialize + DeserializeOwned + Ord,
    S: Stream,
{
    /// Get the maximum "size" that elements in the container can fit.
    ///
    /// This is the maximum in-memory size of the serialized container
    /// elements before compression. The compressed size will likely be smaller.
    fn capacity(&self) -> usize {
        self.capacity as usize
    }

    /// Get the size currently occupied by elements in this [`BuildingBlock`].
    ///
    /// This is the actual in-memory size of the serialized container
    /// elements before compression. The compressed size will likely be smaller.
    fn size(&self) -> usize {
        match self.read_bytes() {
            Err(_) => 0usize,
            Ok(bytes) => bytes.len(),
        }
    }

    /// Check if container contains a matching key.
    ///
    /// This method unpacks the vector of key/values in memory and iterate it
    /// one by one to find a matching key.
    fn contains(&self, key: &K) -> bool {
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return false,
            Ok(v) => v,
        };

        v.iter().any(|(k, _)| k == key)
    }

    /// Take the matching key/value pair out of the container.
    ///
    /// This method unpacks the vector of key/values in memory and iterates it
    /// one by one to find a matching key. If a matching key is found, it is
    /// taken out of the vector and the vector is compressed back to the
    /// underlying [`Stream`](utils/stream/trait.Stream.html) before returning
    /// the matching key/value pair.
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
            // The initial stream is left in an invalid state.
            Err(_) => {
                panic!("An error occurred while rewriting the stream")
            }
        }
    }

    /// Take multiple keys out of a container at once.
    ///
    /// This method unpacks the vector of key/values in memory once and
    /// iterates it one by one to find the matching keys. If a matching key is
    /// found, it is taken from the container vector and added to the vector
    /// of returned values. At the end of the iteration, if the vector to return
    /// is not empty, then the updated container vector is compressed and
    /// written back to the underlying
    /// [`Stream`](utils/stream/trait.Stream.html) before returning
    /// the matching key/value pairs.
    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(keys.len());

        // Read elements into memory.
        let mut v = match self.read() {
            Err(_) => return out,
            Ok(v) => v,
        };

        // Sort the keys to lookup to speedup lookups.
        keys.sort();

        // Store the index of matches in the container vector and the input
        // keys.
        //
        // The collect below is not needless. We collect the indices of
        // matching keys from `v` elements in ascending order to remove them
        // from the same vector `v` with a swap remove in descending order.
        #[allow(clippy::needless_collect)]
        let matches: Vec<(usize, usize)> = v
            .iter()
            .enumerate()
            .filter_map(|(container_i, (k, _))| {
                match keys.binary_search(k) {
                    Ok(keys_i) => Some((container_i, keys_i)),
                    Err(_) => None,
                }
            })
            .collect();

        // Take matched keys out of the container.
        let mut matched_keys: Vec<usize> = matches
            .into_iter()
            .rev()
            .map(|(container_i, keys_i)| {
                // Move matched keys out of the container.
                out.push(v.swap_remove(container_i));
                // Return key index in input vector.
                keys_i
            })
            .collect();

        // Remove the matched keys from the input vector of keys.
        matched_keys.sort();
        for key_i in matched_keys.into_iter().rev() {
            keys.swap_remove(key_i);
        }

        // Write back the modified container (if modified).
        if !out.is_empty() {
            self.write(&v)
                .expect("An error occurred while rewriting the stream");
        }

        out
    }

    /// Free up to `size` space from the container.
    ///
    /// This method unpacks the vector of key/values in memory.
    /// If the vector of elements is empty, an empty vector is returned.
    ///
    /// Otherwise, it sorts it by values and find where to cut it based on the
    /// the sum of the serialized size of its elements with the largest values.
    ///
    /// If the entire vector needs to be popped out, the
    /// [`Stream`](utils/stream/trait.Stream.html) where the values were stored
    /// is emptied while the key/value pairs read are returned.
    ///
    /// Otherwise, the values remaining in the container are compressed and
    /// written back to the container
    /// [`Stream`](utils/stream/trait.Stream.html) while the other values are
    /// returned.
    fn pop(&mut self, size: usize) -> Vec<(K, V)> {
        // Read elements into memory.
        let mut v = match self.read() {
            Ok(v) => v,
            _ => return Vec::new(),
        };

        // If there is nothing to pop, return early.
        if v.is_empty() {
            return Vec::new();
        }

        // Sort elements by value.
        v.sort_by(|(k_a, v_a), (k_b, v_b)| {
            (v_a, k_a).partial_cmp(&(v_b, k_b)).unwrap()
        });

        // Find where to cut elements
        let (split, split_size, _) = find_cut_at_size(
            &v,
            |e| match bincode::serialized_size(e) {
                Err(_) => 0usize,
                Ok(s) => s as usize,
            },
            size,
        );

        // If we walked the entire vector without being able to
        // clear requested size, we pop the whole container.
        if split_size < size {
            match self.stream.resize(0) {
                Err(_) => Vec::new(),
                Ok(_) => v,
            }
        }
        // Else we split input vector out of its elements to return and write it
        // back to the container stream.
        else {
            let out = v.split_off(split);
            match self.write(&v) {
                Ok(_) => out,
                Err(_) => {
                    panic!("An error occurred while rewriting the stream")
                }
            }
        }
    }

    /// Insert key/value pairs in the container.
    ///
    /// This method unpacks and deserializes the vector of key/values of the
    /// container in memory. It also computes the serialized size of input
    /// values.
    ///
    /// If there is not enough space for the values to insert, the vector
    /// of values already in the container is sorted by value and some elements
    /// are evicted to fit the ones to insert. If the input vector itself would
    /// exceed the container capacity, then this vector is sorted instead, and
    /// the elements that would overflow the container will be returned along
    /// with the elements that were previously in the container.
    ///
    /// At the end, the vector of inserted values is serialized then compressed
    /// and written to the underlying
    /// [`Stream`](utils/stream/trait.Stream.html).
    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        // Read and decompress bytes from the stream.
        let bytes = match self.read_bytes() {
            Err(_) => return values,
            Ok(bytes) => bytes,
        };

        // Deserialize bytes into a vector.
        let mut vec = if !bytes.is_empty() {
            match bincode::deserialize_from(bytes.as_slice()) {
                Ok(vec) => vec,
                Err(_) => return values,
            }
        } else {
            Vec::new()
        };

        // Compute total serialized size of values to insert.
        let total_size = match bincode::serialized_size(&values) {
            Ok(s) => s as u64,
            Err(_) => return values,
        };
        // The room for insertion in the container.
        let room = self.capacity - (bytes.len() as u64);
        // The closure to get the size of an element in the input vector.
        let get_serialized_size =
            |kv: &(K, V)| match bincode::serialized_size(&kv) {
                Ok(s) => s as usize,
                Err(_) => 0usize,
            };

        // Here we create the vector to write in the container stream and the
        // vector to return.
        //
        // It does not make sense to create a type here to remove the
        // "type_complexity"
        #[allow(clippy::type_complexity)]
        let (in_vec, out_vec): (Vec<(K, V)>, Vec<(K, V)>) =
            if total_size > self.capacity {
                // If there is more values to insert than the container capacity:
                // We remove the highest values from the values to insert.
                values.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
                let (cut, _, _) = find_cut_at_size(
                    &values,
                    get_serialized_size,
                    (total_size - self.capacity) as usize,
                );
                vec.append(&mut values.split_off(cut));
                // We will add the new truncated values and will return the old
                // values with the right hand side of the truncateture from
                // the new values to insert.
                (values, vec)
            } else if total_size <= room {
                // If there is enough room for the new values, we add them to
                // the old one and we will insert the combination of both and
                // return an empty vector.
                vec.append(&mut values);
                (vec, Vec::new())
            } else {
                // If none of the previous branches are taken, we need to evict
                // some old values to make more rooms for the new ones.
                // First we sort the old values.
                vec.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
                // Then we find the cut to leave just enough room for the new
                // values.
                let (cut, _, _) = find_cut_at_size(
                    &vec,
                    get_serialized_size,
                    (total_size - room) as usize,
                );

                // We will insert the new values and return the evicted values.
                let victims = vec.split_off(cut);
                vec.append(&mut values);
                (vec, victims)
            };

        // Write new vector to stream and return not inserted keys.
        self.write(&in_vec)
            .expect("An error occurred while rewriting the stream");
        out_vec
    }

    type FlushIterator = std::vec::IntoIter<(K, V)>;

    /// Empty the container and retrieve all of its elements.
    ///
    /// This function reads, uncompress and deserializes the container
    /// key/value pairs from the underlying
    /// [`Stream`](utils/stream/trait.Stream.html) into the memory.
    /// Then the stream is resized to a `0` size.
    /// An iterator of the read values is returned with all of its values
    /// in the memory.
    fn flush(&mut self) -> Self::FlushIterator {
        // Read elements into memory.
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return Vec::new().into_iter(),
            Ok(v) => v,
        };

        if self.stream.resize(0).is_err() {
            return Vec::new().into_iter();
        }

        v.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Compressed;
    use crate::stream::VecStream;
    use crate::tests::test_building_block;

    #[test]
    fn building_block() {
        for i in [0usize, 10usize, 100usize] {
            test_building_block(
                Compressed::new(VecStream::new(), i),
                true,
            );
        }
    }
}
