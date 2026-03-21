//! Cross-platform futex (fast userspace mutex) for efficient wait/wake signaling.
//! Uses native syscalls: Linux futex, macOS ulock, Windows WaitOnAddress.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FutexError {
    Timeout,
    Interrupted,
    InvalidArgument,
    Unsupported,
    Other(i32),
}

impl std::fmt::Display for FutexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FutexError::Timeout => write!(f, "Futex operation timed out"),
            FutexError::Interrupted => write!(f, "Futex operation interrupted"),
            FutexError::InvalidArgument => write!(f, "Invalid futex argument"),
            FutexError::Unsupported => write!(f, "Futex not supported on this platform"),
            FutexError::Other(code) => write!(f, "Futex error: {}", code),
        }
    }
}

impl std::error::Error for FutexError {}

pub type FutexResult = Result<(), FutexError>;

pub struct Futex {
    value: AtomicU32,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    _platform: LinuxFutex,
    #[cfg(target_os = "macos")]
    _platform: MacOSFutex,
    #[cfg(target_os = "windows")]
    _platform: WindowsFutex,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
struct LinuxFutex;

#[cfg(target_os = "macos")]
struct MacOSFutex;

#[cfg(target_os = "windows")]
struct WindowsFutex;

impl Futex {
    pub fn new(value: u32) -> Self {
        Self {
            value: AtomicU32::new(value),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            _platform: LinuxFutex,
            #[cfg(target_os = "macos")]
            _platform: MacOSFutex,
            #[cfg(target_os = "windows")]
            _platform: WindowsFutex,
        }
    }

    pub fn load(&self) -> u32 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn store(&self, value: u32) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn wait(&self, expected: u32, timeout: Option<Duration>) -> FutexResult {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            linux_futex_wait(&self.value, expected, timeout)
        }
        #[cfg(target_os = "macos")]
        {
            macos_futex_wait(&self.value, expected, timeout)
        }
        #[cfg(target_os = "windows")]
        {
            windows_futex_wait(&self.value, expected, timeout)
        }
        #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos", target_os = "windows")))]
        {
            let _ = (expected, timeout);
            Err(FutexError::Unsupported)
        }
    }

    pub fn wake_one(&self) -> usize {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            linux_futex_wake(&self.value, 1)
        }
        #[cfg(target_os = "macos")]
        {
            macos_futex_wake(&self.value, 1)
        }
        #[cfg(target_os = "windows")]
        {
            windows_futex_wake(&self.value, 1)
        }
        #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos", target_os = "windows")))]
        {
            0
        }
    }

    pub fn wake_all(&self) -> usize {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            linux_futex_wake(&self.value, usize::MAX)
        }
        #[cfg(target_os = "macos")]
        {
            macos_futex_wake(&self.value, usize::MAX)
        }
        #[cfg(target_os = "windows")]
        {
            windows_futex_wake(&self.value, usize::MAX)
        }
        #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos", target_os = "windows")))]
        {
            0
        }
    }

    pub fn as_atomic(&self) -> &AtomicU32 {
        &self.value
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn linux_futex_wait(
    value: &AtomicU32,
    expected: u32,
    timeout: Option<Duration>,
) -> FutexResult {
    use std::ptr;

    const FUTEX_WAIT: i32 = 0;

    if value.load(Ordering::SeqCst) != expected {
        return Err(FutexError::InvalidArgument);
    }

    let timespec = timeout.map(duration_to_timespec);
    let timespec_ptr = timespec
        .as_ref()
        .map(|ts| ts as *const libc::timespec)
        .unwrap_or(ptr::null());

    loop {
        if value.load(Ordering::SeqCst) != expected {
            return Err(FutexError::InvalidArgument);
        }

        let ret = unsafe {
            libc::syscall(
                libc::SYS_futex,
                value as *const AtomicU32 as *const u32,
                FUTEX_WAIT,
                expected as i32,
                timespec_ptr,
            )
        };

        if ret == 0 {
            return Ok(());
        }

        let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        match err {
            libc::EINTR => continue,
            libc::ETIMEDOUT => return Err(FutexError::Timeout),
            libc::EAGAIN => continue,
            libc::EINVAL | libc::EFAULT => return Err(FutexError::InvalidArgument),
            _ => return Err(FutexError::Other(err)),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn linux_futex_wake(value: &AtomicU32, count: usize) -> usize {
    use std::ptr;

    const FUTEX_WAKE: i32 = 1;

    let ret = unsafe {
        libc::syscall(
            libc::SYS_futex,
            value as *const AtomicU32 as *const u32,
            FUTEX_WAKE,
            count as i32,
            ptr::null::<libc::timespec>(),
        )
    };

    if ret < 0 {
        0
    } else {
        ret as usize
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn duration_to_timespec(dur: Duration) -> libc::timespec {
    libc::timespec {
        tv_sec: dur.as_secs() as libc::time_t,
        tv_nsec: dur.subsec_nanos() as libc::c_long,
    }
}

#[cfg(target_os = "macos")]
fn macos_futex_wait(
    value: &AtomicU32,
    expected: u32,
    timeout: Option<Duration>,
) -> FutexResult {
    if value.load(Ordering::SeqCst) != expected {
        return Err(FutexError::InvalidArgument);
    }

    const UL_WAIT: i32 = 0;
    const UL_NO_ERR: i32 = 1;

    let timeout_ns = timeout
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(i64::MAX);

    let ret = unsafe {
        libc::ulock_wait(
            UL_WAIT as u32,
            value as *const AtomicU32 as *mut libc::c_void,
            expected as u32,
            timeout_ns,
        )
    };

    match ret {
        0 => Ok(()),
        libc::ETIMEDOUT => Err(FutexError::Timeout),
        libc::EINTR => Err(FutexError::Interrupted),
        libc::EINVAL => Err(FutexError::InvalidArgument),
        _ if ret < 0 => Err(FutexError::Other(ret)),
        _ => Ok(()),
    }
}

#[cfg(target_os = "macos")]
fn macos_futex_wake(value: &AtomicU32, count: usize) -> usize {
    const UL_WAKE: i32 = 1;
    const UL_WAKE_ALL: i32 = 3;

    let op = if count == usize::MAX {
        UL_WAKE_ALL
    } else {
        UL_WAKE
    };

    let ret = unsafe {
        libc::ulock_wake(
            op as u32,
            value as *const AtomicU32 as *mut libc::c_void,
            0,
        )
    };

    if ret < 0 {
        0
    } else if count == usize::MAX {
        count
    } else {
        ret as usize
    }
}

#[cfg(target_os = "windows")]
fn windows_futex_wait(
    value: &AtomicU32,
    expected: u32,
    timeout: Option<Duration>,
) -> FutexResult {
    use windows::Win32::System::Threading::WaitOnAddress;

    if value.load(Ordering::SeqCst) != expected {
        return Err(FutexError::InvalidArgument);
    }

    let timeout_ms = timeout
        .map(|d| d.as_millis().clamp(1, u32::MAX as u128) as u32)
        .unwrap_or(0xFFFFFFFF);

    let result = unsafe {
        WaitOnAddress(
            value as *const AtomicU32 as *const u32 as *mut _,
            &expected as *const u32 as *mut _,
            std::mem::size_of::<u32>(),
            timeout_ms,
        )
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            const ERROR_TIMEOUT_HRESULT: u32 = 0x800705B4;
            if e.code().0 == ERROR_TIMEOUT_HRESULT as i32 {
                Err(FutexError::Timeout)
            } else {
                Err(FutexError::Other(e.code().0))
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_futex_wake(value: &AtomicU32, count: usize) -> usize {
    use windows::Win32::System::Threading::{
        WakeByAddressAll, WakeByAddressSingle,
    };

    if count == usize::MAX {
        unsafe {
            WakeByAddressAll(value as *const AtomicU32 as *mut _);
        }
        count
    } else {
        for _ in 0..count {
            unsafe {
                WakeByAddressSingle(value as *const AtomicU32 as *mut _);
            }
        }
        count
    }
}
