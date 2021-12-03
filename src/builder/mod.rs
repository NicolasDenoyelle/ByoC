use crate::policy::{Reference, ReferenceFactory};
mod array;
pub use crate::builder::array::ArrayBuilder;
mod associative;
pub use crate::builder::associative::AssociativeBuilder;
mod btree;
pub use crate::builder::btree::BTreeBuilder;
mod forward;
pub use crate::builder::forward::ForwardBuilder;
mod policy;
pub use crate::builder::policy::PolicyBuilder;
mod sequential;
pub use crate::builder::sequential::SequentialBuilder;
#[cfg(feature = "stream")]
mod stream;
#[cfg(feature = "stream")]
pub use crate::builder::stream::ByteStreamBuilder;

pub trait Builder<C> {
    fn build(self) -> C;
}

// pub trait Forward<L, R, B: Builder<R>>: Builder<L> {
//     fn forward(self) -> ForwardBuilder<L, Self, R, B>;
// }

// pub trait WithPolicy<V, C, R, F>: Builder<C>
// where
//     R: Reference<V>,
//     F: ReferenceFactory<V, R>,
// {
//     fn with_policy(self) -> PolicyBuilder<C,V,R,F,Self>;
// }
