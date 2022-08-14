/// This is a marker trait for [`BuildingBlock`](trait.BuildingBlock.html).
/// When this trait is implemented, the building blocks will
/// [pop](trait.BuildingBlock.html#tymethod.pop) values in descending
/// order.
pub trait Ordered<V: std::cmp::Ord> {}
