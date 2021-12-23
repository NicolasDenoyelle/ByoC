/// Utils for bit calculation
pub mod bits;
/// Utils for obtaining multiple writable shallow copies of a resource.
pub mod clone;
#[cfg(feature = "stream")]
/// Array implementation above a
/// [stream](../utils/stream/trait.Stream.html) and
/// utils for reading and writing a stream in fixed sized chunks.
pub mod io_vec;
/// Library custom read/write lock.
pub mod lock;
/// Utils for ordering pointers of orderable elements.
pub mod ptr;
#[cfg(feature = "stream")]
/// Utils for extracting n min elements.
pub mod set;
