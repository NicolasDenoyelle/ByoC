use super::Sequential;
use crate::BuildingBlock;

impl<'a, K, V, C> BuildingBlock<'a, K, V> for Sequential<C>
where
    K: 'a,
    V: 'a,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        let _ = self.lock.lock_for(()).unwrap();
        self.container.as_ref().capacity()
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        self.lock.lock().unwrap();
        let mut container = self.container.as_mut();
        let out = Box::new(container.flush());
        self.lock.unlock();
        out
    }

    fn count(&self) -> usize {
        let _ = self.lock.lock_for(()).unwrap();
        let container = self.container.as_ref();
        container.count()
    }

    fn contains(&self, key: &K) -> bool {
        let _ = self.lock.lock_for(()).unwrap();
        let container = self.container.as_ref();
        container.contains(key)
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        let mut container = self.container.as_mut();
        container.take(key)
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let _ = self.lock.lock_for(()).unwrap();
        let mut container = self.container.as_mut();
        container.take_multiple(keys)
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let _ = self.lock.lock_mut_for(()).unwrap();
        let mut container = self.container.as_mut();
        container.pop(n)
    }

    fn push(&mut self, elements: Vec<(K, V)>) -> Vec<(K, V)> {
        match self.lock.lock_mut() {
            Ok(_) => {
                let mut container = self.container.as_mut();
                let out = container.push(elements);
                self.lock.unlock();
                out
            }
            Err(_) => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sequential;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(Sequential::new(Array::new(0)));
        test_building_block(Sequential::new(Array::new(100)));
    }
}
