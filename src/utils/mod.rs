/// Utils for obtaining multiple writable shallow copies of a resource.
pub mod clone;
/// Utils for reading and writing a stream in fixed sized chunks.
pub mod io;
/// Utils for ordering pointers of orderable elements.
pub mod ptr;
/// Utils for extracting n min elements.
pub mod set;
// pub mod stats;

#[cfg(test)]
pub mod vstream;
