use super::ByteStream;
use crate::stream::{Stream, StreamFactory};
use crate::Ordered;
use serde::{de::DeserializeOwned, Serialize};

impl<'a, K, V: Ord, S, F> Ordered<V> for ByteStream<'a, (K, V), S, F>
where
    K: DeserializeOwned + Serialize,
    V: DeserializeOwned + Serialize,
    S: Stream<'a>,
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
