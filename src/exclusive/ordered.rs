use super::Exclusive;
use crate::policy::Ordered;
use crate::BuildingBlock;

// Make this container usable with a policy.
impl<'a, K, R, F, B> Ordered<R> for Exclusive<'a, K, R, F, B>
where
    K: 'a,
    R: 'a + std::cmp::Ord,
    F: BuildingBlock<'a, K, R> + Ordered<R>,
    B: BuildingBlock<'a, K, R> + Ordered<R>,
{
}
