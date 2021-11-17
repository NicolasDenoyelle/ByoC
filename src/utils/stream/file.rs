use crate::private::lock::RWLock;
use crate::utils::stream::Stream;
#[cfg(feature = "tempfile")]
use crate::utils::stream::StreamFactory;
use std::fs::File;
use std::path::PathBuf;
#[cfg(feature = "tempfile")]
use tempfile::NamedTempFile;

/// A [`Stream`](trait.Stream.html) implementation based on a file.
///
/// The underlying file is deleted when all the clones go out of scope.
pub struct FileStream {
    file: File,
    path: PathBuf,
    rc: RWLock,
}

impl Clone for FileStream {
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
    fn drop(&mut self) {
        self.rc.unlock();
        match self.rc.try_lock_mut() {
            Err(_) => {}
            Ok(_) => std::fs::remove_file(&self.path).unwrap(),
        }
    }
}

impl crate::utils::stream::Resize for FileStream {
    fn resize(&mut self, size: u64) -> std::io::Result<()> {
        match self.file.set_len(size) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
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

impl Stream for FileStream {}

/// Factory to spawn temporary file streams.
#[cfg(feature = "tempfile")]
#[derive(Clone)]
pub struct TempFileStreamFactory {}

#[cfg(feature = "tempfile")]
impl StreamFactory<FileStream> for TempFileStreamFactory {
    fn create(&mut self) -> FileStream {
        let named_tmpfile =
            NamedTempFile::new().expect("Temporary file creation failed.");
        let path = named_tmpfile.path().to_path_buf();
        let file = named_tmpfile.into_file();
        let rc = RWLock::new();
        rc.lock().unwrap();

        FileStream {
            file: file,
            path: path,
            rc: rc,
        }
    }
}
