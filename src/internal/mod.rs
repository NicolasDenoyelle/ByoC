/// Utils for bit calculation
pub mod bits;
/// Utils for obtaining multiple writable shallow copies of a resource.
mod shared_ptr;
pub use shared_ptr::SharedPtr;
/// Library custom read/write lock.
pub mod lock;
#[cfg(feature = "stream")]
/// Utils for extracting n min elements.
pub mod set;
