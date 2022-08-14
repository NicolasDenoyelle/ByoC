use super::Sequential;
use crate::Concurrent;

unsafe impl<C> Send for Sequential<C> {}

unsafe impl<C> Sync for Sequential<C> {}

impl<C> Concurrent for Sequential<C> {
    fn clone(&self) -> Self {
        Sequential {
            container: self.container.clone(),
            lock: self.lock.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sequential;
    use crate::tests::test_concurrent;
    use crate::Array;

    #[test]
    fn concurrent() {
        test_concurrent(Sequential::new(Array::new(0)), 64);
        test_concurrent(Sequential::new(Array::new(100)), 64);
    }
}
