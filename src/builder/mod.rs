mod builder;
pub use builder::Begin;

/// Build a specific container builder.
pub mod builders {
    pub use crate::array::builder::ArrayBuilder;
    pub use crate::associative::builder::AssociativeBuilder;
    pub use crate::btree::builder::BTreeBuilder;
    #[cfg(feature = "compression")]
    pub use crate::compression::builder::CompressorBuilder;
    pub use crate::multilevel::builder::MultilevelBuilder;
    pub use crate::policy::builder::PolicyBuilder;
    pub use crate::profiler::builder::ProfilerBuilder;
    pub use crate::sequential::builder::SequentialBuilder;
    #[cfg(feature = "stream")]
    pub use crate::stream::builder::StreamBuilder;
}

/// Traits enabling builders chaining capabilities.
pub mod traits {
    use crate::associative::builder::AssociativeBuilder;
    use crate::multilevel::builder::MultilevelBuilder;
    use crate::policy::builder::PolicyBuilder;
    use crate::policy::{Reference, ReferenceFactory};
    use crate::profiler::builder::ProfilerBuilder;
    use crate::sequential::builder::SequentialBuilder;
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
        fn into_associative(
            self,
            key_hasher: H,
            num_keys: usize,
        ) -> AssociativeBuilder<C, H, Self>
        where
            Self: Sized,
        {
            AssociativeBuilder::new(self, key_hasher, num_keys)
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
