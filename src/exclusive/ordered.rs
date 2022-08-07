use super::Exclusive;
use crate::policy::Ordered;

// Make this container usable with a policy.
impl<K, R, F, B> Ordered<R> for Exclusive<K, R, F, B>
where
    R: std::cmp::Ord,
    F: Ordered<R>,
    B: Ordered<R>,
{
}
