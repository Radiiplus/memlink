//! Runtime metrics collection with Prometheus-compatible output.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            value: AtomicU64::new(0),
        }
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_by(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct Histogram {
    count: AtomicU64,
    sum: AtomicU64,
    min: AtomicU64,
    max: AtomicU64,
    buckets: &'static [u64],
    bucket_counts: Box<[AtomicU64]>,
}

impl Histogram {
    pub fn new() -> Self {
        const DEFAULT_BUCKETS: &[u64] = &[
            10, 50, 100, 500, 1_000, 5_000, 10_000, 50_000, 100_000, 500_000, 1_000_000,
        ];
        Self::with_buckets(DEFAULT_BUCKETS)
    }

    pub fn with_buckets(buckets: &'static [u64]) -> Self {
        let bucket_counts: Box<[AtomicU64]> = buckets
            .iter()
            .map(|_| AtomicU64::new(0))
            .collect();

        Histogram {
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
            buckets,
            bucket_counts,
        }
    }

    pub fn observe(&self, duration: Duration) {
        let micros = duration.as_micros() as u64;

        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(micros, Ordering::Relaxed);

        let mut current_min = self.min.load(Ordering::Relaxed);
        while micros < current_min {
            match self.min.compare_exchange_weak(
                current_min,
                micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        let mut current_max = self.max.load(Ordering::Relaxed);
        while micros > current_max {
            match self.max.compare_exchange_weak(
                current_max,
                micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        for (i, &bucket) in self.buckets.iter().enumerate() {
            if micros <= bucket {
                self.bucket_counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }

    pub fn avg(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            self.sum.load(Ordering::Relaxed) as f64 / count as f64
        }
    }

    pub fn min(&self) -> u64 {
        let min = self.min.load(Ordering::Relaxed);
        if min == u64::MAX { 0 } else { min }
    }

    pub fn max(&self) -> u64 {
        self.max.load(Ordering::Relaxed)
    }

    pub fn bucket_counts(&self) -> Vec<u64> {
        self.bucket_counts
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect()
    }

    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.sum.store(0, Ordering::Relaxed);
        self.min.store(u64::MAX, Ordering::Relaxed);
        self.max.store(0, Ordering::Relaxed);
        for bucket in self.bucket_counts.iter() {
            bucket.store(0, Ordering::Relaxed);
        }
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RuntimeMetrics {
    pub loads: Counter,
    pub unloads: Counter,
    pub calls: Counter,
    pub panics: Counter,
    pub reloads: Counter,
    pub timeouts: Counter,
    pub load_time: Histogram,
    pub call_latency: Histogram,
    pub modules_loaded: AtomicUsize,
    pub calls_in_flight: AtomicUsize,
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        RuntimeMetrics {
            loads: Counter::new(),
            unloads: Counter::new(),
            calls: Counter::new(),
            panics: Counter::new(),
            reloads: Counter::new(),
            timeouts: Counter::new(),
            load_time: Histogram::new(),
            call_latency: Histogram::new(),
            modules_loaded: AtomicUsize::new(0),
            calls_in_flight: AtomicUsize::new(0),
        }
    }

    pub fn record_load(&self, duration: Duration) {
        self.loads.inc();
        self.modules_loaded.fetch_add(1, Ordering::Relaxed);
        self.load_time.observe(duration);
    }

    pub fn record_unload(&self) {
        self.unloads.inc();
        self.modules_loaded.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_call(&self, duration: Duration) {
        self.calls.inc();
        self.calls_in_flight.fetch_sub(1, Ordering::Relaxed);
        self.call_latency.observe(duration);
    }

    pub fn record_call_start(&self) {
        self.calls_in_flight.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_panic(&self) {
        self.panics.inc();
    }

    pub fn record_reload(&self) {
        self.reloads.inc();
    }

    pub fn record_timeout(&self) {
        self.timeouts.inc();
    }

    pub fn prometheus_export(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "# HELP memlink_loads_total Total number of module loads\n\
             # TYPE memlink_loads_total counter\n\
             memlink_loads_total {}\n",
            self.loads.get()
        ));

        output.push_str(&format!(
            "# HELP memlink_unloads_total Total number of module unloads\n\
             # TYPE memlink_unloads_total counter\n\
             memlink_unloads_total {}\n",
            self.unloads.get()
        ));

        output.push_str(&format!(
            "# HELP memlink_calls_total Total number of module calls\n\
             # TYPE memlink_calls_total counter\n\
             memlink_calls_total {}\n",
            self.calls.get()
        ));

        output.push_str(&format!(
            "# HELP memlink_panics_total Total number of panics caught\n\
             # TYPE memlink_panics_total counter\n\
             memlink_panics_total {}\n",
            self.panics.get()
        ));

        output.push_str(&format!(
            "# HELP memlink_reloads_total Total number of reload operations\n\
             # TYPE memlink_reloads_total counter\n\
             memlink_reloads_total {}\n",
            self.reloads.get()
        ));

        output.push_str(&format!(
            "# HELP memlink_timeouts_total Total number of call timeouts\n\
             # TYPE memlink_timeouts_total counter\n\
             memlink_timeouts_total {}\n",
            self.timeouts.get()
        ));

        output.push_str(&format!(
            "# HELP memlink_modules_loaded Current number of loaded modules\n\
             # TYPE memlink_modules_loaded gauge\n\
             memlink_modules_loaded {}\n",
            self.modules_loaded.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# HELP memlink_calls_in_flight Current number of in-flight calls\n\
             # TYPE memlink_calls_in_flight gauge\n\
             memlink_calls_in_flight {}\n",
            self.calls_in_flight.load(Ordering::Relaxed)
        ));

        output.push_str(
            "# HELP memlink_load_time_us Module load time in microseconds\n\
             # TYPE memlink_load_time_us histogram\n"
        );
        let mut cumulative = 0u64;
        for (i, &bucket) in self.load_time.buckets.iter().enumerate() {
            cumulative += self.load_time.bucket_counts()[i];
            output.push_str(&format!(
                "memlink_load_time_us_bucket{{le=\"{}\"}} {}\n",
                bucket, cumulative
            ));
        }
        output.push_str(&format!(
            "memlink_load_time_us_bucket{{le=\"+Inf\"}} {}\n\
             memlink_load_time_us_sum {}\n\
             memlink_load_time_us_count {}\n",
            self.load_time.count(),
            self.load_time.sum(),
            self.load_time.count()
        ));

        output.push_str(
            "# HELP memlink_call_latency_us Module call latency in microseconds\n\
             # TYPE memlink_call_latency_us histogram\n"
        );
        cumulative = 0;
        for (i, &bucket) in self.call_latency.buckets.iter().enumerate() {
            cumulative += self.call_latency.bucket_counts()[i];
            output.push_str(&format!(
                "memlink_call_latency_us_bucket{{le=\"{}\"}} {}\n",
                bucket, cumulative
            ));
        }
        output.push_str(&format!(
            "memlink_call_latency_us_bucket{{le=\"+Inf\"}} {}\n\
             memlink_call_latency_us_sum {}\n\
             memlink_call_latency_us_count {}\n",
            self.call_latency.count(),
            self.call_latency.sum(),
            self.call_latency.count()
        ));

        output
    }

    pub fn reset(&self) {
        self.loads.reset();
        self.unloads.reset();
        self.calls.reset();
        self.panics.reset();
        self.reloads.reset();
        self.timeouts.reset();
        self.load_time.reset();
        self.call_latency.reset();
        self.modules_loaded.store(0, Ordering::Relaxed);
        self.calls_in_flight.store(0, Ordering::Relaxed);
    }
}

impl Default for RuntimeMetrics {
    fn default() -> Self {
        Self::new()
    }
}
