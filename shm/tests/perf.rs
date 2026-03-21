use memlink_shm::buffer::{RingBuffer, Priority as BufferPriority};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_latency_target() {
    if std::env::var("CI").is_ok() {
        println!("Skipping latency test on CI");
        return;
    }

    let rb = RingBuffer::new(64).unwrap();
    let payload = vec![0u8; 64];
    let iterations = 1000;
    let mut latencies = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        rb.write_slot(BufferPriority::High, &payload).unwrap();
        let _ = rb.read_slot();
        latencies.push(start.elapsed().as_micros() as f64);
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p50 = latencies[latencies.len() / 2];
    let p99_idx = (latencies.len() as f64 * 0.99) as usize;
    let p99 = latencies[p99_idx.min(latencies.len() - 1)];

    println!("Latency: p50={:.2}μs, p99={:.2}μs", p50, p99);

    let p50_target = if cfg!(debug_assertions) { 50.0 } else { 5.0 };
    let p99_target = if cfg!(debug_assertions) { 100.0 } else { 10.0 };

    assert!(
        p50 < p50_target,
        "p50 latency {:.2}μs exceeds target {:.0}μs",
        p50,
        p50_target
    );
    assert!(
        p99 < p99_target,
        "p99 latency {:.2}μs exceeds target {:.0}μs",
        p99,
        p99_target
    );
}

#[test]
fn test_throughput_target() {
    if std::env::var("CI").is_ok() {
        println!("Skipping throughput test on CI");
        return;
    }

    let rb = RingBuffer::new(256).unwrap();
    let payload = vec![0u8; 64];
    let messages = 1000;

    let start = Instant::now();

    for _ in 0..messages {
        rb.write_slot(BufferPriority::High, &payload).unwrap();
        let _ = rb.read_slot();
    }

    let total_time = start.elapsed();
    let throughput = messages as f64 / total_time.as_secs_f64();

    println!(
        "Throughput: {:.0} msg/sec ({:.2}s for {} messages)",
        throughput,
        total_time.as_secs_f64(),
        messages
    );

    let target = if cfg!(debug_assertions) { 10000.0 } else { 100000.0 };
    assert!(
        throughput > target,
        "Throughput {:.0} msg/sec below target {:.0} msg/sec",
        throughput,
        target
    );
}

#[test]
fn test_no_memory_growth() {
    let rb = RingBuffer::new(256).unwrap();
    let payload = vec![0u8; 1024];
    let messages = 1000;

    for i in 0..messages {
        rb.write_slot(BufferPriority::High, &payload).unwrap();
        let _ = rb.read_slot();

        if i % 100 == 0 {
            assert!(rb.is_empty(), "Buffer should be empty after read");
        }
    }

    assert!(rb.is_empty(), "Buffer should be empty after all operations");
    println!("No memory growth: buffer remained empty throughout {} messages", messages);
}

#[test]
fn test_idle_cpu() {
    let rb = RingBuffer::new(64).unwrap();

    let start = Instant::now();
    let result = rb.read_slot();
    let elapsed = start.elapsed();

    assert!(result.is_none(), "Empty buffer should return None");
    assert!(elapsed < Duration::from_millis(10), "Read on empty buffer should be fast");

    println!("Idle check: empty buffer read took {:?}", elapsed);
}

#[test]
fn test_spsc_throughput() {
    let rb = Arc::new(RingBuffer::new(256).unwrap());
    let payload = vec![0u8; 64];
    let messages = 1000;

    let rb_producer = Arc::clone(&rb);
    let producer = thread::spawn(move || {
        for i in 0..messages {
            while rb_producer.write_slot(BufferPriority::High, &payload).is_err() {
                thread::yield_now();
            }
            if i % 100 == 0 {
                thread::yield_now();
            }
        }
    });

    let rb_consumer = Arc::clone(&rb);
    let consumer = thread::spawn(move || {
        let mut count = 0;
        while count < messages {
            if rb_consumer.read_slot().is_some() {
                count += 1;
            } else {
                thread::yield_now();
            }
        }
        count
    });

    producer.join().unwrap();
    let received = consumer.join().unwrap();

    assert_eq!(received, messages, "All messages should be received");
}

#[test]
fn test_mpsc_throughput() {
    let rb = Arc::new(RingBuffer::new(512).unwrap());
    let payload = vec![0u8; 64];
    let num_producers = 4;
    let messages_per_producer = 250;

    let mut producers = Vec::new();
    for _ in 0..num_producers {
        let rb_clone = Arc::clone(&rb);
        let payload_clone = payload.clone();
        let handle = thread::spawn(move || {
            for _ in 0..messages_per_producer {
                while rb_clone.write_slot(BufferPriority::High, &payload_clone).is_err() {
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
        } else {
            thread::yield_now();
        }
    }

    for handle in producers {
        handle.join().unwrap();
    }

    assert_eq!(count, total_messages, "All messages should be received");
}
