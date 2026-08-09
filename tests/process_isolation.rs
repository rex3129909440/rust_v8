#![cfg(not(windows))]

use edge_sandbox::{EdgeRuntimeOptions, Evaluation, IsolatedEdgeRuntime, PageInit, SandboxLimits};
use std::time::Duration;

fn isolated_runtime(options: EdgeRuntimeOptions) -> IsolatedEdgeRuntime {
    IsolatedEdgeRuntime::self_hosted(options).expect("self-hosted isolated Edge worker")
}

#[test]
fn worker_is_a_distinct_process_and_preserves_runtime_state() {
    let runtime = isolated_runtime(EdgeRuntimeOptions::default());
    assert_ne!(
        runtime.process_id().expect("worker PID"),
        std::process::id()
    );
    assert!(
        runtime
            .resident_memory_bytes()
            .expect("resident-memory sample")
            .is_some()
    );
    assert_eq!(
        runtime
            .evaluate("globalThis.processBoundaryValue = 40; processBoundaryValue")
            .expect("first isolated evaluation"),
        Evaluation::Number("40".to_owned())
    );
    assert_eq!(
        runtime
            .evaluate("processBoundaryValue + 2")
            .expect("stateful isolated evaluation"),
        Evaluation::Number("42".to_owned())
    );
}

#[test]
fn worker_enforces_limits_and_remains_reusable_after_script_timeout() {
    let options = EdgeRuntimeOptions {
        limits: SandboxLimits {
            timeout: Some(Duration::from_millis(25)),
            max_heap_bytes: Some(64 * 1024 * 1024),
            max_resident_bytes: Some(512 * 1024 * 1024),
            max_source_bytes: Some(8 * 1024),
            max_output_bytes: Some(8 * 1024),
        },
        ..EdgeRuntimeOptions::default()
    };
    let runtime = isolated_runtime(options);
    let original_pid = runtime.process_id().expect("original worker PID");
    assert_eq!(
        runtime.evaluate("21 * 2").expect("isolated evaluation"),
        Evaluation::Number("42".to_owned())
    );
    let timeout = runtime
        .evaluate("while (true) {}")
        .expect_err("infinite script must time out");
    assert!(timeout.contains("timeout") || timeout.contains("exceeded"));
    assert_eq!(
        runtime.evaluate("6 * 7").expect("runtime reusable"),
        Evaluation::Number("42".to_owned())
    );
    assert_eq!(
        runtime.process_id().expect("reused worker PID"),
        original_pid
    );
}

#[test]
fn proxy_trace_crosses_the_binary_process_boundary() {
    let runtime = isolated_runtime(EdgeRuntimeOptions::default());
    runtime.enable_proxy_trace().expect("enable trace");
    assert_eq!(
        runtime
            .evaluate("navigator.userAgent.length > 0")
            .expect("traced evaluation"),
        Evaluation::Boolean(true)
    );
    let trace = runtime.proxy_trace().expect("proxy trace response");
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "get" && entry.api == "window.navigator")
    );
    runtime.clear_proxy_trace().expect("clear trace");
    assert!(runtime.proxy_trace().expect("cleared trace").is_empty());
    runtime.disable_proxy_trace().expect("disable trace");
}

#[test]
fn resident_memory_limit_contains_allocation_failure_and_controller_recovers() {
    let options = EdgeRuntimeOptions {
        limits: SandboxLimits {
            timeout: Some(Duration::from_secs(10)),
            max_heap_bytes: Some(1024 * 1024 * 1024),
            max_resident_bytes: Some(256 * 1024 * 1024),
            max_source_bytes: Some(8 * 1024),
            max_output_bytes: Some(8 * 1024),
        },
        ..EdgeRuntimeOptions::default()
    };
    let runtime = isolated_runtime(options);
    let original_pid = runtime.process_id().expect("original worker PID");
    let failure = runtime
        .evaluate(
            r#"
        (() => {
          const committed = [];
          for (let index = 0; index < 512; index += 1) {
            const chunk = new Uint8Array(1024 * 1024);
            chunk.fill(index & 255);
            committed.push(chunk);
          }
          return committed.length;
        })()
        "#,
        )
        .expect_err("resident-memory allocation must be bounded");
    assert!(
        failure.contains("max_resident_bytes"),
        "controller must report the resident-memory boundary: {failure}"
    );
    assert_eq!(
        runtime
            .evaluate("6 * 7")
            .expect("controller restarts or reuses a healthy worker"),
        Evaluation::Number("42".to_owned())
    );
    assert_ne!(
        runtime.process_id().expect("replacement worker PID"),
        original_pid
    );
}

#[test]
fn typed_page_init_crosses_the_binary_process_boundary() {
    let runtime = isolated_runtime(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://isolated.example/app/".to_owned(),
            html: "<title>Isolated</title><main id=app>ready</main>".to_owned(),
            referrer: "https://source.example/".to_owned(),
            content_type: "text/html".to_owned(),
        }),
        ..Default::default()
    });
    assert_eq!(
        runtime
            .evaluate(
                r#"
                [
                  location.href,
                  document.title,
                  document.getElementById("app").textContent,
                  document.referrer
                ].join("|")
                "#,
            )
            .expect("isolated initialized page"),
        Evaluation::String(
            "https://isolated.example/app/|Isolated|ready|https://source.example/".to_owned()
        )
    );
}
