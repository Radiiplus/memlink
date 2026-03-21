//! Multi-priority ring buffer with three-tier message queuing.
//! Critical (20%), High (50%), Low (30%) slot distribution with FIFO ordering.

use crate::buffer::{RingBuffer, RingBufferError, SlotId};
use crate::priority::Priority;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct PriorityRingBuffer {
    critical: RingBuffer,
    high: RingBuffer,
    low: RingBuffer,
    critical_count: usize,
    high_count: usize,
    low_count: usize,
    critical_written: AtomicU64,
    critical_read: AtomicU64,
    high_written: AtomicU64,
    high_read: AtomicU64,
    low_written: AtomicU64,
    low_read: AtomicU64,
}

pub type PriorityResult<T> = Result<T, RingBufferError>;

impl PriorityRingBuffer {
    pub fn new(total_slots: usize) -> Result<Self, RingBufferError> {
        if total_slots < 3 {
            return Err(RingBufferError::Full);
        }

        let critical_count = ((total_slots as f64 * 0.2).ceil() as usize).max(1);
        let high_count = ((total_slots as f64 * 0.5).ceil() as usize).max(1);
        let low_count = (total_slots - critical_count - high_count).max(1);

        let critical = RingBuffer::new(critical_count.next_power_of_two())?;
        let high = RingBuffer::new(high_count.next_power_of_two())?;
        let low = RingBuffer::new(low_count.next_power_of_two())?;

        Ok(Self {
            critical,
            high,
            low,
            critical_count,
            high_count,
            low_count,
            critical_written: AtomicU64::new(0),
            critical_read: AtomicU64::new(0),
            high_written: AtomicU64::new(0),
            high_read: AtomicU64::new(0),
            low_written: AtomicU64::new(0),
            low_read: AtomicU64::new(0),
        })
    }

    pub fn total_slots(&self) -> usize {
        self.critical_count + self.high_count + self.low_count
    }

    pub fn slot_counts(&self) -> (usize, usize, usize) {
        (self.critical_count, self.high_count, self.low_count)
    }

    pub fn pending_counts(&self) -> (u64, u64, u64) {
        let crit = self.critical_written.load(Ordering::Acquire)
            .wrapping_sub(self.critical_read.load(Ordering::Acquire));
        let high = self.high_written.load(Ordering::Acquire)
            .wrapping_sub(self.high_read.load(Ordering::Acquire));
        let low = self.low_written.load(Ordering::Acquire)
            .wrapping_sub(self.low_read.load(Ordering::Acquire));
        (crit, high, low)
    }

    pub fn total_pending(&self) -> u64 {
        let (crit, high, low) = self.pending_counts();
        crit + high + low
    }

    pub fn is_empty(&self) -> bool {
        self.total_pending() == 0
    }

    pub fn write(&self, priority: Priority, data: &[u8]) -> PriorityResult<SlotId> {
        match priority {
            Priority::Critical => {
                let pending = self.critical_written.load(Ordering::Acquire)
                    .wrapping_sub(self.critical_read.load(Ordering::Acquire));
                if pending >= self.critical_count as u64 {
                    return Err(RingBufferError::Full);
                }
                self.critical.write_slot(crate::buffer::Priority::Normal, data)
                    .map(|_| {
                        self.critical_written.fetch_add(1, Ordering::Release);
                        SlotId { index: pending, seq: pending }
                    })
            }
            Priority::High => {
                let pending = self.high_written.load(Ordering::Acquire)
                    .wrapping_sub(self.high_read.load(Ordering::Acquire));
                if pending >= self.high_count as u64 {
                    return Err(RingBufferError::Full);
                }
                self.high.write_slot(crate::buffer::Priority::Normal, data)
                    .map(|_| {
                        self.high_written.fetch_add(1, Ordering::Release);
                        SlotId { index: pending, seq: pending }
                    })
            }
            Priority::Low => {
                let pending = self.low_written.load(Ordering::Acquire)
                    .wrapping_sub(self.low_read.load(Ordering::Acquire));
                if pending >= self.low_count as u64 {
                    return Err(RingBufferError::Full);
                }
                self.low.write_slot(crate::buffer::Priority::Normal, data)
                    .map(|_| {
                        self.low_written.fetch_add(1, Ordering::Release);
                        SlotId { index: pending, seq: pending }
                    })
            }
        }
    }

    pub fn read(&self) -> Option<(Priority, Vec<u8>)> {
        if let Some(data) = self.critical.read_slot() {
            self.critical_read.fetch_add(1, Ordering::Release);
            return Some((Priority::Critical, data.1));
        }

        if let Some(data) = self.high.read_slot() {
            self.high_read.fetch_add(1, Ordering::Release);
            return Some((Priority::High, data.1));
        }

        if let Some(data) = self.low.read_slot() {
            self.low_read.fetch_add(1, Ordering::Release);
            return Some((Priority::Low, data.1));
        }

        None
    }

    pub fn clear(&self) {
        self.critical_written.store(0, Ordering::Release);
        self.critical_read.store(0, Ordering::Release);
        self.high_written.store(0, Ordering::Release);
        self.high_read.store(0, Ordering::Release);
        self.low_written.store(0, Ordering::Release);
        self.low_read.store(0, Ordering::Release);
    }
}
