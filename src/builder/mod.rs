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
//! We want the [most recently used](../decorator/struct.Lru.html) elements
//! to stay in the first layer, and we want to be able to access the
//! container [concurrently](../trait.Concurrent.html).
//!
//! Without the builder pattern, such container would be built as follow:
//! ```
//! use byoc::BuildingBlock;
//! use byoc::{Array, Exclusive, Sequential, Decorator};
//! use byoc::utils::timestamp::Clock;
//! use byoc::utils::decorator::Lru;
//!
//! let front = Array::new(10000);
//! let back = Array::new(1000000);
//! let exclusive = Exclusive::new(front, back);
//! let decorator = Decorator::new(exclusive, Lru::<Clock>::new());
//! let mut container = Sequential::new(decorator);
//! container.push(vec![(1,2)]);
//! ```
//!
//! With a builder pattern, the same code becomes:
//! ```
//! use byoc::BuildingBlock;
//! use byoc::utils::timestamp::Clock;
//! use byoc::utils::decorator::Lru;
//! use byoc::builder::{Build,
//!                     Builder,
//!                     ExclusiveBuild,
//!                     DecoratorBuild,
//!                     SequentialBuild};
//!
//! let mut container = Builder::array(10000)
//!                             .exclusive(Builder::array(1000000))
//!                             .with_decorator(Lru::<Clock>::new())
//!                             .into_sequential()
//!                             .build();
//! container.push(vec![(1,2)]);
//! ```
#[allow(clippy::module_inception)]
mod builder;
pub use builder::Builder;

/// `BuildingBlock` building capability.
pub trait Build<C> {
    fn build(self) -> C;
}

pub use crate::array::builder::ArrayBuilder;
pub use crate::associative::builder::AssociativeBuilder;
pub use crate::btree::builder::BTreeBuilder;
#[cfg(feature = "compression")]
pub use crate::compression::builder::CompressedBuilder;
pub use crate::decorator::builder::DecoratorBuilder;
pub use crate::exclusive::builder::ExclusiveBuilder;
pub use crate::flush_stopper::builder::FlushStopperBuilder;
pub use crate::inclusive::builder::InclusiveBuilder;
pub use crate::profiler::builder::ProfilerBuilder;
pub use crate::sequential::builder::SequentialBuilder;
#[cfg(feature = "socket")]
pub use crate::socket::builder::{
    ServerBuild, SocketClientBuilder, SocketServerBuilder,
};
#[cfg(feature = "stream")]
pub use crate::stream::builder::StreamBuilder;

pub use crate::associative::builder::AssociativeBuild;
pub use crate::decorator::builder::DecoratorBuild;
pub use crate::exclusive::builder::ExclusiveBuild;
pub use crate::flush_stopper::builder::FlushStopperBuild;
pub use crate::inclusive::builder::InclusiveBuild;
pub use crate::profiler::builder::ProfilerBuild;
pub use crate::sequential::builder::SequentialBuild;
