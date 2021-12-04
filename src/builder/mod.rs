mod array;
mod associative;
mod btree;
mod forward;
mod policy;
mod sequential;
#[cfg(feature = "stream")]
mod stream;

mod begin;
pub use begin::Begin;

/// Build a specific container builder.
pub mod builders {
    pub use crate::builder::array::ArrayBuilder;
    pub use crate::builder::associative::AssociativeBuilder;
    pub use crate::builder::btree::BTreeBuilder;
    pub use crate::builder::forward::ForwardBuilder;
    pub use crate::builder::policy::PolicyBuilder;
    pub use crate::builder::sequential::SequentialBuilder;
    #[cfg(feature = "stream")]
    pub use crate::builder::stream::ByteStreamBuilder;
}

/// Traits enabling builders chaining capabilities.
pub mod traits {
    use crate::builder::associative::AssociativeBuilder;
    use crate::builder::forward::ForwardBuilder;
    use crate::builder::policy::PolicyBuilder;
    use crate::builder::sequential::SequentialBuilder;
    use crate::policy::{Reference, ReferenceFactory};
    use std::hash::Hasher;

    /// [Building Block](../../trait.BuildingBlock.html) building
    /// capability.
    pub trait Builder<C> {
        fn build(self) -> C;
    }

    /// [`Policy`](../../policy/policy/struct.Policy.html)
    /// wrapping capability.
    pub trait Policy<C>: Builder<C> {
        fn with_policy<V, R: Reference<V>, F: ReferenceFactory<V, R>>(
            self,
            policy: F,
        ) -> PolicyBuilder<C, V, R, F, Self>
        where
            Self: Sized,
        {
            PolicyBuilder::new(self, policy)
        }
    }

    /// Connection between two building blocks with a
    /// [`Forward`](../../connector/struct.Forward.html)
    /// [building block](../../trait.BuildingBlock.html).
    pub trait Forward<C, R, RB: Builder<R>>: Builder<C> {
        fn forward(self, rbuilder: RB) -> ForwardBuilder<C, Self, R, RB>
        where
            Self: Sized,
        {
            ForwardBuilder::new(self, rbuilder)
        }
    }

    /// Replicate a builder into multiple builders to later build
    /// an [`Associative`](../../concurrent/struct.Associative.html)
    /// container.
    pub trait Associative<C>: Builder<C> + Clone {
        fn into_associative<H: Hasher + Clone>(
            self,
            n_sets: usize,
            key_hasher: H,
        ) -> AssociativeBuilder<C, H, Self>
        where
            Self: Sized,
        {
            AssociativeBuilder::new(self, n_sets, key_hasher)
        }
    }

    /// Wrap a container builder into a
    /// [sequential](../../concurrent/struct.Sequential.html) building block
    /// to secure concurrent access behind a lock.
    pub trait Sequential<C>: Builder<C> {
        fn into_sequential(self) -> SequentialBuilder<C, Self>
        where
            Self: Sized,
        {
            SequentialBuilder::new(self)
        }
    }
}
