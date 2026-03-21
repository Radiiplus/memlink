//! Windows memory-mapped file implementation using CreateFileMappingW and MapViewOfFile.

use std::io;
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ,
    FILE_MAP_WRITE, PAGE_READWRITE,
};

#[derive(Debug)]
pub struct WindowsMmap {
    #[allow(dead_code)]
    file_handle: HANDLE,
    mapping_handle: HANDLE,
    view_ptr: *mut u8,
    size: usize,
}

unsafe impl Send for WindowsMmap {}
unsafe impl Sync for WindowsMmap {}

impl WindowsMmap {
    pub fn create<P: AsRef<Path>>(path: P, size: usize) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.as_ref())?;

        file.set_len(size as u64)?;

        let file_handle = HANDLE(file.as_raw_handle() as *mut _);

        let mapping_handle = unsafe {
            CreateFileMappingW(
                file_handle,
                None,
                PAGE_READWRITE,
                0,
                size as u32,
                None,
            )
        }
        .map_err(io::Error::other)?;

        let view_addr = unsafe {
            MapViewOfFile(
                mapping_handle,
                FILE_MAP_READ | FILE_MAP_WRITE,
                0,
                0,
                size,
            )
        };

        let view_ptr = view_addr.Value as *mut u8;

        if view_ptr.is_null() {
            unsafe {
                let _ = CloseHandle(mapping_handle);
            }
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            file_handle,
            mapping_handle,
            view_ptr,
            size,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P, size: usize) -> io::Result<Self> {
        let metadata = std::fs::metadata(path.as_ref())?;

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

        Self::create(path, size)
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.view_ptr, self.size) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.view_ptr, self.size) }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn flush(&self) -> io::Result<()> {
        use windows::Win32::System::Memory::FlushViewOfFile;

        unsafe {
            FlushViewOfFile(self.view_ptr as *const _, self.size)
                .map_err(io::Error::other)
        }
    }

    pub fn flush_region(&self, offset: usize, len: usize) -> io::Result<()> {
        use windows::Win32::System::Memory::FlushViewOfFile;

        if offset.checked_add(len).is_none_or(|end| end > self.size) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Region exceeds mapping size",
            ));
        }

        unsafe {
            FlushViewOfFile(
                (self.view_ptr as usize + offset) as *const _,
                len,
            )
            .map_err(io::Error::other)
        }
    }
}

impl Drop for WindowsMmap {
    fn drop(&mut self) {
        if !self.view_ptr.is_null() {
            unsafe {
                let addr = windows::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: self.view_ptr as *mut _,
                };
                let _ = UnmapViewOfFile(addr);
            }
        }

        if !self.mapping_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.mapping_handle);
            }
        }
    }
}
