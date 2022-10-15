use super::{Inclusive, InclusiveCell};
use crate::policy::Ordered;
use crate::BuildingBlock;

// Make this container usable with a policy.
impl<'a, K, V, L, R> Ordered<V> for Inclusive<'a, K, V, L, R>
where
    K: 'a + Clone,
    V: 'a + Clone + Ord,
    L: BuildingBlock<'a, K, InclusiveCell<V>> + Ordered<InclusiveCell<V>>,
    R: BuildingBlock<'a, K, InclusiveCell<V>> + Ordered<InclusiveCell<V>>,
{
}
