use super::Compressed;
use crate::policy::Ordered;
use crate::stream::Stream;
use serde::{de::DeserializeOwned, Serialize};

impl<K, V, S> Ordered<V> for Compressed<(K, V), S>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned + Ord,
    S: Stream,
{
}

#[cfg(test)]
mod tests {
    use super::Compressed;
    use crate::stream::VecStream;
    use crate::tests::test_ordered;

    #[test]
    fn ordered() {
        for i in [0usize, 10usize, 100usize] {
            test_ordered(Compressed::new(VecStream::new(), i));
        }
    }
}
