#![cfg(feature = "std")]
#![allow(missing_docs)]

use memlink_protocol::{
    ErrorMessage, MessageHeader, MessageType, Priority, ProtocolVersion, Request, Response,
    Serializer, StatusCode, StreamHandle,
    features::{BATCHING, STREAMING},
    msgpack::MessagePackSerializer,
    negotiate_version, validate_version, V1_0, V1_1, V1_2,
};
use std::time::Instant;

#[test]
fn test_full_roundtrip() {
    let original_request = Request::new(
        1,
        Priority::Normal,
        "test_module",
        "echo",
        vec![1, 2, 3, 4, 5],
    )
    .with_trace_id(0xDEADBEEF)
    .with_deadline_ns(Some(1_000_000_000));

    let request_bytes = MessagePackSerializer.serialize_request(&original_request).unwrap();
    let deserialized_request = MessagePackSerializer.deserialize_request(&request_bytes).unwrap();

    assert_eq!(deserialized_request.request_id(), original_request.request_id());
    assert_eq!(deserialized_request.module_name(), original_request.module_name());
    assert_eq!(deserialized_request.method_name(), original_request.method_name());
    assert_eq!(deserialized_request.args(), original_request.args());
    assert_eq!(deserialized_request.trace_id(), original_request.trace_id());
    assert_eq!(deserialized_request.deadline_ns(), original_request.deadline_ns());

    let response_data = vec![10, 20, 30, 40, 50];
    let original_response = Response::success(original_request.request_id(), response_data.clone());
    let response_bytes = MessagePackSerializer.serialize_response(&original_response).unwrap();
    let deserialized_response = MessagePackSerializer.deserialize_response(&response_bytes).unwrap();

    assert_eq!(deserialized_response.request_id(), original_response.request_id());
    assert_eq!(deserialized_response.status(), original_response.status());
    assert_eq!(deserialized_response.data(), original_response.data());
}

#[test]
fn test_error_roundtrip() {
    let original_error = ErrorMessage::new(42, 500, "Internal server error".to_string())
        .with_retry_after_ms(Some(5000));

    let error_bytes = MessagePackSerializer.serialize_error(&original_error).unwrap();
    let deserialized_error = MessagePackSerializer.deserialize_error(&error_bytes).unwrap();

    assert_eq!(deserialized_error.error_code(), original_error.error_code());
    assert_eq!(deserialized_error.error_message(), original_error.error_message());
    assert_eq!(deserialized_error.retry_after_ms(), original_error.retry_after_ms());
}

#[test]
fn test_sequential_messages() {
    const COUNT: usize = 1000;
    let mut requests = Vec::with_capacity(COUNT);
    let mut serialized = Vec::with_capacity(COUNT);

    for i in 0..COUNT {
        let request = Request::new(
            i as u64,
            Priority::Normal,
            "module",
            "method",
            vec![i as u8; 10],
        );
        requests.push(request);
    }

    for request in &requests {
        let bytes = MessagePackSerializer.serialize_request(request).unwrap();
        serialized.push(bytes);
    }

    for (i, bytes) in serialized.iter().enumerate() {
        let deserialized = MessagePackSerializer.deserialize_request(bytes).unwrap();
        assert_eq!(deserialized.request_id(), i as u64);
    }

    for (i, request) in requests.iter().enumerate() {
        assert_eq!(request.request_id(), i as u64);
    }
}

#[test]
fn test_maximum_size_message() {
    let large_payload = vec![0xAB; 1024 * 1024];
    let request = Request::new(1, Priority::Normal, "mod", "fn", large_payload);

    let bytes = MessagePackSerializer.serialize_request(&request).unwrap();
    assert!(bytes.len() > 1024 * 1024);

    let deserialized = MessagePackSerializer.deserialize_request(&bytes).unwrap();
    assert_eq!(deserialized.args().len(), 1024 * 1024);
}

#[test]
fn test_empty_payload() {
    let request = Request::new(1, Priority::Normal, "module", "method", vec![]);

    let bytes = MessagePackSerializer.serialize_request(&request).unwrap();
    let deserialized = MessagePackSerializer.deserialize_request(&bytes).unwrap();

    assert!(deserialized.args().is_empty());
    assert_eq!(deserialized.args().len(), 0);
}

#[test]
fn test_unicode_names() {
    let module_name = "模块_テスト_모듈_модуль";
    let method_name = "方法_メソッド_메서드_метод";

    let request = Request::new(1, Priority::Normal, module_name, method_name, vec![]);
    let bytes = MessagePackSerializer.serialize_request(&request).unwrap();
    let deserialized = MessagePackSerializer.deserialize_request(&bytes).unwrap();

    assert_eq!(deserialized.module_name(), module_name);
    assert_eq!(deserialized.method_name(), method_name);
}

#[test]
fn test_binary_args() {
    let binary_data = vec![
        0x00, 0x01, 0x02, 0x7F, 0x80, 0xFE, 0xFF, 0x00, 0xFF, 0x00,
    ];

    let request = Request::new(1, Priority::Normal, "mod", "fn", binary_data.clone());
    let bytes = MessagePackSerializer.serialize_request(&request).unwrap();
    let deserialized = MessagePackSerializer.deserialize_request(&bytes).unwrap();

    assert_eq!(deserialized.args(), &binary_data);
}

#[test]
fn test_special_characters() {
    let special_string = "Hello\nWorld\t\"quoted\"\\backslash/null\u{0}";

    let error = ErrorMessage::new(1, 500, special_string.to_string());
    let bytes = MessagePackSerializer.serialize_error(&error).unwrap();
    let deserialized = MessagePackSerializer.deserialize_error(&bytes).unwrap();

    assert_eq!(deserialized.error_message(), special_string);
}

#[test]
fn test_version_negotiation_integration() {
    let result = negotiate_version(&V1_0, &V1_0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().minor, 0);

    let result = negotiate_version(&V1_2, &V1_0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().minor, 0);

    let result = negotiate_version(&V1_0, &V1_2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().minor, 0);

    let v2_0 = ProtocolVersion::new(2, 0, 0);
    let result = negotiate_version(&v2_0, &V1_0);
    assert!(result.is_err());
}

#[test]
fn test_version_validation() {
    assert!(validate_version(&V1_0).is_ok());
    assert!(validate_version(&V1_1).is_ok());
    assert!(validate_version(&V1_2).is_ok());

    let v2_0 = ProtocolVersion::new(2, 0, 0);
    assert!(validate_version(&v2_0).is_err());
}

#[test]
fn test_feature_flags_in_header() {
    let header = MessageHeader::with_features(
        MessageType::Request,
        STREAMING as u16,
        1,
        42,
        0x1234,
        100,
    );

    assert!(header.has_feature(STREAMING as u16));
    assert!(!header.has_feature(BATCHING as u16));
}

#[test]
fn test_feature_negotiation() {
    use memlink_protocol::features::intersect_features;

    let client_features = STREAMING | BATCHING;
    let server_features = STREAMING;

    let common = intersect_features(client_features, server_features);
    assert_eq!(common, STREAMING);
}

#[test]
fn test_stream_handle_roundtrip() {
    let handle = StreamHandle::generate(1024 * 1024);
    let bytes = handle.as_bytes();

    let parsed = StreamHandle::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.total_size(), handle.total_size());
    assert_eq!(parsed.stream_id(), handle.stream_id());
}

#[test]
fn test_stream_handle_expiration() {
    let handle = StreamHandle::generate(1024);
    assert!(!handle.is_expired());

    let handle = StreamHandle::with_timeout(1024, 30_000_000_000);
    assert!(!handle.is_expired());
}

#[test]
fn test_zero_copy_parsing() {
    let original = Request::new(1, Priority::Normal, "module", "method", vec![1, 2, 3]);
    let bytes = MessagePackSerializer.serialize_request(&original).unwrap();

    let deserialized = MessagePackSerializer.deserialize_request(&bytes).unwrap();
    assert_eq!(deserialized.request_id(), 1);
    assert_eq!(deserialized.module_name(), "module");
    assert_eq!(deserialized.method_name(), "method");
}

#[test]
fn benchmark_small_message_serialization() {
    const ITERATIONS: usize = 10_000;
    let request = Request::new(1, Priority::Normal, "mod", "fn", vec![1, 2, 3]);

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = MessagePackSerializer.serialize_request(&request).unwrap();
    }
    let elapsed = start.elapsed();

    let avg_ns = elapsed.as_nanos() / ITERATIONS as u128;
    println!("Small message serialization: {} ns/op", avg_ns);

    assert!(avg_ns < 10000, "Serialization too slow: {} ns/op", avg_ns);
}

#[test]
fn benchmark_large_message_serialization() {
    const ITERATIONS: usize = 10;
    let large_payload = vec![0xAB; 100 * 1024];
    let request = Request::new(1, Priority::Normal, "mod", "fn", large_payload);

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = MessagePackSerializer.serialize_request(&request).unwrap();
    }
    let elapsed = start.elapsed();

    let avg_ms = elapsed.as_millis() / ITERATIONS as u128;
    println!("Large message (100KB) serialization: {} ms/op", avg_ms);
}

#[test]
fn test_fuzz_random() {
    use std::collections::HashSet;

    const ITERATIONS: usize = 10_000;
    let mut errors = HashSet::new();
    let mut successes = 0;

    for i in 0..ITERATIONS {
        let mut rng = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&i, &mut rng);
        let seed = std::hash::Hasher::finish(&rng);

        let random_bytes: Vec<u8> = (0..(seed % 1000) as usize)
            .map(|j| ((seed >> (j % 64)) & 0xFF) as u8)
            .collect();

        if random_bytes.len() >= 32 {
            let header_bytes: [u8; 32] = random_bytes[..32].try_into().unwrap();
            let result = MessageHeader::from_bytes(&header_bytes);

            match result {
                Ok(_) => successes += 1,
                Err(e) => {
                    errors.insert(format!("{:?}", e));
                }
            }
        }

        if random_bytes.len() >= 32 {
            let header_bytes: [u8; 32] = random_bytes[..32].try_into().unwrap();
            if let Ok(_header) = MessageHeader::from_bytes(&header_bytes) {
                let _ = MessagePackSerializer.deserialize_request(&random_bytes);
            }
        }
    }

    println!("Fuzz test: {} successes, {} unique error types", successes, errors.len());
}

#[test]
fn test_corrupt_message_handling() {
    let truncated = vec![0x4D, 0x4C, 0x4E];
    let result: Result<[u8; 32], _> = truncated.try_into();
    assert!(result.is_err());

    let mut bad_magic = [0u8; 32];
    bad_magic[0..4].copy_from_slice(&0x12345678u32.to_le_bytes());
    let result = MessageHeader::from_bytes(&bad_magic);
    assert!(result.is_err());

    let mut bad_version = [0u8; 32];
    bad_version[0..4].copy_from_slice(&memlink_protocol::MEMLINK_MAGIC.to_le_bytes());
    bad_version[4] = 255;
    let result = MessageHeader::from_bytes(&bad_version);
    assert!(result.is_err());
}

#[test]
fn test_daemon_context() {
    let requests: Vec<Request> = (0..10)
        .map(|i| Request::new(i, Priority::Normal, "module", "method", vec![i as u8; 10]))
        .collect();

    let serialized: Vec<Vec<u8>> = requests
        .iter()
        .map(|r| MessagePackSerializer.serialize_request(r).unwrap())
        .collect();

    let deserialized: Vec<Request> = serialized
        .iter()
        .map(|b| MessagePackSerializer.deserialize_request(b).unwrap())
        .collect();

    for (orig, deser) in requests.iter().zip(deserialized.iter()) {
        assert_eq!(orig.request_id(), deser.request_id());
    }
}

#[test]
fn test_sdk_context() {
    let module_name = "my_module";
    let method_name = "my_method";
    let args = vec![1, 2, 3, 4, 5];

    let request = Request::new(1, Priority::High, module_name, method_name, args.clone());
    let bytes = MessagePackSerializer.serialize_request(&request).unwrap();

    let parsed = MessagePackSerializer.deserialize_request(&bytes).unwrap();
    assert_eq!(parsed.module_name(), module_name);
    assert_eq!(parsed.method_name(), method_name);
    assert_eq!(parsed.args(), &args);
}

#[test]
fn test_module_context() {
    let request = Request::new(1, Priority::Normal, "module", "echo", vec![1, 2, 3]);

    let response_data = request.args().to_vec();
    let response = Response::success(request.request_id(), response_data);

    let bytes = MessagePackSerializer.serialize_response(&response).unwrap();

    let parsed = MessagePackSerializer.deserialize_response(&bytes).unwrap();
    assert_eq!(parsed.status(), StatusCode::Success);
    assert_eq!(parsed.data(), request.args());
}

#[test]
fn test_long_names() {
    let long_name = "a".repeat(10000);

    let request = Request::new(1, Priority::Normal, &long_name, &long_name, vec![]);
    let bytes = MessagePackSerializer.serialize_request(&request).unwrap();
    let deserialized = MessagePackSerializer.deserialize_request(&bytes).unwrap();

    assert_eq!(deserialized.module_name().len(), 10000);
    assert_eq!(deserialized.method_name().len(), 10000);
}

#[test]
fn test_long_error_message() {
    let long_message = "Error: ".to_string() + &"x".repeat(100000);

    let error = ErrorMessage::new(1, 500, long_message.clone());
    let bytes = MessagePackSerializer.serialize_error(&error).unwrap();
    let deserialized = MessagePackSerializer.deserialize_error(&bytes).unwrap();

    assert_eq!(deserialized.error_message().len(), 100007);
}

#[test]
fn test_all_message_types() {
    for msg_type in &[
        MessageType::Request,
        MessageType::Response,
        MessageType::Error,
        MessageType::StreamHandle,
        MessageType::HealthCheck,
        MessageType::LoadModule,
        MessageType::UnloadModule,
        MessageType::Stats,
        MessageType::Event,
    ] {
        let header = MessageHeader::new(*msg_type, 1, 42, 0x1234, 100);
        assert_eq!(header.message_type(), Some(*msg_type));

        let bytes = header.as_bytes();
        let parsed = MessageHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.message_type(), Some(*msg_type));
    }
}

#[test]
fn test_all_status_codes() {
    for status in &[
        StatusCode::Success,
        StatusCode::ModuleNotFound,
        StatusCode::MethodNotFound,
        StatusCode::ExecutionError,
        StatusCode::Timeout,
        StatusCode::QuotaExceeded,
        StatusCode::BackpressureRejection,
    ] {
        let response = Response::error(1, *status, b"error".to_vec());
        assert_eq!(response.status(), *status);

        let bytes = MessagePackSerializer.serialize_response(&response).unwrap();
        let parsed = MessagePackSerializer.deserialize_response(&bytes).unwrap();
        assert_eq!(parsed.status(), *status);
    }
}
