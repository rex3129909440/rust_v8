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
fn browser_surface_version_range_is_validated_from_user_agent() {
    for major in 140..=151 {
        let mut fingerprint = EdgeFingerprint::default();
        fingerprint.navigator.user_agent = format!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{major}.0.0.0 Safari/537.36"
        );
        fingerprint.navigator.app_version = fingerprint
            .navigator
            .user_agent
            .strip_prefix("Mozilla/")
            .unwrap()
            .to_owned();
        assert!(fingerprint.validate().is_ok(), "major {major}");
    }
    for major in [139, 152] {
        let mut fingerprint = EdgeFingerprint::default();
        fingerprint.navigator.user_agent = format!("Chrome/{major}.0.0.0");
        fingerprint.navigator.app_version = fingerprint.navigator.user_agent.clone();
        assert!(fingerprint.validate().is_err(), "major {major}");
    }
}

#[test]
fn custom_ua_version_synchronizes_only_the_default_ua_client_hints() {
    let mut default_hints = chromium_fingerprint(145);
    default_hints.navigator.user_agent = default_hints
        .navigator
        .user_agent
        .replace("145.0.0.0", "145.0.7632.117");
    default_hints.navigator.app_version = default_hints
        .navigator
        .user_agent
        .strip_prefix("Mozilla/")
        .unwrap()
        .to_owned();
    let mut runtime = EdgeRuntime::with_fingerprint(default_hints).unwrap();
    assert_eq!(
        text(
            &mut runtime,
            r#"[
              navigator.userAgentData.brands.find(value => value.brand === "Chromium").version,
              navigator.userAgentData.getHighEntropyValues(["uaFullVersion"]).then(
                value => value.uaFullVersion
              )
            ][0]"#,
        ),
        "145"
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"navigator.userAgentData.getHighEntropyValues(["uaFullVersion"])
              .then(value => value.uaFullVersion)"#,
        ),
        "145.0.7632.117"
    );

    let mut explicit_hints = chromium_fingerprint(145);
    explicit_hints.navigator.user_agent_data.ua_full_version = "custom-version".to_owned();
    let mut runtime = EdgeRuntime::with_fingerprint(explicit_hints).unwrap();
    assert_eq!(
        text(
            &mut runtime,
            r#"navigator.userAgentData.getHighEntropyValues(["uaFullVersion"])
              .then(value => value.uaFullVersion)"#,
        ),
        "custom-version"
    );
}

fn chromium_fingerprint(major: u16) -> EdgeFingerprint {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.navigator.user_agent = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/{major}.0.0.0 Safari/537.36"
    );
    fingerprint.navigator.app_version = fingerprint
        .navigator
        .user_agent
        .strip_prefix("Mozilla/")
        .unwrap()
        .to_owned();
    fingerprint
}

fn android_chromium_fingerprint(major: u16) -> EdgeFingerprint {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.navigator.user_agent = format!(
        "Mozilla/5.0 (Linux; Android 11; Pixel 4) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/{major}.0.0.0 Mobile Safari/537.36"
    );
    fingerprint.navigator.app_version = fingerprint
        .navigator
        .user_agent
        .strip_prefix("Mozilla/")
        .unwrap()
        .to_owned();
    fingerprint
}

#[test]
fn android_user_agent_converts_only_untouched_desktop_platform_defaults() {
    let mut runtime = EdgeRuntime::with_fingerprint(android_chromium_fingerprint(151)).unwrap();
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const context = new AudioContext();
              return [
                navigator.platform,
                navigator.hardwareConcurrency,
                navigator.deviceMemory,
                navigator.maxTouchPoints,
                navigator.pdfViewerEnabled,
                navigator.plugins.length,
                navigator.mimeTypes.length,
                navigator.userAgentData.mobile,
                navigator.userAgentData.platform,
                matchMedia("(pointer: coarse)").matches,
                matchMedia("(hover: hover)").matches,
                context.baseLatency,
                PerformanceObserver.supportedEntryTypes.join(",")
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "Linux armv81|8|4|5|false|0|0|true|Android|true|false|",
            "0.0026666666666666666|element,event,first-input,interaction-contentful-paint,",
            "largest-contentful-paint,layout-shift,long-animation-frame,longtask,mark,measure,",
            "navigation,paint,resource,soft-navigation,visibility-state"
        )
    );

    let mut explicit = android_chromium_fingerprint(151);
    explicit.navigator.platform = "ExplicitPlatform".to_owned();
    explicit.navigator.hardware_concurrency = 16;
    explicit.navigator.device_memory_gb = 12.0;
    explicit.navigator.max_touch_points = 2;
    explicit.navigator.pdf_viewer_enabled = true;
    explicit.navigator.user_agent_data.platform = "ExplicitCH".to_owned();
    explicit.navigator.user_agent_data.architecture = "arm".to_owned();
    explicit.navigator.user_agent_data.bitness = "64".to_owned();
    let mut runtime = EdgeRuntime::with_fingerprint(explicit).unwrap();
    assert_eq!(
        text(
            &mut runtime,
            "navigator.userAgentData.getHighEntropyValues(['architecture','bitness']).then(value =>\
             [navigator.platform,navigator.hardwareConcurrency,navigator.deviceMemory,\
              navigator.maxTouchPoints,navigator.pdfViewerEnabled,\
              navigator.userAgentData.platform,value.architecture,value.bitness].join('|'))",
        ),
        "ExplicitPlatform|16|12|2|true|ExplicitCH|arm|64"
    );
}

#[test]
fn android_network_information_matches_mobile_https_members_and_profile_values() {
    let mut fingerprint = android_chromium_fingerprint(151);
    fingerprint.navigator.network.effective_type = "3g".to_owned();
    fingerprint.navigator.network.rtt = 275;
    fingerprint.navigator.network.downlink = 0.85;
    fingerprint.navigator.network.save_data = true;
    fingerprint.navigator.network.connection_type = "cellular".to_owned();
    fingerprint.navigator.network.downlink_max = 42.0;
    let mut runtime = EdgeRuntime::with_fingerprint(fingerprint).unwrap();
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const connection = navigator.connection;
              const handler = () => {};
              connection.ontypechange = handler;
              return [
                Object.getOwnPropertyNames(NetworkInformation.prototype).join(","),
                connection.effectiveType,
                connection.rtt,
                connection.downlink,
                connection.saveData,
                connection.type,
                connection.downlinkMax,
                connection.ontypechange === handler,
                NetworkInformation.toString()
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "onchange,effectiveType,rtt,downlink,saveData,type,downlinkMax,ontypechange,constructor|",
            "3g|275|0.85|true|cellular|42|true|function NetworkInformation() { [native code] }"
        )
    );

    // These legacy members are Android-only. A loose JSON probe can make a
    // missing property look like null after transport, so use membership and
    // the complete prototype evidence rather than serialized values.
    let mut desktop = EdgeRuntime::with_fingerprint(chromium_fingerprint(151)).unwrap();
    assert_eq!(
        text(
            &mut desktop,
            "['type','downlinkMax','ontypechange'].map(name => name in navigator.connection).join(',')",
        ),
        "false,false,false"
    );
}

#[test]
fn chromium_140_through_150_switch_window_iframe_and_navigator_order() {
    const EXPECTED: &[(u16, usize, usize, &str)] = &[
        (140, 1196, 83, "vendorSub"),
        (141, 1200, 83, "vendorSub"),
        (142, 1202, 83, "vendorSub"),
        (143, 1204, 83, "vendorSub"),
        (144, 1208, 83, "vendorSub"),
        (145, 1213, 83, "vendorSub"),
        (146, 1219, 83, "vendorSub"),
        (147, 1230, 83, "vendorSub"),
        (148, 1231, 83, "vendorSub"),
        (149, 1232, 83, "vendorSub"),
        (150, 1232, 83, "vendorSub"),
        (151, 1236, 83, "vendorSub"),
    ];
    for &(major, window_count, navigator_count, first_navigator) in EXPECTED {
        let mut runtime = EdgeRuntime::with_fingerprint(chromium_fingerprint(major))
            .unwrap_or_else(|error| panic!("Chromium {major} runtime: {error}"));
        let value = text(
            &mut runtime,
            r#"
            (() => {
              const before = Object.getOwnPropertyNames(window);
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              const top = Object.getOwnPropertyNames(window).filter(name => name !== "0");
              const child = Object.getOwnPropertyNames(frame.contentWindow);
              const navigator = Object.getOwnPropertyNames(Navigator.prototype);
              const childNavigator = Object.getOwnPropertyNames(
                frame.contentWindow.Navigator.prototype
              );
              return [
                before.length,
                child.length,
                JSON.stringify(top) === JSON.stringify(child),
                navigator.length,
                navigator[0],
                JSON.stringify(navigator) === JSON.stringify(childNavigator),
                "LanguageModel" in window,
                Object.prototype.hasOwnProperty.call(SharedStorage.prototype, "get")
                ,child.filter(name => !top.includes(name)).join(",")
                ,top.filter(name => !child.includes(name)).join(",")
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            value,
            format!(
                "{window_count}|{window_count}|true|{navigator_count}|{first_navigator}|true|{}|{}||",
                major >= 148,
                major <= 147,
            ),
            "Chromium {major} surface"
        );
    }
}

#[test]
fn chromium_140_through_150_window_names_and_descriptors_match_evidence() {
    for major in 140..=151 {
        let mut runtime = EdgeRuntime::with_fingerprint(chromium_fingerprint(major)).unwrap();
        let observed = text(
            &mut runtime,
            r#"
            (() => {
              const hash = text => {
                let value = 2166136261;
                for (let index = 0; index < text.length; index += 1) {
                  value = Math.imul(value ^ text.charCodeAt(index), 16777619);
                }
                return value >>> 0;
              };
              const names = Object.getOwnPropertyNames(window);
              const descriptors = names.map(name => {
                const descriptor = Object.getOwnPropertyDescriptor(window, name);
                return name + ":" + ("value" in descriptor ? "d" : "a") + ":" +
                  Number(descriptor.enumerable) + Number(descriptor.configurable) +
                  Number(Boolean(descriptor.writable)) + ":" +
                  Number(Boolean(descriptor.get)) + Number(Boolean(descriptor.set));
              });
              return [names.join("\u001f"), hash(descriptors.join("\u001f"))];
            })()
            "#,
        );
        let expected_names = crate::browser_surface_data::window_names(major).join("\u{1f}");
        let separator = observed.rfind(',').expect("array result separator");
        // Evaluation renders arrays using comma separators; names contain no commas.
        let actual_names = &observed[0..separator];
        let actual_hash = observed[separator + 1..].parse::<u32>().unwrap();
        assert_eq!(
            actual_names, expected_names,
            "Chromium {major} Window names"
        );
        assert_eq!(
            actual_hash,
            crate::browser_surface_data::expected_window_descriptor_hash(major),
            "Chromium {major} Window descriptors",
        );
    }
}

#[test]
fn chromium_140_through_150_versioned_prototypes_statics_and_objects_match_tables() {
    for major in 140..=151 {
        let mut runtime = EdgeRuntime::with_fingerprint(chromium_fingerprint(major)).unwrap();
        assert_eq!(
            text(
                &mut runtime,
                "Object.getOwnPropertyNames(Document).join(',')"
            ),
            crate::browser_surface_data::constructor_static_names("Document", major)
                .unwrap()
                .join(","),
            "Chromium {major} static Document",
        );
        assert_eq!(
            text(
                &mut runtime,
                "Object.getOwnPropertyNames(Iterator).join(',')"
            ),
            crate::browser_surface_data::constructor_static_names("Iterator", major)
                .unwrap()
                .join(","),
            "Chromium {major} static Iterator",
        );
        assert_eq!(
            text(&mut runtime, "Object.getOwnPropertyNames(Math).join(',')"),
            crate::browser_surface_data::global_object_names("Math", major)
                .unwrap()
                .join(","),
            "Chromium {major} Math",
        );
        assert_eq!(
            text(
                &mut runtime,
                "Object.getOwnPropertyNames(GPUTextureUsage).join(',')",
            ),
            crate::browser_surface_data::global_object_names("GPUTextureUsage", major)
                .unwrap()
                .join(","),
            "Chromium {major} GPUTextureUsage",
        );
    }
}

#[test]
fn chromium_140_through_150_complete_versioned_surface_hash_matches_evidence() {
    for major in 140..=151 {
        let mut runtime = EdgeRuntime::with_fingerprint(chromium_fingerprint(major)).unwrap();
        let observed = text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              const hash = (realm, reflectKeys) => {
                const keyName = key => typeof key === "symbol" ?
                  "@@" + String(key.description || "") : key;
                const ownKeys = value => (reflectKeys ?
                  Reflect.ownKeys(value).map(keyName) :
                  Object.getOwnPropertyNames(value));
                const records = [];
                const names = Object.getOwnPropertyNames(realm).sort();
                for (const owner of names) {
                  // A numeric named-frame property is not a browser interface/object
                  // surface. The evidence collector excludes it from this matrix too.
                  if (owner === "0") continue;
                  const descriptor = Object.getOwnPropertyDescriptor(realm, owner);
                  if (!descriptor || !("value" in descriptor)) continue;
                  const value = descriptor.value;
                  if (typeof value === "function") {
                    if (value.prototype) records.push(
                      "constructorPrototypes:" + owner + ":" +
                      ownKeys(value.prototype).join("\u001e")
                    );
                    records.push(
                      "constructorStatics:" + owner + ":" +
                      ownKeys(value).join("\u001e")
                    );
                  } else if (value && typeof value === "object" && value !== realm) {
                    const objectNames = Object.getOwnPropertyNames(value);
                    if (objectNames.length) records.push(
                      "globalObjects:" + owner + ":" + ownKeys(value).join("\u001e")
                    );
                  }
                }
                records.sort();
                let result = 2166136261;
                const input = records.join("\u001f");
                for (let index = 0; index < input.length; index += 1) {
                  result = Math.imul(result ^ input.charCodeAt(index), 16777619);
                }
                return String(result >>> 0);
              };
              return [
                hash(globalThis, false),
                hash(frame.contentWindow, false),
                hash(globalThis, true),
                hash(frame.contentWindow, true)
              ].join("|");
            })()
            "#,
        );
        let expected = crate::browser_surface_data::expected_versioned_surface_hash(major);
        let expected_keys =
            crate::browser_surface_data::expected_versioned_surface_keys_hash(major);
        assert_eq!(
            observed,
            format!("{expected}|{expected}|{expected_keys}|{expected_keys}"),
            "Chromium {major} complete top/iframe constructor, object, and symbol surface"
        );
    }
}

#[test]
#[ignore = "developer diagnostic for generated browser surface allowlists"]
fn diagnose_chromium_140_complete_surface_owners() {
    let mut runtime = EdgeRuntime::with_fingerprint(chromium_fingerprint(140)).unwrap();
    let observed = text(
        &mut runtime,
        r#"
        (() => {
          const keyName = key => typeof key === "symbol" ?
            "@@" + String(key.description || "") : key;
          const records = [];
          for (const owner of Object.getOwnPropertyNames(globalThis).sort()) {
            const descriptor = Object.getOwnPropertyDescriptor(globalThis, owner);
            if (!descriptor || !("value" in descriptor)) continue;
            const value = descriptor.value;
            if (typeof value === "function") {
              if (value.prototype) records.push([
                "constructorPrototypes", owner,
                Object.getOwnPropertyNames(value.prototype),
                Reflect.ownKeys(value.prototype).map(keyName)
              ]);
              records.push([
                "constructorStatics", owner,
                Object.getOwnPropertyNames(value), Reflect.ownKeys(value).map(keyName)
              ]);
            } else if (value && typeof value === "object" && value !== globalThis &&
                       Object.getOwnPropertyNames(value).length) {
              records.push([
                "globalObjects", owner,
                Object.getOwnPropertyNames(value), Reflect.ownKeys(value).map(keyName)
              ]);
            }
          }
          return JSON.stringify(records);
        })()
        "#,
    );
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("build/chromium-version-surfaces/sandbox-140-owner-diagnostic.json");
    std::fs::write(&path, observed).unwrap();
    panic!("wrote {}", path.display());
}

#[test]
fn android_chromium_140_through_151_window_iframe_and_navigator_match_https_evidence() {
    const WINDOW_COUNTS: [usize; 12] = [
        1191, 1195, 1196, 1199, 1204, 1207, 1213, 1226, 1226, 1232, 1230, 1234,
    ];
    for (offset, expected_count) in WINDOW_COUNTS.into_iter().enumerate() {
        let major = 140 + offset as u16;
        let browser = crate::browser_version::BrowserVersion::from_user_agent(
            &android_chromium_fingerprint(major).navigator.user_agent,
        )
        .unwrap();
        let mut runtime = EdgeRuntime::with_fingerprint(android_chromium_fingerprint(major))
            .unwrap_or_else(|error| panic!("Android Chromium {major} runtime: {error}"));
        let observed = text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              const top = Object.getOwnPropertyNames(window).filter(name => name !== "0");
              const child = Object.getOwnPropertyNames(frame.contentWindow);
              const nav = Object.getOwnPropertyNames(Navigator.prototype);
              const childNav = Object.getOwnPropertyNames(frame.contentWindow.Navigator.prototype);
              return [
                top.length,
                child.length,
                JSON.stringify(top) === JSON.stringify(child),
                nav.length,
                JSON.stringify(nav) === JSON.stringify(childNav),
                typeof navigator.contacts,
                typeof navigator.modelContext,
                typeof navigator.cookieDeprecationLabel,
                typeof window.orientation,
                "capture" in HTMLInputElement.prototype,
                "ontouchstart" in HTMLElement.prototype,
                child.filter(name => !top.includes(name)).join(","),
                top.filter(name => !child.includes(name)).join(",")
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            observed,
            format!(
                "{expected_count}|{expected_count}|true|{}|true|object|{}|{}|number|true|true||",
                crate::browser_surface::navigator_names(browser).len(),
                if major >= 149 { "object" } else { "undefined" },
                if major <= 143 { "object" } else { "undefined" },
            ),
            "Android Chromium {major} HTTPS surface",
        );
    }
}

#[test]
fn android_chromium_140_through_151_descriptors_and_complete_surface_match_https_evidence() {
    for major in 140..=151 {
        let mut runtime = EdgeRuntime::with_fingerprint(android_chromium_fingerprint(major))
            .unwrap_or_else(|error| panic!("Android Chromium {major} runtime: {error}"));
        let observed = text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              const fnv = input => {
                let value = 2166136261;
                for (let index = 0; index < input.length; index += 1) {
                  value = Math.imul(value ^ input.charCodeAt(index), 16777619);
                }
                return value >>> 0;
              };
              const descriptors = realm => Object.getOwnPropertyNames(realm)
                .filter(name => name !== "0").map(name => {
                const descriptor = Object.getOwnPropertyDescriptor(realm, name);
                return name + ":" + ("value" in descriptor ? "d" : "a") + ":" +
                  Number(descriptor.enumerable) + Number(descriptor.configurable) +
                  Number(Boolean(descriptor.writable)) + ":" +
                  Number(Boolean(descriptor.get)) + Number(Boolean(descriptor.set));
              });
              const surface = (realm, reflectKeys) => {
                const keyName = key => typeof key === "symbol" ?
                  "@@" + String(key.description || "") : key;
                const ownKeys = value => (reflectKeys ? Reflect.ownKeys(value) :
                  Object.getOwnPropertyNames(value)).map(keyName);
                const records = [];
                for (const owner of Object.getOwnPropertyNames(realm).sort()) {
                  if (owner === "0") continue;
                  const descriptor = Object.getOwnPropertyDescriptor(realm, owner);
                  if (!descriptor || !("value" in descriptor)) continue;
                  const value = descriptor.value;
                  if (typeof value === "function") {
                    if (value.prototype) records.push(
                      "constructorPrototypes:" + owner + ":" + ownKeys(value.prototype).join("\u001e")
                    );
                    records.push("constructorStatics:" + owner + ":" + ownKeys(value).join("\u001e"));
                  } else if (value && typeof value === "object" && value !== realm &&
                             Object.getOwnPropertyNames(value).length) {
                    records.push("globalObjects:" + owner + ":" + ownKeys(value).join("\u001e"));
                  }
                }
                records.sort();
                return fnv(records.join("\u001f"));
              };
              return [
                fnv(descriptors(globalThis).join("\u001f")),
                fnv(descriptors(frame.contentWindow).join("\u001f")),
                surface(globalThis, false),
                surface(frame.contentWindow, false),
                surface(globalThis, true),
                surface(frame.contentWindow, true)
              ].join("|");
            })()
            "#,
        );
        let descriptor =
            crate::browser_android_surface_data::expected_window_descriptor_hash(major);
        let surface = crate::browser_android_surface_data::expected_versioned_surface_hash(major);
        let keys = crate::browser_android_surface_data::expected_versioned_surface_keys_hash(major);
        assert_eq!(
            observed,
            format!("{descriptor}|{descriptor}|{surface}|{surface}|{keys}|{keys}"),
            "Android Chromium {major} HTTPS descriptors and complete surface",
        );
    }
}

#[test]
fn chromium_140_shared_storage_get_matches_versioned_edge_contract() {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.navigator.user_agent = "Mozilla/5.0 Chrome/140.0.0.0 Safari/537.36".to_owned();
    fingerprint.navigator.app_version = "5.0 Chrome/140.0.0.0 Safari/537.36".to_owned();
    let mut runtime = EdgeRuntime::new_with_fingerprint(fingerprint).expect("Chrome 140 runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const descriptor = Object.getOwnPropertyDescriptor(SharedStorage.prototype, "get");
              return [
                typeof descriptor.value,
                descriptor.value.name,
                descriptor.value.length,
                descriptor.enumerable,
                descriptor.configurable,
                Function.prototype.toString.call(descriptor.value)
              ].join("|");
            })()
            "#,
        ),
        "function|get|1|true|true|function get() { [native code] }"
    );
    runtime
        .evaluate(
            r#"
            globalThis.sharedStorage140 = [];
            sharedStorage.get().then(
              value => sharedStorage140.push(["missing", "ok", value]),
              error => sharedStorage140.push(["missing", error.name, error.message])
            );
            sharedStorage.get("key").then(
              value => sharedStorage140.push(["normal", "ok", value]),
              error => sharedStorage140.push(["normal", error.name, error.message])
            );
            SharedStorage.prototype.get.call({}, "key").then(
              value => sharedStorage140.push(["illegal", "ok", value]),
              error => sharedStorage140.push(["illegal", error.name, error.message])
            );
            "#,
        )
        .expect("schedule SharedStorage promises");
    assert_eq!(
        text(&mut runtime, "JSON.stringify(sharedStorage140)"),
        r#"[["missing","TypeError","Failed to execute 'get' on 'SharedStorage': 1 argument required, but only 0 present."],["normal","OperationError","Cannot call get() outside of a fenced frame."],["illegal","TypeError","Failed to execute 'get' on 'SharedStorage': Illegal invocation"]]"#
    );
}

#[test]
fn chromium_150_does_not_expose_removed_shared_storage_get() {
    let mut runtime = EdgeRuntime::new().expect("Chrome 150 runtime");
    assert_eq!(
        text(
            &mut runtime,
            "String(Object.prototype.hasOwnProperty.call(SharedStorage.prototype, 'get'))",
        ),
        "false"
    );
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

#[test]
fn navigator_prototype_uses_evidenced_chromium_148_property_order() {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.navigator.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.7778.217 Safari/537.36"
        .to_owned();
    fingerprint.navigator.app_version = fingerprint
        .navigator
        .user_agent
        .strip_prefix("Mozilla/")
        .expect("Mozilla prefix")
        .to_owned();
    let mut runtime =
        EdgeRuntime::with_fingerprint(fingerprint).expect("Chromium 148 navigator order runtime");
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
            "webkitPersistentStorage,hardwareConcurrency,cookieEnabled,",
            "appCodeName,appName,appVersion,platform,product,userAgent,language,",
            "languages,onLine,webdriver,plugins,mimeTypes,pdfViewerEnabled,",
            "connection,getGamepads,javaEnabled,sendBeacon,vibrate,",
            "windowControlsOverlay,constructor"
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
