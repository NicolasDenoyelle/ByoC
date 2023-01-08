#[allow(clippy::module_inception)]
mod array;
pub use array::Array;
mod building_block;
mod get;
pub use get::{ArrayCell, ArrayMutCell};
pub(crate) mod builder;
#[cfg(feature = "config")]
pub(crate) mod config;
