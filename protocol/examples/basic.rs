//! Basic protocol usage example.
//!
//! Demonstrates creating, serializing, and deserializing protocol messages.

use memlink_protocol::{ErrorMessage, MessageHeader, MessageType, Request, Response};
use memlink_protocol::msgpack::MessagePackSerializer;
use memlink_protocol::serializer::Serializer;

fn main() {
    println!("=== MemLink Protocol Examples ===\n");

    example_header_operations();
    example_request_response();
    example_error_handling();
    example_version_negotiation();
}

fn example_header_operations() {
    println!("1. Header Operations");
    println!("   -----------------");

    let header = MessageHeader::new(MessageType::Request, 1, 42, 0x1234, 256);

    println!("   Created header:");
    println!("   - Magic: 0x{:08X}", header.magic());
    println!("   - Version: {}", header.version());
    println!("   - Type: {:?}", header.message_type());
    println!("   - Request ID: {}", header.request_id());
    println!("   - Payload: {} bytes", header.payload_len());

    let bytes = header.as_bytes();
    let parsed = MessageHeader::from_bytes(&bytes).unwrap();

    println!("   ✓ Roundtrip successful: {}", parsed.validate().is_ok());
    println!();
}

fn example_request_response() {
    println!("2. Request/Response Cycle");
    println!("   ----------------------");

    let request = Request::new(
        1,
        memlink_protocol::Priority::Normal,
        "calculator",
        "add",
        vec![1, 2, 3, 4],
    )
    .with_trace_id(0xDEADBEEF);

    println!("   Created request:");
    println!("   - Module: {}", request.module_name());
    println!("   - Method: {}", request.method_name());
    println!("   - Args: {} bytes", request.args().len());
    println!("   - Trace ID: 0x{:016X}", request.trace_id());

    let request_bytes = MessagePackSerializer.serialize_request(&request).unwrap();
    println!("   - Serialized: {} bytes", request_bytes.len());

    let _parsed_request = MessagePackSerializer.deserialize_request(&request_bytes).unwrap();
    println!("   ✓ Request roundtrip successful");

    let response = Response::success(
        request.request_id(),
        vec![10, 20, 30, 40, 50],
    );

    println!("\n   Created response:");
    println!("   - Status: {:?}", response.status());
    println!("   - Data: {} bytes", response.data().len());

    let response_bytes = MessagePackSerializer.serialize_response(&response).unwrap();
    println!("   - Serialized: {} bytes", response_bytes.len());

    let _parsed_response = MessagePackSerializer.deserialize_response(&response_bytes).unwrap();
    println!("   ✓ Response roundtrip successful");
    println!();
}

fn example_error_handling() {
    println!("3. Error Handling");
    println!("   --------------");

    let error = ErrorMessage::new(
        42,
        500,
        "Internal server error".to_string(),
    )
    .with_retry_after_ms(Some(5000));

    println!("   Created error:");
    println!("   - Code: {}", error.error_code());
    println!("   - Message: {}", error.error_message());
    println!("   - Retry after: {:?} ms", error.retry_after_ms());

    let error_bytes = MessagePackSerializer.serialize_error(&error).unwrap();
    println!("   - Serialized: {} bytes", error_bytes.len());

    let _parsed_error = MessagePackSerializer.deserialize_error(&error_bytes).unwrap();
    println!("   ✓ Error roundtrip successful");

    let invalid_error = ErrorMessage::new(1, 404, "Not found".to_string());
    println!("\n   Created another error:");
    println!("   - Code: {}", invalid_error.error_code());
    println!("   - Message: {}", invalid_error.error_message());
    println!();
}

fn example_version_negotiation() {
    use memlink_protocol::{ProtocolVersion, negotiate_version, validate_version, V1_0, V1_1, V1_2};

    println!("4. Version Negotiation");
    println!("   -------------------");

    println!("   Supported versions: V1_0, V1_1, V1_2");
    println!("   Current version: V1_2");

    let result = negotiate_version(&V1_2, &V1_0);
    match result {
        Ok(v) => println!("   ✓ V1.2 client + V1.0 server = V{}.{}", v.major(), v.minor()),
        Err(e) => println!("   ✗ Negotiation failed: {:?}", e),
    }

    let result = negotiate_version(&V1_0, &V1_2);
    match result {
        Ok(v) => println!("   ✓ V1.0 client + V1.2 server = V{}.{}", v.major(), v.minor()),
        Err(e) => println!("   ✗ Negotiation failed: {:?}", e),
    }

    let v2_0 = ProtocolVersion::new(2, 0, 0);
    let result = negotiate_version(&v2_0, &V1_0);
    match result {
        Ok(_) => println!("   ✗ V2.0 should be incompatible"),
        Err(_) => println!("   ✓ V2.0 client + V1.0 server = Incompatible (as expected)"),
    }

    println!("\n   Version validation:");
    println!("   - V1.0: {}", validate_version(&V1_0).is_ok());
    println!("   - V1.1: {}", validate_version(&V1_1).is_ok());
    println!("   - V1.2: {}", validate_version(&V1_2).is_ok());
    println!("   - V2.0: {}", validate_version(&v2_0).is_err());
    println!();
}
