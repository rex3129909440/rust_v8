use edge_sandbox::ffi::{
    EdgeSandboxBuffer, edge_sandbox_buffer_free, edge_sandbox_clear_network_requests,
    edge_sandbox_create, edge_sandbox_destroy, edge_sandbox_evaluate,
    edge_sandbox_network_requests,
};

fn take_bytes(buffer: &mut EdgeSandboxBuffer) -> Vec<u8> {
    let output = if buffer.data.is_null() || buffer.len == 0 {
        Vec::new()
    } else {
        // SAFETY: the native API returned a readable allocation of this length.
        unsafe { std::slice::from_raw_parts(buffer.data.cast_const(), buffer.len) }.to_vec()
    };
    // SAFETY: this buffer came from the native API and is freed exactly once.
    unsafe {
        edge_sandbox_buffer_free(buffer);
    }
    output
}

#[test]
fn ffi_exports_versioned_binary_requests_without_trace() {
    let worker = env!("CARGO_BIN_EXE_edge-sandbox");
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the worker path and error output are live for the call.
    let runtime = unsafe { edge_sandbox_create(worker.as_ptr(), worker.len(), &mut error) };
    assert!(
        !runtime.is_null(),
        "{}",
        String::from_utf8_lossy(&take_bytes(&mut error))
    );
    assert!(take_bytes(&mut error).is_empty());

    let source = r#"
        const xhr = new XMLHttpRequest();
        xhr.open("POST", "https://ffi.example/collect");
        xhr.setRequestHeader("X-Binary", "present");
        xhr.send("body-from-ffi");
    "#;
    let mut result = EdgeSandboxBuffer::default();
    // SAFETY: the runtime, source, and output buffers are live.
    assert!(unsafe {
        edge_sandbox_evaluate(
            runtime,
            source.as_ptr(),
            source.len(),
            &mut result,
            &mut error,
        )
    });
    take_bytes(&mut result);
    assert!(take_bytes(&mut error).is_empty());

    // No trace API is enabled. Network capture is queried independently.
    let mut requests = EdgeSandboxBuffer::default();
    // SAFETY: the runtime and output buffers are live.
    assert!(unsafe { edge_sandbox_network_requests(runtime, &mut requests, &mut error) });
    assert!(take_bytes(&mut error).is_empty());
    let requests = take_bytes(&mut requests);
    assert_eq!(&requests[..4], b"ESNR");
    assert_eq!(u16::from_le_bytes([requests[4], requests[5]]), 1);
    assert_eq!(
        u32::from_le_bytes([requests[8], requests[9], requests[10], requests[11]]),
        1
    );
    assert!(
        requests
            .windows(b"X-Binary".len())
            .any(|value| value == b"X-Binary")
    );
    assert!(
        requests
            .windows(b"body-from-ffi".len())
            .any(|value| value == b"body-from-ffi")
    );

    // SAFETY: the runtime and error output are live.
    assert!(unsafe { edge_sandbox_clear_network_requests(runtime, &mut error) });
    assert!(take_bytes(&mut error).is_empty());
    // SAFETY: the runtime is destroyed exactly once.
    unsafe {
        edge_sandbox_destroy(runtime);
    }
}
