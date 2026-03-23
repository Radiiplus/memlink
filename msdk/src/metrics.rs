//! Metrics recording for memlink modules, exporting metrics to the daemon for monitoring.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(f64),
}

pub fn record_metric(_name: &str, _value: MetricValue) {}

pub fn increment_counter(name: &str, delta: u64) {
    record_metric(name, MetricValue::Counter(delta));
}

pub fn set_gauge(name: &str, value: f64) {
    record_metric(name, MetricValue::Gauge(value));
}

pub fn observe_histogram(name: &str, value: f64) {
    record_metric(name, MetricValue::Histogram(value));
}
