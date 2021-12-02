use crate::container::stream::{Resize, Stream, StreamFactory};
use crate::private::clone::CloneCell;
use std::io::{Read, Result, Seek, SeekFrom, Write};

/// An implementation of a [`Stream`](trait.Stream.html) in a `Vec<u8>`.
pub struct VecStream {
    vec: CloneCell<Vec<u8>>,
    pos: usize,
}

impl VecStream {
    pub fn new() -> Self {
        VecStream {
            vec: CloneCell::new(Vec::new()),
            pos: 0usize,
        }
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
        let buf_len = buf.len();
        let vec_len = self.vec.len();

        let len = if self.pos >= vec_len {
            return Ok(0);
        } else if (vec_len - self.pos) < buf_len {
            vec_len - self.pos
        } else {
            buf_len
        };

        let range = self.pos..(self.pos + len);
        let slice = self.vec.as_slice().get(range.clone()).unwrap();
        buf.get_mut(0..len).unwrap().copy_from_slice(slice);
        self.pos += len;
        Ok(len)
    }
}

impl Write for VecStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let buf_len = buf.len();
        let vec_len = self.vec.len();

        let len = if (vec_len - self.pos) < buf_len {
            self.vec.resize(self.pos + buf_len, 0u8);
            buf_len
        } else {
            buf_len
        } as usize;

        let range = self.pos..(self.pos + len);
        let slice =
            self.vec.as_mut_slice().get_mut(range.clone()).unwrap();
        let buf = buf.get(0..len).unwrap();
        slice.copy_from_slice(buf);
        self.pos += len;
        Ok(len)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for VecStream {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let max = self.vec.len() as i64;
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

impl Resize for VecStream {
    fn resize(&mut self, size: u64) -> Result<()> {
        let size = size as usize;
        self.vec.resize(size, 0u8);
        self.pos = if self.pos > size { size } else { self.pos };
        Ok(())
    }
}

impl Stream for VecStream {}

/// A Factory yielding [`VecStream`](struct.VecStream.html) streams.
#[derive(Clone)]
pub struct VecStreamFactory {}

impl StreamFactory<VecStream> for VecStreamFactory {
    fn create(&mut self) -> VecStream {
        VecStream::new()
    }
}
