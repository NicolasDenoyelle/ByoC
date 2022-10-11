use super::Compressed;
use crate::stream::{IOError, Stream};
use crate::BuildingBlock;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, K, V, S> BuildingBlock<'a, K, V> for Compressed<(K, V), S>
where
    K: 'a + Serialize + DeserializeOwned + Ord,
    V: 'a + Serialize + DeserializeOwned + Ord,
    S: Stream,
{
    fn capacity(&self) -> usize {
        self.capacity as usize
    }

    fn size(&self) -> usize {
        match self.read_bytes() {
            Err(_) => 0usize,
            Ok(vec) => vec.len(),
        }
    }

    fn contains(&self, key: &K) -> bool {
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return false,
            Ok(v) => v,
        };

        v.iter().any(|(k, _)| k == key)
    }

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

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(keys.len());

        // Read elements into memory.
        let mut v = match self.read() {
            Err(_) => return out,
            Ok(v) => v,
        };

        keys.sort();

        #[allow(clippy::needless_collect)]
        {
            let matches: Vec<usize> = v
                .iter()
                .enumerate()
                .filter_map(|(i, (k, _))| {
                    if keys.binary_search(k).is_ok() {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            for i in matches.into_iter().rev() {
                out.push(v.swap_remove(i));
            }
        }

        // Rewrite vector to stream.
        match self.write(&v) {
            Ok(_) => out,
            Err(_) => {
                panic!("An error occurred while rewriting the stream")
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        // Read elements into memory.
        let mut v = match self.read() {
            Ok(v) => v,
            _ => return Vec::new(),
        };

        // Sort elements by value.
        v.sort_by(|(k_a, v_a), (k_b, v_b)| {
            (v_a, k_a).partial_cmp(&(v_b, k_b)).unwrap()
        });

        // Find where to cut elements
        let mut size = 0usize;
        let mut split: Option<usize> = None;
        for (i, e) in v.iter().enumerate().rev() {
            match bincode::serialized_size(e) {
                Err(_) => break,
                Ok(s) => size += s as usize,
            }
            if size >= n {
                split = Some(i);
                break;
            }
        }

        // If there was an error computing serialized size of the first
        // element, we don't pop anything.
        if size == 0 {
            return Vec::new();
        }

        match split {
            // If we walked the entire vector without being able to
            // clear requested size, we pop the whole container.
            None => match self.stream.resize(0) {
                Err(_) => Vec::new(),
                Ok(_) => v,
            },
            // We split the container elements to free requested space.
            Some(i) => {
                let out = v.split_off(i);
                match self.write(&v) {
                    Ok(_) => out,
                    Err(_) => panic!(
                        "An error occurred while rewriting the stream"
                    ),
                }
            }
        }
    }

    fn push(&mut self, values: Vec<(K, V)>) -> Vec<(K, V)> {
        // Read and decompress bytes from the stream.
        let bytes = match self.read_bytes() {
            Err(_) => return values,
            Ok(bytes) => bytes,
        };

        // Compute total serialized size.
        let mut size = bytes.len() as u64;

        // Deserialize bytes into a vector.
        let mut vec = if !bytes.is_empty() {
            match bincode::deserialize_from(bytes.as_slice()) {
                Ok(vec) => vec,
                Err(_) => return values,
            }
        } else {
            Vec::new()
        };
        let i = vec.len();

        // Walk elements and sort those going inside or outside the container
        let mut out = Vec::<(K, V)>::new();
        for kv in values.into_iter() {
            match bincode::serialized_size(&kv) {
                Err(_) => out.push(kv),
                Ok(s) => {
                    if size + s >= self.capacity {
                        out.push(kv);
                    } else {
                        vec.push(kv);
                        size += s;
                    }
                }
            };
        }

        // Write new vector to stream and return not inserted keys.
        match self.write(&vec) {
            Ok(_) => {}
            // This error may happen for a container of a small capacity when
            // the size of one element is less than the capacity but the size
            // of a serialized vector of one element is more than the capacity.
            // In that case we don't insert anything.
            Err(IOError::InvalidSize) => {
                out.append(&mut vec.split_off(i));
            }
            Err(_) => {
                panic!("An error occurred while rewriting the stream")
            }
        }

        out
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        // Read elements into memory.
        let v: Vec<(K, V)> = match self.read() {
            Err(_) => return Box::new(std::iter::empty()),
            Ok(v) => v,
        };

        if self.stream.resize(0).is_err() {
            return Box::new(std::iter::empty());
        }

        Box::new(v.into_iter())
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
