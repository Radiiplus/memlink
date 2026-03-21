use memlink_shm::buffer::{RingBuffer, Priority as BufferPriority};
use memlink_shm::futex::Futex;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const PING_COUNT: u64 = 10000;
const MESSAGE_SIZE: usize = 64;

struct SharedState {
    request_buffer: RingBuffer,
    response_buffer: RingBuffer,
    request_futex: Futex,
    response_futex: Futex,
    ping_count: AtomicU64,
    daemon_running: AtomicBool,
}

impl SharedState {
    fn new() -> Self {
        Self {
            request_buffer: RingBuffer::new(64).unwrap(),
            response_buffer: RingBuffer::new(64).unwrap(),
            request_futex: Futex::new(0),
            response_futex: Futex::new(0),
            ping_count: AtomicU64::new(0),
            daemon_running: AtomicBool::new(true),
        }
    }
}

fn run_daemon() {
    println!("Starting daemon...");
    let state = Arc::new(SharedState::new());
    let state_clone = Arc::clone(&state);

    ctrlc_handler(move || {
        println!("\nDaemon shutting down...");
        state_clone.daemon_running.store(false, Ordering::Release);
    });

    let mut pings_handled = 0u64;
    let start = Instant::now();

    println!("Daemon ready, waiting for pings...");

    while state.daemon_running.load(Ordering::Acquire) {
        if let Some((_, data)) = state.request_buffer.read_slot() {
            let ping_num = state.ping_count.load(Ordering::Acquire);

            let response = format!("pong_{}", String::from_utf8_lossy(&data));
            state
                .response_buffer
                .write_slot(BufferPriority::High, response.as_bytes())
                .unwrap();
            state.response_futex.wake_one();

            pings_handled += 1;
            if pings_handled % 1000 == 0 {
                let elapsed = start.elapsed();
                let throughput = pings_handled as f64 / elapsed.as_secs_f64();
                println!(
                    "Daemon: handled {} pings ({:.0} msg/sec)",
                    pings_handled, throughput
                );
            }

            if ping_num >= PING_COUNT {
                println!("Daemon: received {} pings, shutting down", ping_num);
                break;
            }
        } else {
            let _ = state.request_futex.wait(0, Some(Duration::from_millis(100)));
        }
    }

    let elapsed = start.elapsed();
    println!(
        "Daemon: total {} pings in {:.2}s ({:.0} msg/sec)",
        pings_handled,
        elapsed.as_secs_f64(),
        pings_handled as f64 / elapsed.as_secs_f64()
    );
}

fn run_client() {
    println!("Starting client...");
    let state = SharedState::new();

    let message = "ping".as_bytes().to_vec();
    let mut pings_sent = 0u64;
    let mut pongs_received = 0u64;
    let start = Instant::now();

    println!("Client: sending {} pings...", PING_COUNT);

    while pings_sent < PING_COUNT {
        state.ping_count.fetch_add(1, Ordering::AcqRel);
        state
            .request_buffer
            .write_slot(BufferPriority::High, &message)
            .unwrap();
        state.request_futex.wake_one();
        pings_sent += 1;

        let result = state.response_futex.wait(0, Some(Duration::from_secs(1)));
        if result.is_ok()
            && state.response_buffer.read_slot().is_some() {
            pongs_received += 1;
            if pongs_received % 1000 == 0 {
                let elapsed = start.elapsed();
                let throughput = pongs_received as f64 / elapsed.as_secs_f64();
                println!(
                    "Client: received {} pongs ({:.0} msg/sec)",
                    pongs_received, throughput
                );
            }
        }
    }

    while pongs_received < pings_sent {
        if state.response_buffer.read_slot().is_some() {
            pongs_received += 1;
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }

    let elapsed = start.elapsed();
    println!(
        "\nClient: sent {} pings, received {} pongs in {:.2}s",
        pings_sent,
        pongs_received,
        elapsed.as_secs_f64()
    );
    println!(
        "Throughput: {:.0} msg/sec, {:.0} bytes/sec",
        pongs_received as f64 / elapsed.as_secs_f64(),
        (pongs_received * MESSAGE_SIZE as u64) as f64 / elapsed.as_secs_f64()
    );

    let avg_latency = elapsed.as_micros() as f64 / (pongs_received * 2) as f64;
    println!("Average round-trip latency: {:.2} μs", avg_latency);
}

fn ctrlc_handler<F: Fn() + Send + Sync + 'static>(handler: F) {
    let _handler = Arc::new(handler);
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3600));
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ping_pong <daemon|client>");
        eprintln!("  daemon - Run as server (handles pings)");
        eprintln!("  client - Run as client (sends pings)");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "daemon" => run_daemon(),
        "client" => run_client(),
        _ => {
            eprintln!("Unknown mode: {}. Use 'daemon' or 'client'.", args[1]);
            std::process::exit(1);
        }
    }
}
