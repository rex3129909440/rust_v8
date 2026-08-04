use std::path::PathBuf;
use std::process::Command;

#[test]
fn opaque_ips_script_exports_tl_request_without_native_trace() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("demo")
        .join("ips.js");
    assert!(script.is_file(), "black-box fixture is missing");

    let output = Command::new(env!("CARGO_BIN_EXE_edge-sandbox"))
        .arg("run")
        .arg(&script)
        .arg("--requests")
        .output()
        .expect("run opaque ips.js fixture");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "black-box sandbox execution failed: {stderr}"
    );
    let lines = stdout.lines().collect::<Vec<_>>();
    let request_index = lines
        .iter()
        .position(|line| {
            line.starts_with("REQUEST\t")
                && line.contains("\tXMLHttpRequest\tPOST\t")
                && line.ends_with("/tl")
        })
        .unwrap_or_else(|| {
            panic!(
                "black-box execution did not export an XMLHttpRequest POST ending in /tl: {stdout}"
            )
        });
    assert!(
        lines[request_index + 1..]
            .iter()
            .take_while(|line| **line != "END_REQUEST")
            .any(|line| line.starts_with("BODY\t")),
        "the exported /tl request did not include a body field: {stdout}"
    );
    assert!(
        !stdout.contains("TRACE\t"),
        "network export unexpectedly enabled native trace: {stdout}"
    );
}
