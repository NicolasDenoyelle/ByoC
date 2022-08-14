use super::Profiler;
use crate::Ordered;

impl<V, C> Ordered<V> for Profiler<C>
where
    V: Ord,
    C: Ordered<V>,
{
}
