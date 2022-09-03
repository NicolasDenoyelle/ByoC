use super::Array;
use crate::policy::Ordered;

// Make this container usable with a policy.
impl<K, V: Ord> Ordered<V> for Array<(K, V)> {}

#[cfg(test)]
mod tests {
    use super::Array;
    use crate::tests::test_ordered;

    #[test]
    fn ordered() {
        test_ordered(Array::new(0));
        test_ordered(Array::new(10));
        test_ordered(Array::new(100));
    }
}
