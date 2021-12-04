mod array;
mod associative;
mod btree;
mod forward;
mod policy;
mod sequential;
#[cfg(feature = "stream")]
mod stream;

mod start;
pub use start::Start;

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
    #[cfg(feature = "stream")]
    use crate::container::stream::{Stream, StreamFactory};
    use crate::policy::{Reference, ReferenceFactory};
    #[cfg(feature = "stream")]
    use serde::{de::DeserializeOwned, Serialize};
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
        fn forward(self) -> ForwardBuilder<C, Self, R, RB>
        where
            Self: Sized,
        {
            ForwardBuilder::new(self)
        }
    }

    /// Replicate  a builder into multiple builders to later build
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

    pub trait Sequential<C>: Builder<C> {
        fn into_sequential(self) -> SequentialBuilder<C, Self>
        where
            Self: Sized,
        {
            SequentialBuilder::new(self)
        }
    }

    pub trait Array<T, B> {
        fn array(self, capacity: usize) -> B;
    }

    pub trait BTree<K: Copy + Ord, V: Ord, B> {
        fn btree(self, capacity: usize) -> B;
    }

    #[cfg(feature = "stream")]
    pub trait ByteStream<
        T: DeserializeOwned + Serialize,
        S: Stream,
        F: StreamFactory<S> + Clone,
        B,
    >
    {
        fn byte_stream(self, factory: F, capacity: usize) -> B;
    }
}
