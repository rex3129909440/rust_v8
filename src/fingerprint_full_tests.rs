use crate::{EdgeRuntime, EdgeRuntimeOptions, Evaluation, FontMetricFingerprint};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

fn configured_options() -> EdgeRuntimeOptions {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.id = "edge-fingerprint-propagation".to_owned();
    options.fingerprint.locale.locale = "fr-FR".to_owned();
    options.fingerprint.locale.time_zone = "Europe/Paris".to_owned();
    options.fingerprint.locale.time_zone_offset_minutes = -120;
    options.fingerprint.navigator.language = "fr-FR".to_owned();
    options.fingerprint.navigator.languages = vec!["fr-FR".to_owned(), "fr".to_owned()];
    options.fingerprint.screen.width = 2560;
    options.fingerprint.screen.height = 1440;
    options.fingerprint.screen.avail_width = 2520;
    options.fingerprint.screen.avail_height = 1380;
    options.fingerprint.screen.avail_left = 20;
    options.fingerprint.screen.avail_top = 10;
    options.fingerprint.screen.color_depth = 30;
    options.fingerprint.screen.pixel_depth = 30;
    options.fingerprint.screen.viewport_width = 1440.0;
    options.fingerprint.screen.viewport_height = 900.0;
    options.fingerprint.screen.outer_width = 1500.0;
    options.fingerprint.screen.outer_height = 980.0;
    options.fingerprint.screen.screen_x = 37.0;
    options.fingerprint.screen.screen_y = 41.0;
    options.fingerprint.screen.device_pixel_ratio = 1.5;
    options.fingerprint.rendering.canvas.text_width_scale = 1.4;
    options.fingerprint.rendering.canvas.data_url_salt = "render-profile".to_owned();
    options.fingerprint.rendering.webgl.vendor = "Edge GL Vendor".to_owned();
    options.fingerprint.rendering.webgl.renderer = "Edge GL Renderer".to_owned();
    options.fingerprint.rendering.webgl.unmasked_vendor = "Edge GPU Vendor".to_owned();
    options.fingerprint.rendering.webgl.unmasked_renderer = "Edge GPU Renderer".to_owned();
    options.fingerprint.rendering.webgl.webgl1_extensions = vec![
        "EXT_texture_filter_anisotropic".to_owned(),
        "WEBGL_debug_renderer_info".to_owned(),
    ];
    options.fingerprint.rendering.webgl.webgl2_extensions = vec![
        "EXT_texture_filter_anisotropic".to_owned(),
        "WEBGL_debug_renderer_info".to_owned(),
    ];
    options.fingerprint.rendering.webgl.max_texture_size = 8192;
    options.fingerprint.rendering.webgl.max_anisotropy = 8.0;
    options.fingerprint.rendering.webgpu.vendor = "Edge GPU".to_owned();
    options.fingerprint.rendering.webgpu.architecture = "Custom D3D12".to_owned();
    options.fingerprint.rendering.webgpu.device = "Device 42".to_owned();
    options.fingerprint.rendering.webgpu.description = "Configured adapter".to_owned();
    options.fingerprint.rendering.webgpu.features = vec![
        "bgra8unorm-storage".to_owned(),
        "timestamp-query".to_owned(),
    ];
    options
        .fingerprint
        .rendering
        .webgpu
        .max_texture_dimension_2d = 4096;
    options.fingerprint.rendering.audio.sample_rate = 96_000.0;
    options.fingerprint.rendering.audio.max_channel_count = 6;
    options.fingerprint.rendering.audio.base_latency = 0.004;
    options.fingerprint.rendering.audio.output_latency = 0.017;
    options
}

#[test]
fn font_css_and_webgpu_platform_differences_are_profile_driven() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.fonts.families = vec!["Configured UI".to_owned()];
    options.fingerprint.fonts.metrics = vec![FontMetricFingerprint {
        family: "Configured UI".to_owned(),
        width_scale: 0.5,
        monospace: false,
    }];
    options.fingerprint.css.input_text =
        "display:inline-block;box-sizing:border-box;width:77px;height:9px;padding:0;border-width:0"
            .to_owned();
    options.fingerprint.rendering.webgpu.developer_features = false;
    options.fingerprint.rendering.webgpu.subgroup_min_size = 16;
    options.fingerprint.rendering.webgpu.subgroup_max_size = 16;
    options.fingerprint.rendering.webgpu.is_fallback_adapter = false;
    let mut runtime = EdgeRuntime::with_options(options).expect("configured platform profile");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const canvas = document.createElement("canvas");
              const context = canvas.getContext("2d");
              context.font = "10px sans-serif";
              const fallback = context.measureText("abcd").width;
              context.font = '10px "Configured UI"';
              const configured = context.measureText("abcd").width;
              const input = document.createElement("input");
              document.body.appendChild(input);
              const rect = input.getBoundingClientRect();
              return [(configured / fallback).toFixed(3), rect.width, rect.height].join("|");
            })()
            "#,
        ),
        "0.500|77|9"
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"
            navigator.gpu.requestAdapter().then(adapter => [
              adapter.info.device,
              adapter.info.description,
              adapter.info.subgroupMinSize,
              adapter.info.subgroupMaxSize,
              adapter.info.isFallbackAdapter
            ].join("|"))
            "#,
        ),
        "||16|16|false"
    );
}

#[test]
fn zero_sized_window_viewport_is_independent_from_physical_screen() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.viewport_width = 0.0;
    options.fingerprint.screen.viewport_height = 0.0;
    options.fingerprint.screen.outer_width = 0.0;
    options.fingerprint.screen.outer_height = 0.0;
    let mut runtime = EdgeRuntime::with_options(options).expect("zero-sized window viewport");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              return [
                screen.width,
                screen.height,
                innerWidth,
                innerHeight,
                outerWidth,
                outerHeight,
                visualViewport.width,
                visualViewport.height,
                frame.contentWindow.innerWidth,
                frame.contentWindow.innerHeight,
                typeof matchMedia("(device-height: 720px)").matches,
                matchMedia("(device-height: 720px)").matches,
                matchMedia("(aspect-ratio: 16/9)").matches
              ].join("|");
            })()
            "#,
        ),
        "1280|720|0|0|0|0|0|0|0|0|boolean|true|true"
    );
}

#[test]
fn match_media_reads_viewport_screen_dpr_and_color_from_their_correct_surfaces() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.width = 1512;
    options.fingerprint.screen.height = 982;
    options.fingerprint.screen.viewport_width = 1440.0;
    options.fingerprint.screen.viewport_height = 900.0;
    options.fingerprint.screen.device_pixel_ratio = 2.0;
    options.fingerprint.screen.color_depth = 30;
    options.fingerprint.screen.pixel_depth = 30;
    let source = r#"
      (() => {
        const list = matchMedia("(0.250 <= aspect-ratio <= 4.000)");
        return [
          list instanceof MediaQueryList,
          typeof list.matches,
          list.matches,
          matchMedia("(aspect-ratio: 8/5)").matches,
          matchMedia("(device-width: 1512px)").matches,
          matchMedia("(device-height: 982px)").matches,
          matchMedia("(device-aspect-ratio: 1512/982)").matches,
          matchMedia("(resolution: 2dppx)").matches,
          matchMedia("(color: 10)").matches,
          matchMedia("(device-height: 900px)").matches
        ].join("|");
      })()
    "#;
    let expected = "true|boolean|true|true|true|true|true|true|true|false";
    let mut direct = EdgeRuntime::with_options(options.clone()).expect("direct media runtime");
    assert_eq!(text(&mut direct, source), expected);
    let mut traced = EdgeRuntime::with_options(options).expect("traced media runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn viewport_relative_input_rects_recompute_from_the_live_window_size() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.viewport_width = 800.0;
    options.fingerprint.screen.viewport_height = 600.0;
    options.fingerprint.screen.outer_width = 800.0;
    options.fingerprint.screen.outer_height = 600.0;
    let mut runtime = EdgeRuntime::with_options(options).expect("configured viewport runtime");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const fixed = document.createElement("input");
              const viewport = document.createElement("input");
              const percentage = document.createElement("input");
              viewport.style.cssText = "width:50vw;height:10vh";
              percentage.style.cssText = "width:50%;height:10vh";
              document.body.append(fixed, viewport, percentage);
              const before = [fixed, viewport, percentage].map(element => {
                const rect = element.getBoundingClientRect();
                return `${rect.width}x${rect.height}`;
              });
              resizeTo(1000, 500);
              const after = [fixed, viewport, percentage].map(element => {
                const rect = element.getBoundingClientRect();
                return `${rect.width}x${rect.height}`;
              });
              return [
                before.join(","),
                after.join(","),
                innerWidth,
                innerHeight,
                visualViewport.width,
                visualViewport.height
              ].join("|");
            })()
            "#,
        ),
        "177x21,408x66,400x66|177x21,508x56,500x56|1000|500|1000|500"
    );
}

#[test]
fn screen_locale_media_queries_canvas_webgl_and_audio_share_one_profile() {
    let mut runtime =
        EdgeRuntime::with_options(configured_options()).expect("configured Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const canvas = document.createElement("canvas");
          const twoD = canvas.getContext("2d");
          const gl = document.createElement("canvas").getContext("webgl");
          const debug = gl.getExtension("WEBGL_debug_renderer_info");
          const anisotropy = gl.getExtension("EXT_texture_filter_anisotropic");
          const audio = new AudioContext();
          const filter = audio.createBiquadFilter();
          const oscillator = audio.createOscillator();
          const magnitude = new Float32Array(1);
          const phase = new Float32Array(1);
          filter.getFrequencyResponse(
            new Float32Array([30000]),
            magnitude,
            phase
          );
          return [
            Intl.DateTimeFormat().resolvedOptions().locale,
            Intl.DateTimeFormat().resolvedOptions().timeZone,
            new Date().getTimezoneOffset(),
            new Date("2025-01-15T12:00:00Z").getTimezoneOffset(),
            new Date("2025-07-15T12:00:00Z").getTimezoneOffset(),
            navigator.language,
            navigator.languages.join(","),
            screen.width,
            screen.height,
            screen.availWidth,
            screen.availHeight,
            screen.availLeft,
            screen.availTop,
            screen.colorDepth,
            screen.pixelDepth,
            innerWidth,
            innerHeight,
            outerWidth,
            outerHeight,
            screenX,
            screenY,
            devicePixelRatio,
            matchMedia(
              "screen and (min-width: 1400px) and (resolution: 1.5dppx)"
            ).matches,
            matchMedia("(max-resolution: 143dpi)").matches,
            matchMedia("print, (orientation: landscape)").matches,
            twoD.measureText("abcd").width.toFixed(1),
            gl.getParameter(gl.VENDOR),
            gl.getParameter(gl.RENDERER),
            gl.getParameter(debug.UNMASKED_VENDOR_WEBGL),
            gl.getParameter(debug.UNMASKED_RENDERER_WEBGL),
            gl.getParameter(gl.MAX_TEXTURE_SIZE),
            gl.getParameter(anisotropy.MAX_TEXTURE_MAX_ANISOTROPY_EXT),
            gl.getSupportedExtensions().join(","),
            audio.sampleRate,
            audio.destination.maxChannelCount,
            audio.baseLatency,
            audio.outputLatency,
            filter.frequency.maxValue,
            oscillator.frequency.maxValue,
            Number.isFinite(magnitude[0]),
            Number.isFinite(phase[0])
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        answer,
        concat!(
            "fr-FR|Europe/Paris|-120|-60|-120|fr-FR|fr-FR,fr|",
            "2560|1440|2520|1380|20|10|30|30|",
            "1440|900|1500|980|37|41|1.5|",
            "true|false|true|32.0|",
            "Edge GL Vendor|Edge GL Renderer|Edge GPU Vendor|Edge GPU Renderer|",
            "8192|8|EXT_texture_filter_anisotropic,WEBGL_debug_renderer_info|",
            "96000|6|0.004|0.017|48000|48000|true|true"
        )
    );
}

#[test]
fn in_process_runtime_threads_restore_their_icu_defaults_before_evaluation() {
    let (ready_sender, ready_receiver) = std::sync::mpsc::channel();
    let (evaluate_sender, evaluate_receiver) = std::sync::mpsc::channel();
    let french_thread = std::thread::spawn(move || {
        let mut runtime =
            EdgeRuntime::with_options(configured_options()).expect("configured French runtime");
        ready_sender.send(()).expect("signal French runtime");
        evaluate_receiver
            .recv()
            .expect("wait for default runtime creation");
        text(
            &mut runtime,
            "[Intl.DateTimeFormat().resolvedOptions().locale, Intl.DateTimeFormat().resolvedOptions().timeZone].join('|')",
        )
    });

    ready_receiver.recv().expect("wait for French runtime");
    let mut chinese = EdgeRuntime::new().expect("default Chinese runtime");
    evaluate_sender.send(()).expect("release French runtime");
    assert_eq!(
        text(
            &mut chinese,
            "[Intl.DateTimeFormat().resolvedOptions().locale, Intl.DateTimeFormat().resolvedOptions().timeZone].join('|')",
        ),
        "zh-CN|Asia/Shanghai"
    );
    assert_eq!(
        french_thread.join().expect("join French runtime"),
        "fr-FR|Europe/Paris"
    );
}

#[test]
fn canvas_text_metrics_use_font_spacing_alignment_and_empty_text_ink_bounds() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const context = document.createElement("canvas").getContext("2d");
          const baseline = context.measureText("abcd");
          context.font = "20px monospace";
          const monospace = context.measureText("Wi");
          context.font = "20px sans-serif";
          const proportional = context.measureText("Wi");
          context.font = "10px sans-serif";
          context.letterSpacing = "1px";
          context.wordSpacing = "2px";
          const spaced = context.measureText("a b");
          context.letterSpacing = "0px";
          context.wordSpacing = "0px";
          context.textAlign = "center";
          const centered = context.measureText("ab");
          context.textAlign = "right";
          const right = context.measureText("ab");
          const empty = context.measureText("");
          const fixed = value => value.toFixed(3);
          return [
            fixed(baseline.width),
            fixed(monospace.width),
            fixed(monospace.fontBoundingBoxAscent),
            fixed(proportional.width),
            fixed(spaced.width),
            fixed(centered.actualBoundingBoxLeft),
            fixed(centered.actualBoundingBoxRight),
            fixed(right.actualBoundingBoxLeft),
            fixed(right.actualBoundingBoxRight),
            fixed(empty.width),
            fixed(empty.actualBoundingBoxLeft),
            fixed(empty.actualBoundingBoxRight),
            fixed(empty.actualBoundingBoxAscent),
            fixed(empty.actualBoundingBoxDescent),
            fixed(empty.fontBoundingBoxAscent)
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        answer,
        concat!(
            "22.844|20.000|17.000|23.060|19.050|5.905|5.725|",
            "11.810|-0.180|0.000|0.000|0.000|0.000|0.000|12.000"
        )
    );
}

#[test]
fn webgl_parameter_vectors_use_edge_typed_arrays_and_invalid_enum_state() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const webgl = document.createElement("canvas").getContext("webgl");
          const webgl2 = document.createElement("canvas").getContext("webgl2");
          const tag = value => Object.prototype.toString.call(value);
          const invalidOne = webgl.getParameter(0xFFFFFFFF);
          const errorOne = webgl.getError();
          const clearedOne = webgl.getError();
          const invalidTwo = webgl2.getParameter(0xFFFFFFFF);
          const errorTwo = webgl2.getError();
          const clearedTwo = webgl2.getError();
          const debugOne = webgl.getExtension("WEBGL_debug_renderer_info");
          const debugTwo = webgl2.getExtension("WEBGL_debug_renderer_info");
          return [
            tag(webgl.getParameter(webgl.ALIASED_POINT_SIZE_RANGE)),
            tag(webgl.getParameter(webgl.ALIASED_LINE_WIDTH_RANGE)),
            tag(webgl.getParameter(webgl.DEPTH_RANGE)),
            tag(webgl.getParameter(webgl.VIEWPORT)),
            tag(webgl.getParameter(webgl.MAX_VIEWPORT_DIMS)),
            tag(webgl2.getParameter(webgl2.ALIASED_POINT_SIZE_RANGE)),
            tag(webgl2.getParameter(webgl2.ALIASED_LINE_WIDTH_RANGE)),
            tag(webgl2.getParameter(webgl2.MAX_VIEWPORT_DIMS)),
            webgl.getParameter(debugOne.UNMASKED_VENDOR_WEBGL),
            webgl.getParameter(debugOne.UNMASKED_RENDERER_WEBGL),
            webgl2.getParameter(debugTwo.UNMASKED_VENDOR_WEBGL),
            webgl2.getParameter(debugTwo.UNMASKED_RENDERER_WEBGL),
            invalidOne === null,
            errorOne,
            clearedOne,
            invalidTwo === null,
            errorTwo,
            clearedTwo
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        answer,
        concat!(
            "[object Float32Array]|[object Float32Array]|[object Float32Array]|",
            "[object Int32Array]|[object Int32Array]|",
            "[object Float32Array]|[object Float32Array]|[object Int32Array]|",
            "Google Inc. (Microsoft)|",
            "ANGLE (Microsoft, Microsoft Basic Render Driver (0x0000008C) ",
            "Direct3D11 vs_5_0 ps_5_0, D3D11)|",
            "Google Inc. (Microsoft)|",
            "ANGLE (Microsoft, Microsoft Basic Render Driver (0x0000008C) ",
            "Direct3D11 vs_5_0 ps_5_0, D3D11)|",
            "true|1280|0|true|1280|0"
        )
    );
}

#[test]
fn webgl_static_limits_match_the_captured_edge_150_parameter_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const parameters = [
            35724, 7936, 7937, 7938, 34921, 36347, 35660, 36348, 36349,
            33901, 33902, 34930, 3379, 35661, 34024, 3386, 34076, 2963,
            2968, 36004, 36005, 3408, 35658, 35371, 37154, 35377, 35659,
            35968, 35978, 35979, 35657, 35373, 37157, 35379, 35077, 34852,
            36063, 36183, 32883, 35071, 34045, 35375, 35376, 35374, 33000,
            33001, 36203
          ];
          const render = value => {
            if (value === null) return "null";
            if (ArrayBuffer.isView(value)) {
              return Object.prototype.toString.call(value).slice(8, -1) +
                ":" + Array.from(value).join(",");
            }
            return String(value);
          };
          const webgl = document.createElement("canvas").getContext("webgl");
          const webgl2 = document.createElement("canvas").getContext("webgl2");
          return [
            parameters.map(parameter => render(webgl.getParameter(parameter))).join("|"),
            parameters.map(parameter => render(webgl2.getParameter(parameter))).join("|")
          ].join("||");
        })()
        "#,
    );
    let webgl_nulls = std::iter::repeat_n("null", 25)
        .collect::<Vec<_>>()
        .join("|");
    assert_eq!(
        answer,
        format!(
            concat!(
                "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)|",
                "WebKit|WebKit WebGL|WebGL 1.0 (OpenGL ES 2.0 Chromium)|",
                "16|4096|16|30|1024|Float32Array:1,1024|Float32Array:1,1|",
                "16|16384|32|16384|Int32Array:32767,32767|16384|",
                "4294967295|4294967295|4294967295|4294967295|4|{}||",
                "WebGL GLSL ES 3.00 (OpenGL ES GLSL ES 3.0 Chromium)|",
                "WebKit|WebKit WebGL|WebGL 2.0 (OpenGL ES 3.0 Chromium)|",
                "16|4096|16|30|1024|Float32Array:1,1024|Float32Array:1,1|",
                "16|16384|32|16384|Int32Array:32767,32767|16384|",
                "4294967295|4294967295|4294967295|4294967295|4|",
                "16384|12|120|212992|120|4|120|4|4096|12|120|200704|",
                "7|8|8|16|2048|2048|2|24|65536|24|2147483647|",
                "2147483647|4294967294"
            ),
            webgl_nulls
        )
    );
}

#[test]
fn web_audio_profile_rejects_invalid_device_values() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.rendering.audio.base_latency = -0.001;
    assert!(EdgeRuntime::with_options(options).is_err());

    let mut options = EdgeRuntimeOptions::default();
    options
        .fingerprint
        .rendering
        .audio
        .frequency_noise_amplitude = 1.1;
    assert!(EdgeRuntime::with_options(options).is_err());
}

#[test]
fn web_audio_profile_survives_native_trace_without_shape_drift() {
    let mut options = configured_options();
    options.fingerprint.rendering.audio.channel_noise_amplitude = 0.000_01;
    let mut runtime = EdgeRuntime::with_options(options).expect("configured Edge runtime");
    runtime.enable_native_trace().expect("enable native trace");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const context = new AudioContext();
              const oscillator = context.createOscillator();
              oscillator.connect(context.destination);
              return [
                context.sampleRate,
                context.baseLatency,
                context.outputLatency,
                context.destination.maxChannelCount,
                oscillator.frequency.maxValue,
                Function.prototype.toString.call(oscillator.connect)
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "96000|0.004|0.017|6|48000|",
            "function connect() { [native code] }"
        )
    );
    let trace = runtime.native_trace();
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".connect") })
    );
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "get" && entry.api.ends_with(".sampleRate") })
    );
}

#[test]
fn webgpu_worker_and_iframe_observe_configured_fingerprint_values() {
    let mut runtime =
        EdgeRuntime::with_options(configured_options()).expect("configured Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.gpuFingerprintAnswer = "pending";
        navigator.gpu.requestAdapter().then(adapter => {
          gpuFingerprintAnswer = [
            adapter.info.vendor,
            adapter.info.architecture,
            adapter.info.device,
            adapter.info.description,
            adapter.features.has("bgra8unorm-storage"),
            adapter.features.has("timestamp-query"),
            adapter.limits.maxTextureDimension2D
          ].join("|");
        });

        const frame = document.createElement("iframe");
        frame.srcdoc = "<p>fingerprint</p>";
        document.body.appendChild(frame);
        globalThis.frameFingerprintAnswer = [
          frame.contentWindow.navigator.language,
          frame.contentWindow.navigator.languages.join(","),
          frame.contentWindow.screen.width,
          frame.contentWindow.innerWidth,
          frame.contentWindow.devicePixelRatio
        ].join("|");

        const source = `
          const canvas = new OffscreenCanvas(8, 8);
          const gl = canvas.getContext("webgl");
          postMessage([
            navigator.language,
            navigator.languages.join(","),
            gl.getParameter(gl.VENDOR),
            gl.getParameter(gl.RENDERER)
          ].join("|"));
        `;
        const worker = new Worker(
          "data:text/javascript," + encodeURIComponent(source)
        );
        globalThis.workerFingerprintAnswer = "pending";
        worker.onmessage = event => workerFingerprintAnswer = event.data;
        "#,
    );
    assert_eq!(
        text(&mut runtime, "gpuFingerprintAnswer"),
        "Edge GPU|Custom D3D12|Device 42|Configured adapter|true|true|4096"
    );
    assert_eq!(
        text(&mut runtime, "frameFingerprintAnswer"),
        "fr-FR|fr-FR,fr|2560|1440|1.5"
    );
    assert_eq!(
        text(&mut runtime, "workerFingerprintAnswer"),
        "fr-FR|fr-FR,fr|Edge GL Vendor|Edge GL Renderer"
    );
}
