#![cfg(not(windows))]

use edge_sandbox::ffi::{
    EdgeSandboxBuffer, EdgeSandboxDeterministicOptions, EdgeSandboxLimits,
    edge_sandbox_buffer_free, edge_sandbox_create_self_hosted_with_options, edge_sandbox_destroy,
    edge_sandbox_evaluate, edge_sandbox_options_append_iframe_hook,
    edge_sandbox_options_append_network_replay, edge_sandbox_options_append_network_replay_header,
    edge_sandbox_options_clear_network_replay, edge_sandbox_options_create,
    edge_sandbox_options_destroy, edge_sandbox_options_schema_version,
    edge_sandbox_options_set_cross_origin_isolated, edge_sandbox_options_set_deterministic,
    edge_sandbox_options_set_limits, edge_sandbox_options_set_page, edge_sandbox_options_validate,
};

fn take_buffer(buffer: &mut EdgeSandboxBuffer) -> String {
    let output = if buffer.data.is_null() || buffer.len == 0 {
        String::new()
    } else {
        // SAFETY: the native API returned a readable allocation of this length.
        String::from_utf8_lossy(unsafe {
            std::slice::from_raw_parts(buffer.data.cast_const(), buffer.len)
        })
        .into_owned()
    };
    // SAFETY: this buffer came from the native API and is freed exactly once.
    unsafe {
        edge_sandbox_buffer_free(buffer);
    }
    output
}

fn assert_native_call(succeeded: bool, error: &mut EdgeSandboxBuffer) {
    assert!(succeeded, "{}", take_buffer(error));
    assert!(take_buffer(error).is_empty());
}

fn evaluate(runtime: *mut edge_sandbox::ffi::EdgeSandboxHandle, source: &str) -> String {
    let mut result = EdgeSandboxBuffer::default();
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the runtime and source remain live for this synchronous call.
    let succeeded = unsafe {
        edge_sandbox_evaluate(
            runtime,
            source.as_ptr(),
            source.len(),
            &mut result,
            &mut error,
        )
    };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
    take_buffer(&mut result)
}

#[test]
fn complete_typed_options_cross_ffi_and_binary_worker_boundaries() {
    assert_eq!(edge_sandbox_options_schema_version(), 3);
    let mut error = EdgeSandboxBuffer::default();
    let options = edge_sandbox_options_create(&mut error);
    assert!(!options.is_null(), "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());

    let url = b"https://options.example/app/index.html";
    let html = br#"<!doctype html><title>Options Page</title>
        <main id="configured" class="ready">typed page</main>"#;
    let referrer = b"https://referrer.example/start";
    let content_type = b"text/html; charset=utf-8";
    // SAFETY: the options handle and all input buffers are live.
    assert_native_call(
        unsafe {
            edge_sandbox_options_set_page(
                options,
                url.as_ptr(),
                url.len(),
                html.as_ptr(),
                html.len(),
                referrer.as_ptr(),
                referrer.len(),
                content_type.as_ptr(),
                content_type.len(),
                &mut error,
            )
        },
        &mut error,
    );

    assert_native_call(
        unsafe { edge_sandbox_options_set_cross_origin_isolated(options, true, &mut error) },
        &mut error,
    );

    // SAFETY: the options handle is live and uniquely used by this test.
    assert_native_call(
        unsafe { edge_sandbox_options_clear_network_replay(options, &mut error) },
        &mut error,
    );
    let replay_url = b"https://options.example/api/value";
    let method = b"GET";
    let status_text = b"OK";
    let body = b"offline-body";
    // SAFETY: the options handle and all input buffers are live.
    assert_native_call(
        unsafe {
            edge_sandbox_options_append_network_replay(
                options,
                replay_url.as_ptr(),
                replay_url.len(),
                method.as_ptr(),
                method.len(),
                200,
                status_text.as_ptr(),
                status_text.len(),
                body.as_ptr(),
                body.len(),
                &mut error,
            )
        },
        &mut error,
    );
    let header_name = b"content-type";
    let header_value = b"text/plain";
    // SAFETY: the first replay entry and both strings are live.
    assert_native_call(
        unsafe {
            edge_sandbox_options_append_network_replay_header(
                options,
                0,
                header_name.as_ptr(),
                header_name.len(),
                header_value.as_ptr(),
                header_value.len(),
                &mut error,
            )
        },
        &mut error,
    );

    let deterministic = EdgeSandboxDeterministicOptions {
        clock_epoch_ms: 1_700_000_000_000,
        clock_step_ms: 2,
        random_seed: 1234,
        max_task_turns: 2048,
        has_clock_epoch_ms: 1,
        has_random_seed: 1,
        reserved: [0; 6],
    };
    // SAFETY: both typed structures and the options handle are live.
    assert_native_call(
        unsafe { edge_sandbox_options_set_deterministic(options, &deterministic, &mut error) },
        &mut error,
    );
    let limits = EdgeSandboxLimits {
        timeout_ms: 5_000,
        max_heap_bytes: 256 * 1024 * 1024,
        max_resident_bytes: 768 * 1024 * 1024,
        max_source_bytes: 2 * 1024 * 1024,
        max_output_bytes: 2 * 1024 * 1024,
    };
    // SAFETY: both typed structures and the options handle are live.
    assert_native_call(
        unsafe { edge_sandbox_options_set_limits(options, &limits, &mut error) },
        &mut error,
    );
    let hook_name = b"xhr-open";
    let hook_source = br#"
        parent.__hookBindingWasGlobal = "__edgev8" in window;
        const originalOpen = XMLHttpRequest.prototype.open;
        XMLHttpRequest.prototype.open = __edgev8.proxy(function open(method, url) {
          parent.__hookedIframeUrl = String(url);
          return Reflect.apply(originalOpen, this, arguments);
        }, "open");
    "#;
    assert_native_call(
        unsafe {
            edge_sandbox_options_append_iframe_hook(
                options,
                hook_name.as_ptr(),
                hook_name.len(),
                hook_source.as_ptr(),
                hook_source.len(),
                &mut error,
            )
        },
        &mut error,
    );
    // SAFETY: the complete options builder is live.
    assert_native_call(
        unsafe { edge_sandbox_options_validate(options, &mut error) },
        &mut error,
    );

    // SAFETY: the typed options remain live for this call.
    let runtime = unsafe { edge_sandbox_create_self_hosted_with_options(options, &mut error) };
    assert!(!runtime.is_null(), "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());

    let result = evaluate(
        runtime,
        r#"
        fetch("https://options.example/api/value").then(async response => [
          document.title,
          document.getElementById("configured").className,
          location.href,
          document.referrer,
          await response.text(),
          response.headers.get("content-type"),
          Date.now() >= 1700000000000
        ].join("|"))
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "Options Page|ready|https://options.example/app/index.html|",
            "https://referrer.example/start|offline-body|text/plain|true"
        )
    );
    let iframe_result = evaluate(
        runtime,
        r#"
        (() => {
          const frame = document.createElement("iframe");
          frame.srcdoc = `<script>
            const request = new XMLHttpRequest();
            request.open("POST", "/inside-frame");
          <\/script>`;
          document.body.appendChild(frame);
          return [
            __hookedIframeUrl,
            frame.contentWindow.Function.prototype.toString.call(
              frame.contentWindow.XMLHttpRequest.prototype.open
            ),
            __hookBindingWasGlobal,
            "__edgev8" in frame.contentWindow,
            typeof frame.contentWindow.__edgev8,
            Reflect.ownKeys(frame.contentWindow).includes("__edgev8")
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        iframe_result,
        "/inside-frame|function open() { [native code] }|false|false|undefined|false"
    );

    // SAFETY: both handles are live and destroyed exactly once.
    unsafe {
        edge_sandbox_destroy(runtime);
        edge_sandbox_options_destroy(options);
    }
}

#[test]
fn invalid_typed_options_are_rejected_before_worker_creation() {
    let mut error = EdgeSandboxBuffer::default();
    let options = edge_sandbox_options_create(&mut error);
    assert!(!options.is_null(), "{}", take_buffer(&mut error));
    let url = b"http://insecure.example/";
    let html = b"";
    let referrer = b"";
    let content_type = b"text/html";
    // SAFETY: the options handle and all input strings are live.
    assert!(!unsafe {
        edge_sandbox_options_set_page(
            options,
            url.as_ptr(),
            url.len(),
            html.as_ptr(),
            html.len(),
            referrer.as_ptr(),
            referrer.len(),
            content_type.as_ptr(),
            content_type.len(),
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).contains("HTTPS"));
    // SAFETY: the options handle is live and destroyed once.
    unsafe {
        edge_sandbox_options_destroy(options);
    }
}
