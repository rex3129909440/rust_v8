#![cfg(not(windows))]

use edge_sandbox::{EdgeRuntimeOptions, IsolatedEdgeRuntime, NetworkRequestSource};
use std::path::PathBuf;

#[test]
fn opaque_ips_script_exports_tl_request_without_native_trace() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("demo")
        .join("ips.js");
    assert!(script.is_file(), "black-box fixture is missing");

    let source = std::fs::read_to_string(&script).expect("read black-box fixture bytes");
    let runtime =
        IsolatedEdgeRuntime::self_hosted(EdgeRuntimeOptions::default()).expect("isolated runtime");
    runtime
        .evaluate_with_source_url(&source, "blackbox://ips.js")
        .expect("run opaque ips.js fixture");
    let requests = runtime.network_requests().expect("export network requests");
    let request = requests
        .iter()
        .find(|request| {
            request.source == NetworkRequestSource::XmlHttpRequest
                && request.method == "POST"
                && request.url.ends_with("/tl")
        })
        .expect("black-box execution did not export an XMLHttpRequest POST ending in /tl");
    assert!(
        !request.body.is_empty(),
        "the exported /tl request did not include a body"
    );
    assert!(
        runtime
            .proxy_trace()
            .expect("read disabled trace")
            .is_empty()
    );
}
