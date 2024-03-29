use crate::stream::{Stream, StreamBase, StreamFactory};
use crate::utils::SharedPtr;
use std::io::{Read, Result, Seek, SeekFrom, Write};

/// An implementation of a [`Stream`](trait.Stream.html) in a `Vec<u8>`.
///
/// The internal vector is protected inside an `Arc` like structure and
/// will panic if concurrent access with at least one exclusive access to
/// the vector happen.
pub struct VecStream {
    vec: SharedPtr<Vec<u8>>,
    pos: usize,
}

impl VecStream {
    /// Create a new empty in-memory stream.
    pub fn new() -> Self {
        VecStream {
            vec: SharedPtr::from(Vec::new()),
            pos: 0usize,
        }
    }
}

impl Default for VecStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for VecStream {
    fn clone(&self) -> Self {
        VecStream {
            vec: self.vec.clone(),
            pos: self.pos,
        }
    }
}

impl Read for VecStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let vec = self.vec.as_ref();
        let buf_len = buf.len();
        let vec_len = vec.len();

        let len = if self.pos >= vec_len {
            return Ok(0);
        } else if (vec_len - self.pos) < buf_len {
            vec_len - self.pos
        } else {
            buf_len
        };

        let range = self.pos..(self.pos + len);
        let slice = vec.as_slice().get(range).unwrap();
        buf.get_mut(0..len).unwrap().copy_from_slice(slice);
        self.pos += len;
        Ok(len)
    }
}

impl Write for VecStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let buf_len = buf.len();
        let mut vec = self.vec.as_mut();
        let vec_len = vec.len();

        if (vec_len - self.pos) < buf_len {
            vec.resize(self.pos + buf_len, 0u8);
        }

        let len = buf_len as usize;
        let range = self.pos..(self.pos + len);
        let buf = buf.get(0..len).unwrap();
        vec.as_mut_slice()
            .get_mut(range)
            .unwrap()
            .copy_from_slice(buf);
        self.pos += len;
        Ok(len)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for VecStream {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let max = self.vec.as_ref().len() as i64;
        let pos = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => pos + max,
            SeekFrom::Current(pos) => self.pos as i64 + pos,
        };
        let pos = if pos < 0 {
            0
        } else if pos > max {
            max
        } else {
            pos
        } as u64;

        self.pos = pos as usize;
        Ok(pos)
    }
}

impl StreamBase for VecStream {
    fn box_clone(&self) -> Box<dyn StreamBase> {
        Box::new(self.clone())
    }

    fn resize(&mut self, size: u64) -> Result<()> {
        let size = size as usize;
        self.vec.as_mut().resize(size, 0u8);
        self.pos = if self.pos > size { size } else { self.pos };
        Ok(())
    }
}
impl Stream for VecStream {}

/// Factory to spawn [`VecStream`](struct.VecStream.html) instances.
#[derive(Clone)]
pub struct VecStreamFactory {}

impl StreamFactory for VecStreamFactory {
    type Stream = VecStream;
    fn create(&mut self) -> Self::Stream {
        VecStream::new()
    }
}
