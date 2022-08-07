use super::Batch;
use crate::BuildingBlock;
use std::collections::LinkedList;

impl<'a, K, V, C> BuildingBlock<'a, K, V> for Batch<C>
where
    K: 'a,
    V: 'a + Ord,
    C: BuildingBlock<'a, K, V>,
{
    fn capacity(&self) -> usize {
        self.bb.iter().map(|c| c.capacity()).sum()
    }

    fn count(&self) -> usize {
        self.bb.iter().map(|c| c.count()).sum()
    }

    fn contains(&self, key: &K) -> bool {
        self.bb.iter().any(|c| c.contains(key))
    }

    fn take(&mut self, key: &K) -> Option<(K, V)> {
        self.bb.iter_mut().find_map(|c| c.take(key))
    }

    fn take_multiple(&mut self, keys: &mut Vec<K>) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(keys.len());
        for c in self.bb.iter_mut() {
            if keys.is_empty() {
                break;
            }
            out.append(&mut c.take_multiple(keys))
        }
        out
    }

    fn pop(&mut self, n: usize) -> Vec<(K, V)> {
        let mut out = Vec::with_capacity(n * self.bb.len());

        for bb in self.bb.iter_mut() {
            out.append(&mut bb.pop(n));
        }

        let len = out.len();
        if n > len {
            out
        } else {
            out.sort_unstable_by(|(_, v1), (_, v2)| v1.cmp(v2));
            let victims = out.split_off(len - n);
            assert!(self.push(out).pop().is_none());
            victims
        }
    }

    fn push(&mut self, mut values: Vec<(K, V)>) -> Vec<(K, V)> {
        let mut bb = LinkedList::new();
        loop {
            if values.is_empty() {
                break;
            }
            let mut c = match self.bb.pop_front() {
                None => break,
                Some(c) => c,
            };
            values = c.push(values);
            bb.push_back(c);
        }
        self.bb.append(&mut bb);
        values
    }

    fn flush(&mut self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(
            self.bb
                .iter_mut()
                .map(|c| c.flush())
                .collect::<Vec<Box<dyn Iterator<Item = (K, V)> + 'a>>>()
                .into_iter()
                .flatten(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Batch;
    use crate::tests::test_building_block;
    use crate::Array;

    #[test]
    fn building_block() {
        test_building_block(Batch::<Array<(u16, u32)>>::new());
        test_building_block(Batch::from([Array::new(0)]));
        test_building_block(Batch::from([Array::new(0), Array::new(0)]));
        test_building_block(Batch::from([Array::new(0), Array::new(10)]));
        test_building_block(Batch::from([Array::new(10), Array::new(0)]));
        test_building_block(Batch::from([Array::new(10), Array::new(10)]));
    }
}
