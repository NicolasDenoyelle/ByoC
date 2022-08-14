use super::Batch;
use crate::Prefetch;

impl<'a, K, V, C> Prefetch<'a, K, V> for Batch<C>
where
    K: 'a,
    V: 'a + Ord,
    C: Prefetch<'a, K, V>,
{
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
}

#[cfg(test)]
mod tests {
    use super::Batch;
    use crate::tests::test_prefetch;
    use crate::Array;
    #[test]
    fn prefetch() {
        test_prefetch(Batch::<Array<(u16, u32)>>::new());
        test_prefetch(Batch::from([Array::new(0)]));
        test_prefetch(Batch::from([Array::new(0), Array::new(0)]));
        test_prefetch(Batch::from([Array::new(0), Array::new(10)]));
        test_prefetch(Batch::from([Array::new(10), Array::new(0)]));
        test_prefetch(Batch::from([Array::new(10), Array::new(10)]));
    }
}
