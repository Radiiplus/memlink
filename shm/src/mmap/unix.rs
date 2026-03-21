//! Unix (Linux/macOS) memory-mapped file implementation using memmap2 crate.

use std::io;
use std::path::Path;

use memmap2::MmapMut;

#[derive(Debug)]
pub struct UnixMmap {
    mmap: MmapMut,
}

impl UnixMmap {
    pub fn create<P: AsRef<Path>>(path: P, size: usize) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self { mmap })
    }

    pub fn open<P: AsRef<Path>>(path: P, size: usize) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        let metadata = file.metadata()?;
        if metadata.len() < size as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "File size ({}) is smaller than requested mapping size ({})",
                    metadata.len(),
                    size
                ),
            ));
        }

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self { mmap })
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.mmap
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.mmap
    }

    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }

    pub fn flush(&self) -> io::Result<()> {
        self.mmap.flush()
    }

    pub fn flush_region(&self, offset: usize, len: usize) -> io::Result<()> {
        self.mmap.flush_async()?;
        let _ = (offset, len);
        Ok(())
    }
}
