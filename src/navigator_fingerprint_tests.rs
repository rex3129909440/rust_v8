use crate::{EdgeFingerprint, EdgeRuntime, Evaluation, UserAgentBrandFingerprint};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => value,
        value => value.to_string(),
    }
}

#[test]
fn default_navigator_profile_is_fixed_chrome_150_without_edge_tokens() {
    let mut runtime = EdgeRuntime::new().expect("default Chrome 150 runtime");
    let values = text(
        &mut runtime,
        r#"
        [
          navigator.userAgent,
          navigator.appVersion,
          navigator.userAgentData.brands
            .map(entry => `${entry.brand}:${entry.version}`)
            .join(",")
        ].join("|")
        "#,
    );
    assert_eq!(
        values,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36|\
5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36|\
Not_A Brand:99,Chromium:150,Google Chrome:150"
    );
    assert!(!values.contains("Edg/"), "{values}");
    assert!(!values.contains("HeadlessChrome/"), "{values}");
}

#[test]
fn navigator_prototype_keeps_window_controls_overlay_in_blink_order() {
    let mut runtime = EdgeRuntime::new().expect("navigator order runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const names = Object.getOwnPropertyNames(Navigator.prototype);
              return names.slice(
                names.indexOf("webkitPersistentStorage"),
                names.indexOf("constructor") + 1
              ).join(",");
            })()
            "#,
        ),
        concat!(
            "webkitPersistentStorage,windowControlsOverlay,hardwareConcurrency,",
            "cookieEnabled,",
            "appCodeName,appName,appVersion,platform,product,userAgent,language,",
            "languages,onLine,webdriver,plugins,mimeTypes,pdfViewerEnabled,",
            "connection,getGamepads,javaEnabled,sendBeacon,vibrate,constructor"
        )
    );
}

fn custom_fingerprint() -> EdgeFingerprint {
    let mut fingerprint = EdgeFingerprint {
        id: "windows-11-edge-149-custom".to_owned(),
        ..EdgeFingerprint::default()
    };
    let navigator = &mut fingerprint.navigator;
    navigator.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36 \
Edg/149.0.0.0"
        .to_owned();
    navigator.app_version = navigator
        .user_agent
        .strip_prefix("Mozilla/")
        .expect("Mozilla prefix")
        .to_owned();
    navigator.app_code_name = "MozillaCustom".to_owned();
    navigator.app_name = "NetscapeCustom".to_owned();
    navigator.platform = "Win64".to_owned();
    navigator.product = "GeckoCustom".to_owned();
    navigator.product_sub = "20260725".to_owned();
    navigator.vendor = "Microsoft Corporation".to_owned();
    navigator.vendor_sub = "Edge".to_owned();
    navigator.language = "en-US".to_owned();
    navigator.languages = vec!["en-US".to_owned(), "en".to_owned()];
    navigator.hardware_concurrency = 12;
    navigator.device_memory_gb = 4.0;
    navigator.max_touch_points = 0;
    navigator.cookie_enabled = false;
    navigator.on_line = false;
    navigator.webdriver = false;
    navigator.pdf_viewer_enabled = false;
    navigator.do_not_track = Some("1".to_owned());
    navigator.user_agent_data.brands = vec![
        UserAgentBrandFingerprint {
            brand: "Not/A)Brand".to_owned(),
            version: "8".to_owned(),
            full_version: "8.0.0.0".to_owned(),
        },
        UserAgentBrandFingerprint {
            brand: "Chromium".to_owned(),
            version: "149".to_owned(),
            full_version: "149.0.7753.0".to_owned(),
        },
        UserAgentBrandFingerprint {
            brand: "Microsoft Edge".to_owned(),
            version: "149".to_owned(),
            full_version: "149.0.7753.0".to_owned(),
        },
    ];
    navigator.user_agent_data.mobile = false;
    navigator.user_agent_data.platform = "Windows".to_owned();
    navigator.user_agent_data.architecture = "x86".to_owned();
    navigator.user_agent_data.bitness = "64".to_owned();
    navigator.user_agent_data.model = "Edge Desktop".to_owned();
    navigator.user_agent_data.platform_version = "20.0.0".to_owned();
    navigator.user_agent_data.ua_full_version = "149.0.7753.0".to_owned();
    navigator.user_agent_data.wow64 = true;
    navigator.user_agent_data.form_factors = vec!["Desktop".to_owned(), "Automotive".to_owned()];
    navigator.network.effective_type = "3g".to_owned();
    navigator.network.rtt = 150;
    navigator.network.downlink = 0.8;
    navigator.network.save_data = true;
    fingerprint
}

#[test]
fn navigator_values_are_driven_by_the_typed_fingerprint() {
    let mut runtime =
        EdgeRuntime::with_fingerprint(custom_fingerprint()).expect("custom Edge runtime");
    let values = text(
        &mut runtime,
        r#"
        [
          navigator.vendorSub,
          navigator.productSub,
          navigator.vendor,
          navigator.maxTouchPoints,
          navigator.hardwareConcurrency,
          navigator.cookieEnabled,
          navigator.appCodeName,
          navigator.appName,
          navigator.appVersion,
          navigator.platform,
          navigator.product,
          navigator.userAgent,
          navigator.language,
          navigator.languages.join(","),
          navigator.languages === navigator.languages,
          Object.isFrozen(navigator.languages),
          navigator.onLine,
          navigator.webdriver,
          navigator.pdfViewerEnabled,
          navigator.doNotTrack,
          navigator.deviceMemory,
          navigator.plugins.length,
          navigator.mimeTypes.length,
          navigator.connection.effectiveType,
          navigator.connection.rtt,
          navigator.connection.downlink,
          navigator.connection.saveData
        ].join("\u001f")
        "#,
    );
    let expected = [
        "Edge",
        "20260725",
        "Microsoft Corporation",
        "0",
        "12",
        "false",
        "MozillaCustom",
        "NetscapeCustom",
        "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36 Edg/149.0.0.0",
        "Win64",
        "GeckoCustom",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36 Edg/149.0.0.0",
        "en-US",
        "en-US,en",
        "true",
        "true",
        "false",
        "false",
        "false",
        "1",
        "4",
        "0",
        "0",
        "3g",
        "150",
        "0.8",
        "true",
    ]
    .join("\u{1f}");
    assert_eq!(values, expected);
}

#[test]
fn ua_client_hints_use_the_same_fingerprint_for_low_and_high_entropy_values() {
    let mut runtime =
        EdgeRuntime::with_fingerprint(custom_fingerprint()).expect("custom Edge runtime");
    let low_entropy = text(
        &mut runtime,
        r#"
        [
          navigator.userAgentData.brands
            .map(value => value.brand + ":" + value.version).join(","),
          navigator.userAgentData.mobile,
          navigator.userAgentData.platform,
          navigator.userAgentData.toJSON().platform
        ].join("|")
        "#,
    );
    assert_eq!(
        low_entropy,
        "Not/A)Brand:8,Chromium:149,Microsoft Edge:149|false|Windows|Windows"
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"
            globalThis.navigatorHighEntropy = "pending";
            navigator.userAgentData.getHighEntropyValues([
              "architecture",
              "bitness",
              "model",
              "platformVersion",
              "uaFullVersion",
              "fullVersionList",
              "wow64",
              "formFactors"
            ]).then(value => {
              navigatorHighEntropy = [
                value.architecture,
                value.bitness,
                value.model,
                value.platformVersion,
                value.uaFullVersion,
                value.fullVersionList
                  .map(item => item.brand + ":" + item.version).join(","),
                value.wow64,
                value.formFactors.join(",")
              ].join("|");
            });
            "scheduled"
            "#,
        ),
        "scheduled"
    );
    assert_eq!(
        text(&mut runtime, "navigatorHighEntropy"),
        "x86|64|Edge Desktop|20.0.0|149.0.7753.0|Not/A)Brand:8.0.0.0,Chromium:149.0.7753.0,Microsoft Edge:149.0.7753.0|true|Desktop,Automotive"
    );
}

#[test]
fn worker_navigator_uses_the_window_fingerprint_and_same_object_languages() {
    let mut runtime =
        EdgeRuntime::with_fingerprint(custom_fingerprint()).expect("custom Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const source = `
                postMessage([
                  navigator.userAgent,
                  navigator.appVersion,
                  navigator.platform,
                  navigator.product,
                  navigator.language,
                  navigator.languages.join(","),
                  navigator.languages === navigator.languages,
                  Object.isFrozen(navigator.languages),
                  navigator.hardwareConcurrency,
                  navigator.deviceMemory,
                  navigator.onLine,
                  navigator.userAgentData.platform,
                  navigator.connection.effectiveType,
                  navigator.connection.rtt,
                  navigator.connection.downlink,
                  navigator.connection.saveData
                ].join("\\u001f"));
              `;
              const worker = new Worker(
                "data:text/javascript," + encodeURIComponent(source)
              );
              globalThis.customWorkerNavigator = "pending";
              worker.onmessage = event =>
                globalThis.customWorkerNavigator = event.data;
              return "scheduled";
            })()
            "#,
        ),
        "scheduled"
    );
    let expected = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36 Edg/149.0.0.0",
        "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36 Edg/149.0.0.0",
        "Win64",
        "GeckoCustom",
        "en-US",
        "en-US,en",
        "true",
        "true",
        "12",
        "4",
        "false",
        "Windows",
        "3g",
        "150",
        "0.8",
        "true",
    ]
    .join("\u{1f}");
    assert_eq!(text(&mut runtime, "customWorkerNavigator"), expected);
}

#[test]
fn invalid_navigator_fingerprints_are_rejected_before_v8_is_created() {
    let mut fingerprint = custom_fingerprint();
    fingerprint.navigator.languages[0] = "fr-FR".to_owned();
    let error = EdgeRuntime::with_fingerprint(fingerprint)
        .err()
        .expect("invalid fingerprint must fail");
    assert!(error.contains("navigator.languages"), "{error}");
}

#[test]
fn proxy_trace_preserves_navigator_shapes_and_records_configured_accesses() {
    let mut runtime =
        EdgeRuntime::with_fingerprint(custom_fingerprint()).expect("custom Edge runtime");
    runtime.enable_proxy_trace().expect("enable Proxy trace");
    let shape = text(
        &mut runtime,
        r#"
        [
          navigator.userAgent.includes("Edg/149.0.0.0"),
          navigator.userAgentData.platform,
          navigator.connection.effectiveType,
          Function.prototype.toString.call(
            Object.getOwnPropertyDescriptor(
              Navigator.prototype,
              "userAgent"
            ).get
          ).includes("[native code]"),
          Object.getPrototypeOf(navigator) === Navigator.prototype,
          navigator.languages === navigator.languages
        ].join("|")
        "#,
    );
    assert_eq!(shape, "true|Windows|3g|true|true|true");
    let trace = runtime.proxy_trace();
    assert!(
        trace.iter().any(|entry| {
            entry.operation == "get" && entry.api.ends_with(".navigator.userAgent")
        })
    );
    assert!(trace.iter().any(|entry| {
        entry.operation == "get" && entry.api.ends_with(".navigator.userAgentData.platform")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get" && entry.api.ends_with(".navigator.connection.effectiveType")
    }));
}
