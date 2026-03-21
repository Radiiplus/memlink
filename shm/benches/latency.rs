use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use memlink_shm::buffer::{RingBuffer, Priority as BufferPriority};
use std::sync::Arc;
use std::thread;

fn bench_empty_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_empty");
    group.throughput(Throughput::Elements(1));

    group.bench_function("ring_buffer", |b| {
        b.iter(|| {
            let rb = RingBuffer::new(64).unwrap();
            rb.write_slot(BufferPriority::High, &[]).unwrap();
            let _ = rb.read_slot();
        });
    });

    group.finish();
}

fn bench_small_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_64b");
    group.throughput(Throughput::Elements(1));

    let payload = vec![0u8; 64];

    group.bench_function("ring_buffer", |b| {
        b.iter(|| {
            let rb = RingBuffer::new(64).unwrap();
            rb.write_slot(BufferPriority::High, &payload).unwrap();
            let _ = rb.read_slot();
        });
    });

    group.finish();
}

fn bench_medium_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_1kb");
    group.throughput(Throughput::Elements(1));

    let payload = vec![0u8; 1024];

    group.bench_function("ring_buffer", |b| {
        b.iter(|| {
            let rb = RingBuffer::new(64).unwrap();
            rb.write_slot(BufferPriority::High, &payload).unwrap();
            let _ = rb.read_slot();
        });
    });

    group.finish();
}

fn bench_large_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_4kb");
    group.throughput(Throughput::Elements(1));

    let payload = vec![0u8; 4096];

    group.bench_function("ring_buffer", |b| {
        b.iter(|| {
            let rb = RingBuffer::new(64).unwrap();
            rb.write_slot(BufferPriority::High, &payload).unwrap();
            let _ = rb.read_slot();
        });
    });

    group.finish();
}

fn bench_spsc_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_spsc");

    for size in [64, 256, 1024, 4096] {
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, size| {
            b.iter(|| {
                let payload = vec![0u8; *size];
                let rb = Arc::new(RingBuffer::new(256).unwrap());
                let rb_clone = Arc::clone(&rb);

                let producer = thread::spawn(move || {
                    for _ in 0..1000 {
                        while rb_clone.write_slot(BufferPriority::High, &payload).is_err() {
                            thread::yield_now();
                        }
                    }
                });

                let mut count = 0;
                while count < 1000 {
                    if rb.read_slot().is_some() {
                        count += 1;
                    }
                }

                producer.join().unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_empty_roundtrip,
    bench_small_roundtrip,
    bench_medium_roundtrip,
    bench_large_roundtrip,
    bench_spsc_throughput,
);

criterion_main!(benches);
