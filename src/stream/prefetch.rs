use super::ByteStream;
use crate::stream::{Stream, StreamFactory};
use crate::Prefetch;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, K, V, S, F> Prefetch<'a, K, V> for ByteStream<'a, (K, V), S, F>
where
    K: 'a + DeserializeOwned + Serialize + Ord,
    V: 'a + DeserializeOwned + Serialize + Ord,
    S: 'a + Stream<'a>,
    F: StreamFactory<S>,
{
    fn prefetch(&mut self, _keys: Vec<K>) {}
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
}

#[cfg(test)]
mod tests {
    use super::ByteStream;
    use crate::stream::VecStreamFactory;
    use crate::tests::test_prefetch;

    #[test]
    fn prefetch() {
        for i in [0usize, 10usize, 100usize] {
            test_prefetch(ByteStream::new(VecStreamFactory {}, i));
        }
    }
}
