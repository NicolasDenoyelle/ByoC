mod array;
mod associative;
mod btree;
#[cfg(feature = "compression")]
mod compression;
mod multilevel;
mod policy;
mod profiler;
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
    #[cfg(feature = "compression")]
    pub use crate::builder::compression::CompressorBuilder;
    pub use crate::builder::multilevel::MultilevelBuilder;
    pub use crate::builder::policy::PolicyBuilder;
    pub use crate::builder::profiler::ProfilerBuilder;
    pub use crate::builder::sequential::SequentialBuilder;
    #[cfg(feature = "stream")]
    pub use crate::builder::stream::StreamBuilder;
}

/// Traits enabling builders chaining capabilities.
pub mod traits {
    use crate::builder::associative::AssociativeBuilder;
    use crate::builder::multilevel::MultilevelBuilder;
    use crate::builder::policy::PolicyBuilder;
    use crate::builder::profiler::ProfilerBuilder;
    use crate::builder::sequential::SequentialBuilder;
    use crate::policies::{Reference, ReferenceFactory};
    use crate::ProfilerOutputKind;
    use std::hash::Hasher;

    /// [Building Block](../../trait.BuildingBlock.html) building
    /// capability.
    pub trait Builder<C> {
        fn build(self) -> C;
    }

    /// [`Policy`](../../struct.Policy.html)
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
    /// [`Multilevel`](../../struct.Multilevel.html)
    /// [building block](../../trait.BuildingBlock.html).
    pub trait Multilevel<C, R, RB: Builder<R>>: Builder<C> {
        fn multilevel(
            self,
            rbuilder: RB,
        ) -> MultilevelBuilder<C, Self, R, RB>
        where
            Self: Sized,
        {
            MultilevelBuilder::new(self, rbuilder)
        }
    }

    /// Replicate a builder into multiple builders to later build
    /// an [`Associative`](../../struct.Associative.html)
    /// container.
    pub trait Associative<C, H: Hasher + Clone>:
        Builder<C> + Clone
    {
        fn into_associative<const N: usize>(
            self,
            key_hasher: H,
        ) -> AssociativeBuilder<C, H, Self, N>
        where
            Self: Sized,
        {
            AssociativeBuilder::new(self, key_hasher)
        }
    }

    /// Wrap a container builder into a
    /// [sequential](../../struct.Sequential.html) building block
    /// to secure concurrent access behind a lock.
    pub trait Sequential<C>: Builder<C> {
        fn into_sequential(self) -> SequentialBuilder<C, Self>
        where
            Self: Sized,
        {
            SequentialBuilder::new(self)
        }
    }

    /// [Profile](../../struct.Profiler.html) the preceding
    /// building block.
    pub trait Profiler<C>: Builder<C> {
        fn profile(
            self,
            name: &str,
            output: ProfilerOutputKind,
        ) -> ProfilerBuilder<C, Self>
        where
            Self: Sized,
        {
            ProfilerBuilder::new(name, output, self)
        }
    }
}
