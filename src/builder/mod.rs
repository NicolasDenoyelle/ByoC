//! Builder pattern to instantiate a cache architecture.
//!
//! Builder module provides the tools to ease the process of building
//! a complex building block chain.
//!
//! For instance, consider the following target architecture:   
//! A container made of two layers, where the first layer
//! uses an [Array](../struct.Array.html)
//! [building block](../trait.BuildingBlock.html) with a capacity
//! of 10000 elements. The second layer uses an
//! [Array](../struct.Array.html) building block with
//! a capacity of 1000000 elements. The two containers are connected
//! with a [Exclusive](../struct.Exclusive.html) connector.
//! We want the [most recently used](../policy/struct.Lru.html) elements
//! to stay in the first layer, and we want to be able to access the
//! container [concurrently](../trait.Concurrent.html).
//!
//! Without the builder pattern, such container would be built as follow:
//! ```
//! use byoc::BuildingBlock;
//! use byoc::{Array, Exclusive, Sequential, Policy};
//! use byoc::policy::{Lru, timestamp::Clock};
//!
//! let front = Array::new(10000);
//! let back = Array::new(1000000);
//! let exclusive = Exclusive::new(front, back);
//! let policy = Policy::new(exclusive, Lru::<Clock>::new());
//! let mut container = Sequential::new(policy);
//! container.push(vec![(1,2)]);
//! ```
//!
//! With a builder pattern, the same code becomes:
//! ```
//! use byoc::BuildingBlock;
//! use byoc::policy::{Lru, timestamp::Clock};
//! use byoc::builder::{Build,Builder};
//!
//! let mut container = Builder::array(10000)
//!                             .exclusive(Builder::array(1000000))
//!                             .with_policy(Lru::<Clock>::new())
//!                             .into_sequential()
//!                             .build();
//! container.push(vec![(1,2)]);
//! ```
#[allow(clippy::module_inception)]
mod builder;
pub use builder::Builder;

/// `Build` trait implementers that can be obtained from `Builder` struct.
pub mod builders {
    pub use crate::array::builder::ArrayBuilder;
    pub use crate::associative::builder::AssociativeBuilder;
    pub use crate::btree::builder::BTreeBuilder;
    #[cfg(feature = "compression")]
    pub use crate::compression::builder::CompressedBuilder;
    pub use crate::exclusive::builder::ExclusiveBuilder;
    pub use crate::inclusive::builder::InclusiveBuilder;
    pub use crate::policy::builder::PolicyBuilder;
    pub use crate::profiler::builder::ProfilerBuilder;
    pub use crate::sequential::builder::SequentialBuilder;
    #[cfg(feature = "stream")]
    pub use crate::stream::builder::StreamBuilder;
}

use crate::associative::builder::AssociativeBuilder;
use crate::exclusive::builder::ExclusiveBuilder;
use crate::inclusive::builder::InclusiveBuilder;
use crate::policy::builder::PolicyBuilder;
use crate::policy::{Ordered, ReferenceFactory};
use crate::profiler::builder::ProfilerBuilder;
use crate::sequential::builder::SequentialBuilder;
use crate::utils::profiler::ProfilerOutputKind;
use std::hash::Hasher;

/// `BuildingBlock` building capability.
pub trait Build<C> {
    fn build(self) -> C;

    /// Wrap a container builder into a
    /// [sequential](../../struct.Sequential.html) building block
    /// to secure concurrent access behind a lock.
    ///
    /// ```
    /// use byoc::BuildingBlock;
    /// use byoc::builder::{Build,Builder};
    ///
    /// let mut container = Builder::array(10000).into_sequential().build();
    /// container.push(vec![(1,2)]);
    /// ```
    fn into_sequential(self) -> SequentialBuilder<C, Self>
    where
        Self: Sized,
    {
        SequentialBuilder::new(self)
    }

    /// Connection between two building blocks with a
    /// [`Exclusive`](../../struct.Exclusive.html)
    /// [building block](../../trait.BuildingBlock.html).
    ///
    /// ```
    /// use byoc::BuildingBlock;
    /// use byoc::builder::{Build,Builder};
    ///
    /// let front = Builder::array(10000);
    /// let back = Builder::array(10000);
    /// let mut container = front.exclusive(back).build();
    /// container.push(vec![(1,2)]);
    /// ```
    fn exclusive<R, RB: Build<R>>(
        self,
        rbuilder: RB,
    ) -> ExclusiveBuilder<C, Self, R, RB>
    where
        Self: Sized,
    {
        ExclusiveBuilder::new(self, rbuilder)
    }

    /// Connection between two building blocks with a
    /// [`Inclusive`](../../struct.Inclusive.html)
    /// [building block](../../trait.BuildingBlock.html).
    ///
    /// ```
    /// use byoc::BuildingBlock;
    /// use byoc::builder::{Build,Builder};
    ///
    /// let front = Builder::array(10000);
    /// let back = Builder::array(10000);
    /// let mut container = front.inclusive(back).build();
    /// container.push(vec![(1,2)]);
    /// ```
    fn inclusive<R, RB: Build<R>>(
        self,
        rbuilder: RB,
    ) -> InclusiveBuilder<C, Self, R, RB>
    where
        Self: Sized,
    {
        InclusiveBuilder::new(self, rbuilder)
    }

    /// [`Policy`](../../struct.Policy.html)
    /// wrapping capability.
    ///
    /// ```
    /// use byoc::BuildingBlock;
    /// use byoc::builder::{Build,Builder};
    /// use byoc::policy::Fifo;
    ///
    /// let mut container = Builder::array(10000)
    ///                    .with_policy(Fifo::new())
    ///                    .build();
    /// container.push(vec![(1,2)]);
    /// ```
    fn with_policy<V, F: ReferenceFactory<V>>(
        self,
        policy: F,
    ) -> PolicyBuilder<C, V, F, Self>
    where
        Self: Sized,
        C: Ordered<F::Item>,
    {
        PolicyBuilder::new(self, policy)
    }

    /// [Profile](../../struct.Profiler.html) the preceding
    /// building block.
    ///
    /// ```
    /// use byoc::BuildingBlock;
    /// use byoc::builder::{Build,Builder};
    /// use byoc::utils::profiler::ProfilerOutputKind;
    ///
    /// let mut container = Builder::array(10000)
    ///                    .profile("test", ProfilerOutputKind::Stdout)
    ///                    .build();
    /// container.push(vec![(1,2)]);
    /// ```
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

/// Replicate a builder into multiple builders to later build
/// an `Associative` container.
pub trait AssociativeBuild<C>: Build<C> + Clone {
    /// Replicate a builder into multiple builders to later build
    /// an [`Associative`](../struct.Associative.html)
    /// container.
    ///
    /// * `key_hasher`: The hasher to hash container keys and find
    /// the target bucket.
    /// * `num_sets`: The number of sets or bucket in the container.
    /// the target bucket.
    ///
    /// ```
    /// use byoc::BuildingBlock;
    /// use byoc::builder::{AssociativeBuild, Build, Builder};
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// let mut container = Builder::array(10000)
    ///                    .into_associative(DefaultHasher::new(), 8)
    ///                    .build();
    /// container.push(vec![(1,2)]);
    /// ```
    fn into_associative<H: Hasher + Clone>(
        self,
        key_hasher: H,
        num_sets: usize,
    ) -> AssociativeBuilder<C, H, Self>
    where
        Self: Sized,
    {
        AssociativeBuilder::new(self, key_hasher, num_sets)
    }
}
impl<C, B: Build<C> + Clone> AssociativeBuild<C> for B {}
