//! Three-tier priority system for message classification and routing.
//! Critical (20%), High (50%), Low (30%) slot distribution.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Priority {
    Critical = 0,
    High = 1,
    #[default]
    Low = 2,
}

impl Priority {
    pub fn slot_range(&self, total_slots: usize) -> (usize, usize) {
        match self {
            Priority::Critical => {
                let count = ((total_slots as f64 * 0.2).ceil() as usize).max(1);
                (0, count)
            }
            Priority::High => {
                let crit_count = ((total_slots as f64 * 0.2).ceil() as usize).max(1);
                let high_end = ((total_slots as f64 * 0.7).floor() as usize).max(crit_count);
                let count = (high_end - crit_count).max(1);
                (crit_count, count)
            }
            Priority::Low => {
                let crit_count = ((total_slots as f64 * 0.2).ceil() as usize).max(1);
                let high_end = ((total_slots as f64 * 0.7).floor() as usize).max(crit_count);
                let start = high_end;
                let count = (total_slots - start).max(1);
                (start, count)
            }
        }
    }

    pub fn all_in_order() -> &'static [Priority] {
        &[Priority::Critical, Priority::High, Priority::Low]
    }

    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Priority::Critical),
            1 => Some(Priority::High),
            2 => Some(Priority::Low),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

pub fn calculate_slot_distribution(total_slots: usize) -> (usize, usize, usize) {
    let (_crit_start, crit_count) = Priority::Critical.slot_range(total_slots);
    let (_high_start, high_count) = Priority::High.slot_range(total_slots);
    let (_low_start, low_count) = Priority::Low.slot_range(total_slots);
    (crit_count, high_count, low_count)
}
