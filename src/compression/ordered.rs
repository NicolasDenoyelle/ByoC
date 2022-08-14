use super::Compressor;
use crate::stream::Stream;
use crate::Ordered;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, K, V, S> Ordered<V> for Compressor<'a, (K, V), S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned + Ord,
    S: Stream<'a>,
{
}

#[cfg(test)]
mod tests {
    use super::Compressor;
    use crate::stream::VecStream;
    use crate::tests::test_ordered;

    #[test]
    fn ordered() {
        for i in [0usize, 10usize, 100usize] {
            test_ordered(Compressor::new(VecStream::new(), i));
        }
    }
}
