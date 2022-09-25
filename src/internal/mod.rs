/// Utils for bit calculation
pub mod bits;
/// Utils for obtaining multiple writable shallow copies of a resource.
mod shared_ptr;
pub use shared_ptr::SharedPtr;
#[cfg(feature = "stream")]
/// Utils for extracting k min elements.
pub mod kmin;
/// Library custom read/write lock.
pub mod lock;
