/// Utils for bit calculation
pub mod bits;
/// Utils for obtaining multiple writable shallow copies of a resource.
mod shared_ptr;
pub use shared_ptr::SharedPtr;
#[cfg(feature = "stream")]
/// Array implementation above a
/// [stream](../utils/stream/trait.Stream.html) and
/// utils for reading and writing a stream in fixed sized chunks.
pub mod io_vec;
/// Library custom read/write lock.
pub mod lock;
#[cfg(feature = "stream")]
/// Utils for extracting n min elements.
pub mod set;
