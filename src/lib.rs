#![doc = include_str!("../README.md")]

//-------------------------------------------------------------------------
// Traits
//-------------------------------------------------------------------------

mod traits;
pub use traits::BuildingBlock;
pub use traits::Concurrent;
pub use traits::{Get, GetMut};

//-------------------------------------------------------------------------
// Containers
//-------------------------------------------------------------------------

mod array;
pub use array::Array;
mod associative;
pub use associative::Associative;
mod batch;
pub use batch::Batch;
mod btree;
pub use btree::BTree;
#[cfg(feature = "compression")]
mod compression;
#[cfg(feature = "compression")]
pub use compression::Compressed;
mod exclusive;
pub use exclusive::Exclusive;
mod inclusive;
pub use inclusive::Inclusive;
mod profiler;
pub use profiler::Profiler;
pub mod policy;
pub use policy::policy::Policy;
mod sequential;
pub use sequential::Sequential;
#[cfg(feature = "stream")]
mod stream;
#[cfg(feature = "stream")]
pub use stream::ByteStream as Stream;

//-------------------------------------------------------------------------
// Submodules
//-------------------------------------------------------------------------

pub mod builder;
#[cfg(feature = "config")]
pub mod config;

/// Traits and structures specific helpers.
pub mod utils;

//-------------------------------------------------------------------------
// Private Utils
//-------------------------------------------------------------------------

/// Private crate boilerplate code.
/// This code is not available to user but used throughout the
/// library.
mod internal;

/// Private test utilities available at test time.
///
/// This module tests the expected behavior of
/// [`BuildinlBlock`](trait.BuildingBlock.html) and
/// [`Get`] traits with `test_building_block()` and `test_get()`.
#[cfg(test)]
mod tests;
