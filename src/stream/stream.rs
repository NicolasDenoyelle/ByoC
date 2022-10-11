use crate::internal::math::log2;
use crate::stream::{IOVec, StreamFactory};
use serde::{de::DeserializeOwned, Serialize};
use std::vec::Vec;

/// [`BuildingBlock`](trait.BuildingBlock.html) implementation serializing its
/// elements in a byte [`Stream`](utils/stream/trait.Stream.html).
///
/// The byte stream of a `Stream` can be any kind of byte stream
/// implementing the trait
/// [`Stream`](utils/stream/trait.Stream.html) such as a
/// [file](utils/stream/struct.FileStream.html) or a
/// [vector](utils/stream/struct.VecStream.html).
///
/// This building block implementation behaves similarly as the
/// [`Array`](struct.Array.html) building block implementation.
/// (Key,Value) pairs are stored together as a vector element in a
/// continuous and contiguous list of elements on the backend
/// [`Stream`](utils/stream/trait.Stream.html). For optimization reason,
/// elements of such a vector need to be all of the same size.
/// If a (Key,Value) pair does not fit the size of an element, it is
/// stored on a different [`Stream`](utils/stream/trait.Stream.html) vector
/// where elements are large enough to fit it.
/// For a given (Key,Value) pair, the size of the corresponding element is
/// the closest power of two fitting the serialized pair.
///
/// [`Streams`](utils/stream/trait.Stream.html) are generated from a structure
/// implementing the trait [`StreamFactory`](stream/trait.StreamFactory.html),
/// such as [`VecStreamFactory`](stream/struct.VecStreamFactory.html) or
/// [`TempFileStreamFactory`](utils/stream/struct.TempFileStreamFactory.html).
///
/// ## Examples
///
/// ```
/// use byoc::BuildingBlock;
/// use byoc::Stream;
/// use byoc::utils::stream::VecStreamFactory;
///
/// // The size of one element in the stream is the nearest superior power
/// // of two size of elements inserted.
/// let element_size = std::mem::size_of::<(i32, i32)>();
/// // Array with 3 elements capacity.
/// let mut c = Stream::new(VecStreamFactory{}, 3 * element_size);
///
/// // BuildingBlock as room for 3 elements and returns an empty vector.
/// // No element is rejected.
/// assert!(c.push(vec![(1, 4), (2, 2), (3, 3)]).pop().is_none());
///
/// // Stream is full and pops extra inserted value (all values here).
/// let (key, _) = c.push(vec![(4, 12)]).pop().unwrap();
/// assert_eq!(key, 4);
///
/// // Stream pops elements in order of the highest values.
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, 1);
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, 3);
/// let (key, value) = c.pop(1).pop().unwrap();
/// assert_eq!(key, 2);
/// ```
pub struct ByteStream<T, F>
where
    T: DeserializeOwned + Serialize,
    F: StreamFactory,
{
    pub(super) factory: F,
    pub(super) stream: Vec<Option<IOVec<T, F::Stream>>>,
    pub(super) capacity: usize,
}

impl<T, F> ByteStream<T, F>
where
    T: DeserializeOwned + Serialize,
    F: StreamFactory,
{
    /// Create a new `Stream` building block with a set `capacity`.
    /// Key/value pairs of this building block will be stored on byte
    /// stream generated by a
    /// [`factory`](stream/trait.StreamFactory.html).
    pub fn new(factory: F, capacity: usize) -> Self {
        let max_stream = 8 * std::mem::size_of::<usize>();
        let mut stream =
            Vec::<Option<IOVec<T, F::Stream>>>::with_capacity(max_stream);
        for _ in 0..max_stream {
            stream.push(None)
        }

        ByteStream {
            factory,
            stream,
            capacity,
        }
    }

    /// Returns the position of the most significant byte
    /// starting from the left and associated power of two.
    /// The power of two is the size of the chunk that will hold the
    /// serialized value of the `size` provided as input.
    pub(super) fn chunk_size(size: usize) -> (usize, usize) {
        let i = log2(size as u64) as usize;
        if (1usize << i) == size {
            (i, size)
        } else {
            assert!(i < 64usize); // Size overflow.
            (i + 1usize, 1usize << (i + 1))
        }
    }
}
