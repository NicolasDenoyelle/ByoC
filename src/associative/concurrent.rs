use super::Associative;
use crate::Concurrent;
use std::hash::Hasher;

unsafe impl<C: Send, H: Hasher + Clone> Send for Associative<C, H> {}

unsafe impl<C: Sync, H: Hasher + Clone> Sync for Associative<C, H> {}

impl<C: Concurrent, H: Hasher + Clone> Concurrent for Associative<C, H> {
    fn clone(&self) -> Self {
        Associative {
            containers: self
                .containers
                .iter()
                .map(|c| Concurrent::clone(c))
                .collect(),
            hasher: self.hasher.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Associative;
    use crate::tests::test_concurrent;
    use crate::{Array, Sequential};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn concurrent() {
        test_concurrent(
            Associative::new(
                vec![Sequential::new(Array::new(30)); 30],
                DefaultHasher::new(),
            ),
            64,
        );
    }
}
