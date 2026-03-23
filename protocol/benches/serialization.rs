//! Protocol serialization benchmarks.
//!
//! Benchmarks for MessagePack serialization/deserialization performance.

 use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use memlink_protocol::{ErrorMessage, Request, Response};
use memlink_protocol::msgpack::MessagePackSerializer;
use memlink_protocol::serializer::Serializer;

fn bench_request_serialization(c: &mut Criterion) {
    let request = Request::new(
        1,
        memlink_protocol::Priority::Normal,
        "test_module",
        "echo",
        vec![1, 2, 3, 4, 5],
    );

    let mut group = c.benchmark_group("request_serialization");
    group.throughput(Throughput::Bytes(5));
    group.bench_function("serialize", |b| {
        b.iter(|| {
            let bytes = MessagePackSerializer.serialize_request(black_box(&request)).unwrap();
            black_box(bytes)
        })
    });
    group.bench_function("deserialize", |b| {
        let bytes = MessagePackSerializer.serialize_request(&request).unwrap();
        b.iter(|| {
            let req: Request = MessagePackSerializer.deserialize_request(black_box(&bytes)).unwrap();
            black_box(req)
        })
    });
    group.finish();
}

fn bench_response_serialization(c: &mut Criterion) {
    let response = Response::success(1, vec![10, 20, 30, 40, 50]);

    let mut group = c.benchmark_group("response_serialization");
    group.throughput(Throughput::Bytes(5));
    group.bench_function("serialize", |b| {
        b.iter(|| {
            let bytes = MessagePackSerializer.serialize_response(black_box(&response)).unwrap();
            black_box(bytes)
        })
    });
    group.bench_function("deserialize", |b| {
        let bytes = MessagePackSerializer.serialize_response(&response).unwrap();
        b.iter(|| {
            let resp: Response = MessagePackSerializer.deserialize_response(black_box(&bytes)).unwrap();
            black_box(resp)
        })
    });
    group.finish();
}

fn bench_error_serialization(c: &mut Criterion) {
    let error = ErrorMessage::new(1, 500, "Internal server error".to_string());

    let mut group = c.benchmark_group("error_serialization");
    group.throughput(Throughput::Bytes(21));
    group.bench_function("serialize", |b| {
        b.iter(|| {
            let bytes = MessagePackSerializer.serialize_error(black_box(&error)).unwrap();
            black_box(bytes)
        })
    });
    group.bench_function("deserialize", |b| {
        let bytes = MessagePackSerializer.serialize_error(&error).unwrap();
        b.iter(|| {
            let err: ErrorMessage = MessagePackSerializer.deserialize_error(black_box(&bytes)).unwrap();
            black_box(err)
        })
    });
    group.finish();
}

fn bench_large_payload_serialization(c: &mut Criterion) {
    let sizes = [1024, 10_240, 102_400];

    for &size in &sizes {
        let payload = vec![0xAB; size];
        let response = Response::success(1, payload);

        let mut group = c.benchmark_group(format!("large_payload_{}b", size));
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function("serialize", |b| {
            b.iter(|| {
                let bytes = MessagePackSerializer.serialize_response(black_box(&response)).unwrap();
                black_box(bytes)
            })
        });
        group.bench_function("deserialize", |b| {
            let bytes = MessagePackSerializer.serialize_response(&response).unwrap();
            b.iter(|| {
                let resp: Response = MessagePackSerializer.deserialize_response(black_box(&bytes)).unwrap();
                black_box(resp)
            })
        });
        group.finish();
    }
}

fn bench_header_operations(c: &mut Criterion) {
    use memlink_protocol::{MessageHeader, MessageType};

    let header = MessageHeader::new(MessageType::Request, 1, 42, 0x1234, 100);

    let mut group = c.benchmark_group("header_operations");
    group.bench_function("as_bytes", |b| {
        b.iter(|| {
            let bytes = black_box(&header).as_bytes();
            black_box(bytes)
        })
    });
    group.bench_function("from_bytes", |b| {
        let bytes = header.as_bytes();
        b.iter(|| {
            let h = MessageHeader::from_bytes(black_box(&bytes)).unwrap();
            black_box(h)
        })
    });
    group.bench_function("validate", |b| {
        b.iter(|| {
            let valid = black_box(&header).validate().is_ok();
            black_box(valid)
        })
    });
    group.finish();
}

fn bench_version_negotiation(c: &mut Criterion) {
    use memlink_protocol::{ProtocolVersion, negotiate_version, V1_0, V1_2};

    let mut group = c.benchmark_group("version_negotiation");
    group.bench_function("same_version", |b| {
        b.iter(|| {
            let result = negotiate_version(black_box(&V1_0), black_box(&V1_0));
            black_box(result)
        })
    });
    group.bench_function("client_newer", |b| {
        b.iter(|| {
            let result = negotiate_version(black_box(&V1_2), black_box(&V1_0));
            black_box(result)
        })
    });
    group.bench_function("server_newer", |b| {
        b.iter(|| {
            let result = negotiate_version(black_box(&V1_0), black_box(&V1_2));
            black_box(result)
        })
    });
    group.bench_function("incompatible", |b| {
        let v2_0 = ProtocolVersion::new(2, 0, 0);
        b.iter(|| {
            let result = negotiate_version(black_box(&v2_0), black_box(&V1_0));
            black_box(result)
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_request_serialization,
    bench_response_serialization,
    bench_error_serialization,
    bench_large_payload_serialization,
    bench_header_operations,
    bench_version_negotiation,
);

criterion_main!(benches);
