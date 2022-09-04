use super::ByteStream;
use crate::policy::Ordered;
use crate::stream::StreamFactory;
use serde::{de::DeserializeOwned, Serialize};

impl<K, V: Ord, F> Ordered<V> for ByteStream<(K, V), F>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
    F: StreamFactory,
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
