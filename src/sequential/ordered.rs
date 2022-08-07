use super::Sequential;
use crate::policy::Ordered;

impl<V: Ord, C> Ordered<V> for Sequential<C> where C: Ordered<V> {}
