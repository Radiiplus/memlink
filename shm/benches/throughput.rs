use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use memlink_shm::buffer::{RingBuffer, Priority as BufferPriority};
use std::sync::Arc;
use std::thread;

fn bench_spsc_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_throughput");

    for size in [64, 256, 1024] {
        let payload = vec![0u8; size];
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}b", size)),
            &payload,
            |b, _payload| {
                b.iter(|| {
                    let rb = RingBuffer::new(256).unwrap();
                    let messages = 10000;

                    for i in 0..messages {
                        let msg = format!("msg_{}", i);
                        while rb.write_slot(BufferPriority::High, msg.as_bytes()).is_err() {
                            thread::yield_now();
                        }
                    }

                    let mut count = 0;
                    while count < messages {
                        if rb.read_slot().is_some() {
                            count += 1;
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_mpsc_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("mpsc_throughput");

    for num_producers in [2, 4, 8] {
        for size in [64, 256] {
            let payload = vec![0u8; size];
            group.throughput(Throughput::Bytes((size * num_producers) as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{}p_{}b", num_producers, size)),
                &(num_producers, payload),
                |b, &(num_producers, ref payload)| {
                    b.iter(|| {
                        let rb = Arc::new(RingBuffer::new(512).unwrap());
                        let messages_per_producer = 5000;

                        let mut producers = Vec::new();
                        for _ in 0..num_producers {
                            let rb_clone = Arc::clone(&rb);
                            let payload_clone = payload.clone();
                            let handle = thread::spawn(move || {
                                for _ in 0..messages_per_producer {
                                    while rb_clone
                                        .write_slot(BufferPriority::High, &payload_clone)
                                        .is_err()
                                    {
                                        thread::yield_now();
                                    }
                                }
                            });
                            producers.push(handle);
                        }

                        let total_messages = messages_per_producer * num_producers;
                        let mut count = 0;
                        while count < total_messages {
                            if rb.read_slot().is_some() {
                                count += 1;
                            }
                        }

                        for handle in producers {
                            handle.join().unwrap();
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_priority_throughput(c: &mut Criterion) {
    use memlink_shm::priority::Priority;
    use memlink_shm::pring::PriorityRingBuffer;

    let mut group = c.benchmark_group("priority_throughput");

    for size in [64, 256] {
        let payload = vec![0u8; size];
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}b", size)),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let rb = PriorityRingBuffer::new(256).unwrap();
                    let messages = 5000;

                    for i in 0..messages {
                        let priority = match i % 3 {
                            0 => Priority::Critical,
                            1 => Priority::High,
                            _ => Priority::Low,
                        };
                        while rb.write(priority, payload).is_err() {
                            thread::yield_now();
                        }
                    }

                    let mut count = 0;
                    while count < messages {
                        if rb.read().is_some() {
                            count += 1;
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_spsc_throughput,
    bench_mpsc_throughput,
    bench_priority_throughput,
);

criterion_main!(benches);
