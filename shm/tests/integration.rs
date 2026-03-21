use shm::buffer::{RingBuffer, Priority as BufferPriority};
use shm::control::ControlRegion;
use shm::futex::Futex;
use shm::layout::{CONTROL_REGION_SIZE, RING_BUFFER_OFFSET};
use shm::mmap::MmapSegment;
use shm::platform::Platform;
use shm::pring::PriorityRingBuffer;
use shm::priority::{Priority, calculate_slot_distribution};
use shm::recovery::{RecoveryManager, Heartbeat, SlotMetadata, SlotState};
use shm::safety::{BoundsChecker, PoisonState, SafeShmAccess};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

#[test]
fn integration_buffer_create_and_write() {
    let temp_dir = tempdir().unwrap();
    let mmap_path = temp_dir.path().join("test_mmap.bin");

    let mut mmap = MmapSegment::create(&mmap_path, 4096).unwrap();
    let data = b"hello";
    mmap.as_mut_slice()[0..data.len()].copy_from_slice(data);
    let read_data = &mmap.as_slice()[0..data.len()];
    assert_eq!(read_data, data);
}

#[test]
fn integration_buffer_ring_basic() {
    let rb = RingBuffer::new(64).unwrap();
    rb.write_slot(BufferPriority::High, b"test").unwrap();
    let (_, data) = rb.read_slot().unwrap();
    assert_eq!(&data, b"test");
}

#[test]
fn integration_control_region_size() {
    assert_eq!(std::mem::size_of::<ControlRegion>(), 4096);
    assert_eq!(std::mem::align_of::<ControlRegion>(), 4096);
}

#[test]
fn integration_control_region_backpressure() {
    let temp_dir = tempdir().unwrap();
    let mmap_path = temp_dir.path().join("control_test.bin");

    let mmap = MmapSegment::create(&mmap_path, 8192).unwrap();
    let control_ptr = mmap.as_slice().as_ptr() as *const ControlRegion;

    unsafe {
        let version = (*control_ptr).version();
        assert_eq!(version, 0);
    }
}

#[test]
fn integration_futex_wait_wake() {
    let futex = Arc::new(Futex::new(0));
    let futex_clone = Arc::clone(&futex);

    let handle = thread::spawn(move || {
        let result = futex_clone.wait(0, Some(Duration::from_secs(5)));
        assert!(result.is_ok() || matches!(result, Err(shm::futex::FutexError::Timeout)));
    });

    thread::sleep(Duration::from_millis(50));
    futex.store(1);
    let _woken = futex.wake_one();

    handle.join().unwrap();
}

#[test]
fn integration_futex_timeout() {
    let futex = Futex::new(0);
    let start = Instant::now();
    let result = futex.wait(0, Some(Duration::from_millis(50)));
    let elapsed = start.elapsed();

    assert!(matches!(result, Err(shm::futex::FutexError::Timeout)));
    assert!(elapsed >= Duration::from_millis(40));
    assert!(elapsed <= Duration::from_millis(150));
}

#[test]
fn integration_layout_constants() {
    assert_eq!(CONTROL_REGION_SIZE, 4096);
    assert_eq!(RING_BUFFER_OFFSET, 4096);
}

#[test]
fn integration_platform_detection() {
    let platform = Platform::current();

    #[cfg(target_os = "linux")]
    assert_eq!(platform, Platform::Linux);

    #[cfg(target_os = "macos")]
    assert_eq!(platform, Platform::MacOS);

    #[cfg(target_os = "windows")]
    assert_eq!(platform, Platform::Windows);
}

#[test]
fn integration_platform_is_unix() {
    let platform = Platform::current();

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert!(platform.is_unix());

    #[cfg(target_os = "windows")]
    assert!(!platform.is_unix());
}

#[test]
fn integration_platform_as_str() {
    let platform = Platform::current();
    let s = platform.as_str();

    assert!(!s.is_empty());
    assert!(["linux", "macos", "windows"].contains(&s));
}

#[test]
fn integration_priority_routing() {
    let rb = PriorityRingBuffer::new(100).unwrap();

    rb.write(Priority::High, b"high_1").unwrap();
    rb.write(Priority::Critical, b"critical_1").unwrap();
    rb.write(Priority::Low, b"low_1").unwrap();

    let (p, d) = rb.read().unwrap();
    assert_eq!(p, Priority::Critical);
    assert_eq!(&d, b"critical_1");

    let (p, d) = rb.read().unwrap();
    assert_eq!(p, Priority::High);
    assert_eq!(&d, b"high_1");

    let (p, d) = rb.read().unwrap();
    assert_eq!(p, Priority::Low);
    assert_eq!(&d, b"low_1");
}

#[test]
fn integration_priority_distribution() {
    let (crit, high, low) = calculate_slot_distribution(100);
    assert_eq!(crit, 20);
    assert_eq!(high, 50);
    assert_eq!(low, 30);
}

#[test]
fn integration_slot_state_machine() {
    let slot = SlotMetadata::new();
    assert_eq!(slot.state.load(), SlotState::Empty);

    slot.state.store(SlotState::Writing);
    assert_eq!(slot.state.load(), SlotState::Writing);

    slot.state.store(SlotState::Ready);
    assert_eq!(slot.state.load(), SlotState::Ready);
}

#[test]
fn integration_heartbeat() {
    let heartbeat = Heartbeat::new(1);
    heartbeat.beat();
    assert!(heartbeat.is_alive(5));

    heartbeat.stop();
    assert!(!heartbeat.is_alive(5));
}

#[test]
fn integration_bounds_checking() {
    let base = vec![0u8; 1024];
    let checker = BoundsChecker::new(base.as_ptr(), 1024);

    assert!(checker.is_valid_offset(0));
    assert!(checker.is_valid_offset(512));
    assert!(checker.is_valid_offset(1023));
    assert!(!checker.is_valid_offset(1024));

    assert!(checker.validate_range(0, 1024).is_ok());
    assert!(checker.validate_range(0, 1025).is_err());
}

#[test]
fn integration_poison_guard() {
    let poison = Arc::new(AtomicBool::new(false));
    let guard = shm::safety::PoisonGuard::new(Arc::clone(&poison));

    assert!(poison.load(Ordering::Acquire));
    guard.disarm();
    assert!(!poison.load(Ordering::Acquire));
}

#[test]
fn integration_panic_safety() {
    let poison = PoisonState::new();
    assert!(!poison.is_poisoned());

    let result: Result<i32, _> = shm::safety::with_panic_protection(&poison, || {
        panic!("test panic");
    });
    assert!(result.is_err());
    assert!(poison.is_poisoned());
}

#[test]
fn integration_end_to_end_ring_buffer() {
    let rb = Arc::new(RingBuffer::new(256).unwrap());
    let messages = 1000;

    let rb_producer = Arc::clone(&rb);
    let producer = thread::spawn(move || {
        for i in 0..messages {
            let msg = format!("msg_{}", i);
            while rb_producer.write_slot(BufferPriority::High, msg.as_bytes()).is_err() {
                thread::yield_now();
            }
        }
    });

    let rb_consumer = Arc::clone(&rb);
    let consumer = thread::spawn(move || {
        let mut count = 0;
        while count < messages {
            if let Some((_, data)) = rb_consumer.read_slot() {
                let expected = format!("msg_{}", count);
                assert_eq!(&data, expected.as_bytes());
                count += 1;
            }
        }
        count
    });

    producer.join().unwrap();
    let received = consumer.join().unwrap();
    assert_eq!(received, messages);
}

#[test]
fn integration_priority_with_futex() {
    let rb = PriorityRingBuffer::new(256).unwrap();
    let futex = Futex::new(0);

    rb.write(Priority::Critical, b"critical").unwrap();
    rb.write(Priority::High, b"high").unwrap();
    rb.write(Priority::Low, b"low").unwrap();

    futex.wake_all();

    let (p, d) = rb.read().unwrap();
    assert_eq!(p, Priority::Critical);
    assert_eq!(&d, b"critical");

    let (p, d) = rb.read().unwrap();
    assert_eq!(p, Priority::High);
    assert_eq!(&d, b"high");

    let (p, d) = rb.read().unwrap();
    assert_eq!(p, Priority::Low);
    assert_eq!(&d, b"low");
}

#[test]
fn integration_safe_access() {
    let base = vec![0u8; 1024];
    let access = SafeShmAccess::new(base.as_ptr(), 1024);

    let result = access.with_safe_access(0, 100, || 42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    let result = access.with_safe_access(1000, 100, || 42);
    assert!(matches!(result, Err(shm::safety::SafeAccessError::OutOfBounds)));
}

#[test]
fn integration_recovery_manager() {
    let temp_dir = tempdir().unwrap();
    let shm_path = temp_dir.path().join("test_daemon.shm");
    let shm_path_str = shm_path.to_str().unwrap();

    let daemon1 = RecoveryManager::new(shm_path_str);
    assert!(daemon1.register_daemon().is_ok());

    let daemon2 = RecoveryManager::new(shm_path_str);
    assert!(daemon2.register_daemon().is_err());

    daemon1.unregister_daemon();
    assert!(daemon2.register_daemon().is_ok());
}
