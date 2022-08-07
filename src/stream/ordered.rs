use super::ByteStream;
use crate::policy::Ordered;
use crate::stream::{Stream, StreamFactory};
use serde::{de::DeserializeOwned, Serialize};

impl<K, V: Ord, S, F> Ordered<V> for ByteStream<(K, V), S, F>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
    S: Stream,
    F: StreamFactory<S>,
{
}

#[cfg(test)]
mod tests {
    use super::ByteStream;
    use crate::stream::VecStreamFactory;
    use crate::tests::test_ordered;

    #[test]
    fn ordered() {
        for i in [0usize, 10usize, 100usize] {
            test_ordered(ByteStream::new(VecStreamFactory {}, i));
        }
    }
}
