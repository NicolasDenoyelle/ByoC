use crate::internal::lock::RWLock;
#[cfg(feature = "tempfile")]
use crate::stream::StreamFactory;
use crate::stream::{Stream, StreamBase};
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;
#[cfg(feature = "tempfile")]
use tempfile::NamedTempFile;

/// A [`Stream`](../trait.Stream.html) implementation based on a file.
///
/// This structure can be cloned into multiple handles over the same
/// [`std::fs::File`]. When all the clones go out of scope, the file is deleted,
/// unless the method
/// [`do_not_delete()`](struct.FileStream.html#tymethod.do_not_delete) has
/// been called once on one of the clones.
pub struct FileStream {
    file: File,
    path: PathBuf,
    rc: RWLock,
}

impl FileStream {
    /// FileStream constructor.
    ///
    /// Attempt to open or create the file pointed by `path` for writing.
    /// On success, the function returns a new [`FileStream`]. If the file
    /// cannot be opened, an error is returned.
    fn new(path: PathBuf) -> std::io::Result<Self> {
        let file = match OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())
        {
            Err(e) => return Err(e),
            Ok(f) => f,
        };
        let rc = RWLock::new();
        rc.lock().unwrap();
        Ok(FileStream { file, path, rc })
    }

    /// Keep the file when the last clone of this [`FileStream`] is deleted.
    pub fn do_not_delete(self) -> Self {
        self.rc.lock().unwrap();
        self
    }
}

impl From<&String> for FileStream {
    fn from(s: &String) -> FileStream {
        FileStream::new(PathBuf::from(s.clone())).unwrap()
    }
}

impl Clone for FileStream {
    /// Return a copy of this [`FileStream`] with a [`std::fs::File`] handle
    /// on the same file.
    fn clone(&self) -> Self {
        self.rc.lock().unwrap();
        FileStream {
            file: self.file.try_clone().unwrap(),
            path: self.path.clone(),
            rc: self.rc.clone(),
        }
    }
}

impl Drop for FileStream {
    /// Delete this copy of the [`FileStream`]. If this was the last copy of
    /// the [`FileStream`], the underlying [`std::fs::File`] is deleted, unless
    /// the method
    /// [`do_not_delete()`](struct.FileStream.html#tymethod.do_not_delete) has
    /// been called once on one of the clones.    
    fn drop(&mut self) {
        self.rc.unlock();
        #[allow(unused_must_use)]
        match self.rc.try_lock_mut() {
            Err(_) => {}
            // Try to remove file. File might be already cleaned up by the OS.
            Ok(_) => {
                std::fs::remove_file(&self.path);
            }
        }
    }
}

impl std::io::Read for FileStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl std::io::Write for FileStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl std::io::Seek for FileStream {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl StreamBase for FileStream {
    fn box_clone(&self) -> Box<dyn StreamBase> {
        Box::new(self.clone())
    }

    fn resize(&mut self, size: u64) -> std::io::Result<()> {
        match self.file.set_len(size) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
impl Stream for FileStream {}

/// Factory to spawn temporary [`FileStream`] instances.
#[cfg(feature = "tempfile")]
#[derive(Clone)]
pub struct TempFileStreamFactory {}

#[cfg(feature = "tempfile")]
impl StreamFactory for TempFileStreamFactory {
    type Stream = FileStream;
    fn create(&mut self) -> Self::Stream {
        let named_tmpfile =
            NamedTempFile::new().expect("Temporary file creation failed.");
        let path = named_tmpfile.path().to_path_buf();
        FileStream::new(path).unwrap()
    }
}
