pub trait CGet<K, V, U, W>
where
    U: std::ops::Deref<Target = V>,
    W: std::ops::Deref<Target = V> + std::ops::DerefMut,
{
    fn get(&self, key: &K) -> Option<U>;
    fn get_mut(&mut self, key: &K) -> Option<W>;
}

mod vector;
pub use crate::container::vector::Vector;
mod btree;
pub use crate::container::btree::BTree;
#[cfg(feature = "stream")]
mod stream;
#[cfg(feature = "stream")]
pub use crate::container::stream::Stream;
