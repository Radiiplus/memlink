//! Cross-platform memory-mapped file abstraction with unified MmapSegment enum.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::UnixMmap;
#[cfg(windows)]
pub use windows::WindowsMmap;

pub enum MmapSegment {
    #[cfg(unix)]
    Unix(UnixMmap),

    #[cfg(windows)]
    Windows(WindowsMmap),
}

impl MmapSegment {
    pub fn create<P: AsRef<std::path::Path>>(path: P, size: usize) -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            UnixMmap::create(path, size).map(MmapSegment::Unix)
        }

        #[cfg(windows)]
        {
            WindowsMmap::create(path, size).map(MmapSegment::Windows)
        }

        #[cfg(not(any(unix, windows)))]
        {
            compile_error!("Unsupported platform. Only Unix (Linux/macOS) and Windows are supported.");
        }
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P, size: usize) -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            UnixMmap::open(path, size).map(MmapSegment::Unix)
        }

        #[cfg(windows)]
        {
            WindowsMmap::open(path, size).map(MmapSegment::Windows)
        }

        #[cfg(not(any(unix, windows)))]
        {
            compile_error!("Unsupported platform. Only Unix (Linux/macOS) and Windows are supported.");
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            #[cfg(unix)]
            MmapSegment::Unix(inner) => inner.as_slice(),

            #[cfg(windows)]
            MmapSegment::Windows(inner) => inner.as_slice(),
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            #[cfg(unix)]
            MmapSegment::Unix(inner) => inner.as_mut_slice(),

            #[cfg(windows)]
            MmapSegment::Windows(inner) => inner.as_mut_slice(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            #[cfg(unix)]
            MmapSegment::Unix(inner) => inner.len(),

            #[cfg(windows)]
            MmapSegment::Windows(inner) => inner.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            #[cfg(unix)]
            MmapSegment::Unix(inner) => inner.is_empty(),

            #[cfg(windows)]
            MmapSegment::Windows(inner) => inner.is_empty(),
        }
    }

    pub fn flush(&self) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            MmapSegment::Unix(inner) => inner.flush(),

            #[cfg(windows)]
            MmapSegment::Windows(inner) => inner.flush(),
        }
    }

    pub fn flush_region(&self, offset: usize, len: usize) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            MmapSegment::Unix(inner) => inner.flush_region(offset, len),

            #[cfg(windows)]
            MmapSegment::Windows(inner) => inner.flush_region(offset, len),
        }
    }
}
