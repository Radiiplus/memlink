//! Benchmarks for the memlink runtime.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use memlink_runtime::mhash::fnv1a_hash;
use memlink_runtime::arena::Arena;
use memlink_runtime::metrics::{Counter, Histogram, RuntimeMetrics};
use memlink_runtime::safety::{StackDepth, MemoryTracker};

fn bench_fnv1a_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("fnv1a_hash");

    group.bench_function("short_string", |b| {
        b.iter(|| fnv1a_hash(std::hint::black_box("test")))
    });

    group.bench_function("medium_string", |b| {
        b.iter(|| fnv1a_hash(std::hint::black_box("this_is_a_medium_length_string_for_benchmarking")))
    });

    group.bench_function("long_string", |b| {
        b.iter(|| {
            fnv1a_hash(std::hint::black_box(
                "this_is_a_very_long_string_that_we_are_using_to_benchmark_the_fnv1a_hash_function_performance_characteristics"
            ))
        })
    });

    group.finish();
}

fn bench_arena_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena");

    group.bench_function("alloc_small", |b| {
        let arena = Arena::new(1024 * 1024);
        b.iter(|| {
            arena.alloc(std::hint::black_box(64))
        })
    });

    group.bench_function("alloc_medium", |b| {
        let arena = Arena::new(1024 * 1024);
        b.iter(|| {
            arena.alloc(std::hint::black_box(1024))
        })
    });

    group.bench_function("alloc_large", |b| {
        let arena = Arena::new(1024 * 1024);
        b.iter(|| {
            arena.alloc(std::hint::black_box(65536))
        })
    });

    group.finish();
}

fn bench_arena_reset(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_reset");

    group.bench_function("reset_after_allocs", |b| {
        let arena = Arena::new(1024 * 1024);
        b.iter(|| {
            for _ in 0..100 {
                arena.alloc(1024);
            }
            arena.reset();
        })
    });

    group.finish();
}

fn bench_counter(c: &mut Criterion) {
    let mut group = c.benchmark_group("counter");

    group.bench_function("inc", |b| {
        let counter = Counter::new();
        b.iter(|| {
            counter.inc()
        })
    });

    group.bench_function("inc_by", |b| {
        let counter = Counter::new();
        b.iter(|| {
            counter.inc_by(std::hint::black_box(10))
        })
    });

    group.bench_function("get", |b| {
        let counter = Counter::new();
        counter.inc_by(100);
        b.iter(|| {
            counter.get()
        })
    });

    group.finish();
}

fn bench_histogram(c: &mut Criterion) {
    let mut group = c.benchmark_group("histogram");

    group.bench_function("observe", |b| {
        let histogram = Histogram::new();
        b.iter(|| {
            histogram.observe(std::time::Duration::from_micros(100))
        })
    });

    group.bench_function("avg", |b| {
        let histogram = Histogram::new();
        for i in 0..100 {
            histogram.observe(std::time::Duration::from_micros(i));
        }
        b.iter(|| {
            histogram.avg()
        })
    });

    group.finish();
}

fn bench_runtime_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime_metrics");

    group.bench_function("record_load", |b| {
        let metrics = RuntimeMetrics::new();
        b.iter(|| {
            metrics.record_load(std::time::Duration::from_micros(500))
        })
    });

    group.bench_function("record_call", |b| {
        let metrics = RuntimeMetrics::new();
        metrics.record_call_start();
        b.iter(|| {
            metrics.record_call(std::time::Duration::from_micros(50))
        })
    });

    group.bench_function("prometheus_export", |b| {
        let metrics = RuntimeMetrics::new();
        metrics.record_load(std::time::Duration::from_micros(500));
        metrics.record_call_start();
        metrics.record_call(std::time::Duration::from_micros(50));
        metrics.record_panic();
        b.iter(|| {
            metrics.prometheus_export()
        })
    });

    group.finish();
}

fn bench_stack_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_depth");

    group.bench_function("enter_exit", |b| {
        let depth = StackDepth::new();
        b.iter(|| {
            depth.enter(100).unwrap();
            depth.exit();
        })
    });

    group.bench_function("current", |b| {
        let depth = StackDepth::new();
        depth.enter(100).unwrap();
        b.iter(|| {
            depth.current()
        })
    });

    group.finish();
}

fn bench_memory_tracker(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_tracker");

    group.bench_function("allocate", |b| {
        let tracker = MemoryTracker::new(1024 * 1024);
        b.iter(|| {
            tracker.allocate(std::hint::black_box(1024))
        })
    });

    group.bench_function("free", |b| {
        let tracker = MemoryTracker::new(1024 * 1024);
        tracker.allocate(1024 * 100).unwrap();
        b.iter(|| {
            tracker.free(std::hint::black_box(1024))
        })
    });

    group.bench_function("usage_ratio", |b| {
        let tracker = MemoryTracker::new(1024 * 1024);
        tracker.allocate(512 * 1024).unwrap();
        b.iter(|| {
            tracker.usage_ratio()
        })
    });

    group.finish();
}

fn bench_load_unload(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_unload");

    group.bench_function("metrics_only", |b| {
        let metrics = RuntimeMetrics::new();
        b.iter(|| {
            metrics.record_load(std::time::Duration::from_micros(100));
            metrics.record_unload();
        })
    });

    group.finish();
}

fn bench_call_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("call_latency");

    group.bench_function("empty_call", |b| {
        let metrics = RuntimeMetrics::new();
        b.iter(|| {
            metrics.record_call_start();
            metrics.record_call(std::time::Duration::from_nanos(100));
        })
    });

    group.bench_function("small_payload", |b| {
        let metrics = RuntimeMetrics::new();
        let payload = vec![0u8; 64];
        b.iter(|| {
            metrics.record_call_start();
            std::hint::black_box(&payload);
            metrics.record_call(std::time::Duration::from_micros(5));
        })
    });

    group.finish();
}

fn bench_concurrent_calls(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let mut group = c.benchmark_group("concurrent_calls");

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_threads),
            num_threads,
            |b, &num_threads| {
                let metrics = Arc::new(RuntimeMetrics::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for _ in 0..num_threads {
                        let metrics_clone = Arc::clone(&metrics);
                        let handle = thread::spawn(move || {
                            for _ in 0..100 {
                                metrics_clone.record_call_start();
                                metrics_clone.record_call(std::time::Duration::from_micros(1));
                            }
                        });
                        handles.push(handle);
                    }
                    for handle in handles {
                        handle.join().unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_fnv1a_hash,
    bench_arena_allocation,
    bench_arena_reset,
    bench_counter,
    bench_histogram,
    bench_runtime_metrics,
    bench_stack_depth,
    bench_memory_tracker,
    bench_load_unload,
    bench_call_latency,
    bench_concurrent_calls,
);

criterion_main!(benches);
