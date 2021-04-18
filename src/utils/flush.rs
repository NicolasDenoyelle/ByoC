use std::vec::Vec;

/// `Vec` of flush iterators flushing elements sequentially,
/// starting from last iterator until empty.
pub struct VecFlushIterator<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    pub it: Vec<Box<dyn Iterator<Item = (K, V)> + 'a>>,
}

impl<'a, K, V> Iterator for VecFlushIterator<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.it.pop() {
                None => {
                    return None;
                }
                Some(mut it) => {
                    if let Some(e) = it.next() {
                        self.it.push(it);
                        return Some(e);
                    }
                }
            }
        }
    }
}
