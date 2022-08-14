use super::Compressor;
use crate::stream::Stream;
use crate::Prefetch;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, K, V, S> Prefetch<'a, K, V> for Compressor<'a, (K, V), S>
where
    K: 'a + DeserializeOwned + Serialize + Ord,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: Stream<'a>,
{
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
                panic!("Could not write updated elements to Compressor")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Compressor;
    use crate::stream::VecStream;
    use crate::tests::test_prefetch;

    #[test]
    fn prefetch() {
        for i in [0usize, 10usize, 100usize] {
            test_prefetch(Compressor::new(VecStream::new(), i));
        }
    }
}
