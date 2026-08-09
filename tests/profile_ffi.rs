#![cfg(not(windows))]

use edge_sandbox::ffi::{
    EdgeSandboxBuffer, edge_sandbox_buffer_free, edge_sandbox_create_self_hosted_with_profile,
    edge_sandbox_destroy, edge_sandbox_enable_native_trace, edge_sandbox_evaluate,
    edge_sandbox_native_trace, edge_sandbox_native_trace_matching,
    edge_sandbox_profile_append_keyboard_layout_entry, edge_sandbox_profile_append_local_font,
    edge_sandbox_profile_append_midi_input, edge_sandbox_profile_append_midi_output,
    edge_sandbox_profile_append_speech_voice, edge_sandbox_profile_append_string,
    edge_sandbox_profile_append_ua_brand, edge_sandbox_profile_clear_keyboard_layout,
    edge_sandbox_profile_clear_local_fonts, edge_sandbox_profile_clear_midi_inputs,
    edge_sandbox_profile_clear_midi_outputs, edge_sandbox_profile_clear_speech_voices,
    edge_sandbox_profile_clear_string_list, edge_sandbox_profile_clear_ua_brands,
    edge_sandbox_profile_create, edge_sandbox_profile_destroy, edge_sandbox_profile_schema_version,
    edge_sandbox_profile_set_bool, edge_sandbox_profile_set_f32, edge_sandbox_profile_set_f64,
    edge_sandbox_profile_set_i32, edge_sandbox_profile_set_string, edge_sandbox_profile_set_u32,
    edge_sandbox_profile_set_u64, edge_sandbox_profile_validate, profile_field,
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

fn set_string(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, value: &str) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: all pointers remain live for the duration of the synchronous call.
    let succeeded = unsafe {
        edge_sandbox_profile_set_string(profile, field, value.as_ptr(), value.len(), &mut error)
    };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
}

fn set_i32(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, value: i32) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the profile is live and uniquely used by this test.
    let succeeded = unsafe { edge_sandbox_profile_set_i32(profile, field, value, &mut error) };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
}

fn set_u32(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, value: u32) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the profile is live and uniquely used by this test.
    let succeeded = unsafe { edge_sandbox_profile_set_u32(profile, field, value, &mut error) };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
}

fn set_u64(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, value: u64) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the profile is live and uniquely used by this test.
    let succeeded = unsafe { edge_sandbox_profile_set_u64(profile, field, value, &mut error) };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
}

fn set_f64(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, value: f64) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the profile is live and uniquely used by this test.
    let succeeded = unsafe { edge_sandbox_profile_set_f64(profile, field, value, &mut error) };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
}

fn set_f32(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, value: f32) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the profile is live and uniquely used by this test.
    let succeeded = unsafe { edge_sandbox_profile_set_f32(profile, field, value, &mut error) };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
}

fn set_bool(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, value: bool) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the profile is live and uniquely used by this test.
    let succeeded = unsafe { edge_sandbox_profile_set_bool(profile, field, value, &mut error) };
    assert!(succeeded, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
}

fn set_list(profile: *mut edge_sandbox::ffi::EdgeSandboxProfile, field: u32, values: &[&str]) {
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the profile is live and uniquely used by this test.
    let cleared = unsafe { edge_sandbox_profile_clear_string_list(profile, field, &mut error) };
    assert!(cleared, "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());
    for value in values {
        // SAFETY: all pointers remain live for the duration of the synchronous call.
        let appended = unsafe {
            edge_sandbox_profile_append_string(
                profile,
                field,
                value.as_ptr(),
                value.len(),
                &mut error,
            )
        };
        assert!(appended, "{}", take_buffer(&mut error));
        assert!(take_buffer(&mut error).is_empty());
    }
}

fn evaluate(runtime: *mut edge_sandbox::ffi::EdgeSandboxHandle, source: &str) -> String {
    let mut result = EdgeSandboxBuffer::default();
    let mut error = EdgeSandboxBuffer::default();
    // SAFETY: the runtime and source pointers remain live during this call.
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
fn complete_typed_profile_crosses_native_and_worker_boundaries() {
    assert_eq!(edge_sandbox_profile_schema_version(), 11);
    let mut error = EdgeSandboxBuffer::default();
    let profile = edge_sandbox_profile_create(&mut error);
    assert!(!profile.is_null(), "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());

    set_string(profile, profile_field::ID, "ffi-complete-profile");
    set_string(profile, profile_field::LOCALE, "fr-FR");
    set_string(profile, profile_field::TIME_ZONE, "Europe/Paris");
    set_i32(profile, profile_field::TIME_ZONE_OFFSET_MINUTES, -120);
    set_string(
        profile,
        profile_field::NAVIGATOR_USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
         (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36",
    );
    set_string(profile, profile_field::NAVIGATOR_LANGUAGE, "fr-FR");
    set_list(
        profile,
        profile_field::NAVIGATOR_LANGUAGES,
        &["fr-FR", "fr"],
    );
    set_u32(profile, profile_field::HARDWARE_CONCURRENCY, 16);
    set_f64(profile, profile_field::DEVICE_MEMORY_GB, 4.0);
    set_bool(profile, profile_field::NAVIGATOR_WEBDRIVER, false);
    set_string(profile, profile_field::NETWORK_EFFECTIVE_TYPE, "4g");
    set_u32(profile, profile_field::NETWORK_RTT, 42);
    set_f64(profile, profile_field::NETWORK_DOWNLINK, 8.5);
    set_bool(profile, profile_field::NETWORK_SAVE_DATA, false);
    set_string(profile, profile_field::UA_PLATFORM, "Windows");
    set_string(profile, profile_field::UA_FULL_VERSION, "151.0.0.0");
    set_list(profile, profile_field::UA_FORM_FACTORS, &["Desktop"]);
    set_i32(profile, profile_field::SCREEN_WIDTH, 1920);
    set_i32(profile, profile_field::SCREEN_HEIGHT, 1080);
    set_i32(profile, profile_field::SCREEN_AVAIL_WIDTH, 1900);
    set_i32(profile, profile_field::SCREEN_AVAIL_HEIGHT, 1040);
    set_f64(profile, profile_field::SCREEN_VIEWPORT_WIDTH, 1536.0);
    set_f64(profile, profile_field::SCREEN_VIEWPORT_HEIGHT, 864.0);
    set_f64(profile, profile_field::SCREEN_DEVICE_PIXEL_RATIO, 1.25);
    set_string(
        profile,
        profile_field::SCREEN_ORIENTATION_TYPE,
        "portrait-primary",
    );
    set_u32(profile, profile_field::SCREEN_ORIENTATION_ANGLE, 90);
    set_f64(profile, profile_field::VISUAL_VIEWPORT_OFFSET_LEFT, 3.0);
    set_f64(profile, profile_field::VISUAL_VIEWPORT_SCALE, 1.5);
    set_string(profile, profile_field::CANVAS_DATA_URL_SALT, "ffi-profile");
    set_f64(profile, profile_field::CANVAS_TEXT_WIDTH_SCALE, 1.2);
    set_f64(profile, profile_field::CANVAS_ACTUAL_BOUNDING_BOX_LEFT, 2.0);
    set_string(profile, profile_field::WEBGL_VENDOR, "Profile GL");
    set_string(
        profile,
        profile_field::WEBGL_UNMASKED_RENDERER,
        "Profile GPU Renderer",
    );
    set_list(
        profile,
        profile_field::WEBGL1_EXTENSIONS,
        &["WEBGL_debug_renderer_info"],
    );
    set_list(
        profile,
        profile_field::WEBGL2_EXTENSIONS,
        &["WEBGL_debug_renderer_info"],
    );
    set_i32(
        profile,
        profile_field::WEBGL_MAX_VERTEX_UNIFORM_VECTORS,
        2_048,
    );
    set_i32(
        profile,
        profile_field::WEBGL2_MAX_UNIFORM_BLOCK_SIZE,
        32_768,
    );
    set_u32(profile, profile_field::WEBGL2_MAX_ELEMENT_INDEX, 123_456);
    set_f64(profile, profile_field::WEBGL2_MAX_TEXTURE_LOD_BIAS, 1.5);
    set_string(profile, profile_field::WEBGPU_VENDOR, "Profile GPU");
    set_string(profile, profile_field::WEBGPU_DEVICE, "GPU-151");
    set_list(
        profile,
        profile_field::WEBGPU_FEATURES,
        &["bgra8unorm-storage"],
    );
    set_u32(
        profile,
        profile_field::WEBGPU_MAX_TEXTURE_DIMENSION_2D,
        4096,
    );
    set_f64(profile, profile_field::AUDIO_SAMPLE_RATE, 96_000.0);
    set_u32(profile, profile_field::AUDIO_MAX_CHANNEL_COUNT, 6);
    set_f64(profile, profile_field::AUDIO_BASE_LATENCY, 0.004);
    set_f64(profile, profile_field::AUDIO_OUTPUT_LATENCY, 0.017);
    set_f32(
        profile,
        profile_field::AUDIO_CHANNEL_NOISE_AMPLITUDE,
        0.000_01,
    );
    set_u64(profile, profile_field::STORAGE_QUOTA_BYTES, 2_000_000);
    set_u64(profile, profile_field::STORAGE_USAGE_BYTES, 125_000);
    set_bool(profile, profile_field::STORAGE_PERSISTED, true);
    set_u64(
        profile,
        profile_field::PERFORMANCE_JS_HEAP_SIZE_LIMIT,
        700_000_000,
    );
    set_u64(
        profile,
        profile_field::PERFORMANCE_TOTAL_JS_HEAP_SIZE,
        100_000_000,
    );
    set_u64(
        profile,
        profile_field::PERFORMANCE_USED_JS_HEAP_SIZE,
        60_000_000,
    );
    set_u64(
        profile,
        profile_field::CONSOLE_JS_HEAP_SIZE_LIMIT,
        650_000_000,
    );
    set_u64(
        profile,
        profile_field::CONSOLE_TOTAL_JS_HEAP_SIZE,
        90_000_000,
    );
    set_u64(
        profile,
        profile_field::CONSOLE_USED_JS_HEAP_SIZE,
        50_000_000,
    );
    set_string(profile, profile_field::DEVICE_POSTURE, "folded");
    set_u32(profile, profile_field::DOCUMENT_BODY_CHILD_ELEMENT_COUNT, 5);
    set_f64(profile, profile_field::DOCUMENT_BODY_CLIENT_HEIGHT, 23.0);
    set_bool(profile, profile_field::DOCUMENT_HAS_FOCUS, false);
    set_string(profile, profile_field::DOCUMENT_VISIBILITY_STATE, "hidden");
    set_bool(profile, profile_field::DOCUMENT_IS_POPUP, true);
    set_bool(
        profile,
        profile_field::NAVIGATOR_USER_ACTIVATION_HAS_BEEN_ACTIVE,
        true,
    );
    set_bool(
        profile,
        profile_field::NAVIGATOR_USER_ACTIVATION_IS_ACTIVE,
        true,
    );
    set_bool(
        profile,
        profile_field::MEDIA_PREFERENCE_REDUCED_TRANSPARENCY,
        true,
    );
    set_string(
        profile,
        profile_field::MEDIA_PREFERENCE_VIDEO_DYNAMIC_RANGE,
        "high",
    );
    set_list(
        profile,
        profile_field::IMAGE_DECODER_TYPES,
        &["image/profile"],
    );
    set_list(profile, profile_field::AUDIO_DECODER_CODECS, &["mp4a.40.2"]);
    set_list(profile, profile_field::AUDIO_ENCODER_CODECS, &["opus"]);
    set_list(profile, profile_field::VIDEO_DECODER_CODECS, &["avc1.*"]);
    set_list(profile, profile_field::VIDEO_ENCODER_CODECS, &["vp8"]);
    set_list(
        profile,
        profile_field::XR_SUPPORTED_SESSION_MODES,
        &["inline"],
    );
    set_string(
        profile,
        profile_field::RTC_OFFER_SDP,
        "v=0\r\ns=ffi-offer\r\n",
    );
    set_string(
        profile,
        profile_field::RTC_ANSWER_SDP,
        "v=0\r\ns=ffi-answer\r\n",
    );
    set_string(profile, profile_field::PERMISSION_NOTIFICATIONS, "granted");

    // SAFETY: the profile is live and uniquely used by this test.
    assert!(unsafe { edge_sandbox_profile_clear_ua_brands(profile, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let brand = "Chromium";
    let version = "151";
    let full_version = "151.0.0.0";
    // SAFETY: string pointers remain live during this synchronous call.
    assert!(unsafe {
        edge_sandbox_profile_append_ua_brand(
            profile,
            brand.as_ptr(),
            brand.len(),
            version.as_ptr(),
            version.len(),
            full_version.as_ptr(),
            full_version.len(),
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());

    // SAFETY: the profile is live and uniquely used by this test.
    assert!(unsafe { edge_sandbox_profile_clear_keyboard_layout(profile, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let keyboard_code = "KeyZ";
    let keyboard_value = "ffi-z";
    // SAFETY: string pointers remain live during this synchronous call.
    assert!(unsafe {
        edge_sandbox_profile_append_keyboard_layout_entry(
            profile,
            keyboard_code.as_ptr(),
            keyboard_code.len(),
            keyboard_value.as_ptr(),
            keyboard_value.len(),
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());

    // SAFETY: the profile is live and uniquely used by this test.
    assert!(unsafe { edge_sandbox_profile_clear_local_fonts(profile, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let postscript_name = "FfiProfileSans-Regular";
    let full_name = "FFI Profile Sans Regular";
    let family = "FFI Profile Sans";
    let style = "Regular";
    // SAFETY: string pointers remain live during this synchronous call.
    assert!(unsafe {
        edge_sandbox_profile_append_local_font(
            profile,
            postscript_name.as_ptr(),
            postscript_name.len(),
            full_name.as_ptr(),
            full_name.len(),
            family.as_ptr(),
            family.len(),
            style.as_ptr(),
            style.len(),
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());

    set_bool(profile, profile_field::MIDI_SYSEX_ENABLED, true);
    // SAFETY: the profile is live and uniquely used by this test.
    assert!(unsafe { edge_sandbox_profile_clear_midi_inputs(profile, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let midi_input_id = "ffi-midi-input";
    let midi_input_manufacturer = "FFI MIDI Manufacturer";
    let midi_input_name = "FFI MIDI Input";
    let midi_input_version = "4.0";
    let midi_input_state = "connected";
    let midi_input_connection = "open";
    // SAFETY: string pointers remain live during this synchronous call.
    assert!(unsafe {
        edge_sandbox_profile_append_midi_input(
            profile,
            midi_input_id.as_ptr(),
            midi_input_id.len(),
            midi_input_manufacturer.as_ptr(),
            midi_input_manufacturer.len(),
            midi_input_name.as_ptr(),
            midi_input_name.len(),
            midi_input_version.as_ptr(),
            midi_input_version.len(),
            midi_input_state.as_ptr(),
            midi_input_state.len(),
            midi_input_connection.as_ptr(),
            midi_input_connection.len(),
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());

    // SAFETY: the profile is live and uniquely used by this test.
    assert!(unsafe { edge_sandbox_profile_clear_midi_outputs(profile, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let midi_output_id = "ffi-midi-output";
    let midi_output_manufacturer = "FFI MIDI Manufacturer";
    let midi_output_name = "FFI MIDI Output";
    let midi_output_version = "5.0";
    let midi_output_state = "disconnected";
    let midi_output_connection = "closed";
    // SAFETY: string pointers remain live during this synchronous call.
    assert!(unsafe {
        edge_sandbox_profile_append_midi_output(
            profile,
            midi_output_id.as_ptr(),
            midi_output_id.len(),
            midi_output_manufacturer.as_ptr(),
            midi_output_manufacturer.len(),
            midi_output_name.as_ptr(),
            midi_output_name.len(),
            midi_output_version.as_ptr(),
            midi_output_version.len(),
            midi_output_state.as_ptr(),
            midi_output_state.len(),
            midi_output_connection.as_ptr(),
            midi_output_connection.len(),
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());

    // SAFETY: the profile is live and uniquely used by this test.
    assert!(unsafe { edge_sandbox_profile_clear_speech_voices(profile, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let voice = "Profile Voice";
    let lang = "fr-FR";
    // SAFETY: string pointers remain live during this synchronous call.
    assert!(unsafe {
        edge_sandbox_profile_append_speech_voice(
            profile,
            voice.as_ptr(),
            voice.len(),
            voice.as_ptr(),
            voice.len(),
            lang.as_ptr(),
            lang.len(),
            true,
            true,
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());

    // SAFETY: the profile is live for this synchronous validation call.
    assert!(unsafe { edge_sandbox_profile_validate(profile, &mut error) });
    assert!(take_buffer(&mut error).is_empty());

    // SAFETY: all pointers remain live during runtime creation.
    let runtime = unsafe { edge_sandbox_create_self_hosted_with_profile(profile, &mut error) };
    assert!(!runtime.is_null(), "{}", take_buffer(&mut error));
    assert!(take_buffer(&mut error).is_empty());

    // SAFETY: the runtime handle is live.
    assert!(unsafe { edge_sandbox_enable_native_trace(runtime, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let source = r#"
        (() => {
          const context = new AudioContext();
          const frame = document.createElement("iframe");
          document.body.appendChild(frame);
          const canvas = document.createElement("canvas");
          const twoD = canvas.getContext("2d");
          const gl = document.createElement("canvas").getContext("webgl");
          const gl2 = document.createElement("canvas").getContext("webgl2");
          const debug = gl.getExtension("WEBGL_debug_renderer_info");
          return [
            navigator.userAgent.includes("Chrome/151.0.0.0"),
            navigator.language,
            navigator.languages.join(","),
            navigator.hardwareConcurrency,
            navigator.deviceMemory,
            navigator.webdriver,
            navigator.userAgentData.platform,
            navigator.userAgentData.brands[0].version,
            navigator.connection.rtt,
            navigator.connection.downlink,
            screen.width,
            innerWidth,
            devicePixelRatio,
            screen.orientation.type,
            screen.orientation.angle,
            visualViewport.offsetLeft,
            visualViewport.scale,
            navigator.devicePosture.type,
            matchMedia("(device-posture: folded)").matches,
            matchMedia("(prefers-reduced-transparency: reduce)").matches,
            matchMedia("(video-dynamic-range: high)").matches,
            navigator.userActivation.hasBeenActive,
            navigator.userActivation.isActive,
            document.body.childElementCount,
            document.body.clientHeight,
            document.hasFocus(),
            document.hidden,
            document.visibilityState,
            [locationbar, menubar, personalbar, scrollbars, statusbar, toolbar]
              .every(bar => bar.visible === false),
            performance.memory.jsHeapSizeLimit,
            performance.memory.usedJSHeapSize,
            console.memory.jsHeapSizeLimit,
            console.memory.usedJSHeapSize,
            twoD.measureText("abcd").width.toFixed(1),
            twoD.measureText("abcd").actualBoundingBoxLeft,
            gl.getParameter(gl.VENDOR),
            gl.getParameter(debug.UNMASKED_RENDERER_WEBGL),
            gl.getParameter(gl.MAX_VERTEX_UNIFORM_VECTORS),
            gl2.getParameter(gl2.MAX_UNIFORM_BLOCK_SIZE),
            gl2.getParameter(gl2.MAX_ELEMENT_INDEX),
            gl2.getParameter(gl2.MAX_TEXTURE_LOD_BIAS),
            context.sampleRate,
            context.destination.maxChannelCount,
            context.baseLatency,
            context.outputLatency,
            speechSynthesis.getVoices()[0].name,
            Notification.permission,
            frame.contentWindow.navigator.language,
            frame.contentWindow.screen.width,
            Function.prototype.toString.call(context.createGain)
          ].join("|");
        })()
    "#;
    let mut result = EdgeSandboxBuffer::default();
    // SAFETY: all source and output pointers are valid for this synchronous call.
    assert!(unsafe {
        edge_sandbox_evaluate(
            runtime,
            source.as_ptr(),
            source.len(),
            &mut result,
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());
    assert_eq!(
        take_buffer(&mut result),
        concat!(
            "true|fr-FR|fr-FR,fr|16|4|false|Windows|151|42|8.5|",
            "1920|1536|1.25|portrait-primary|90|3|1.5|folded|",
            "true|true|true|true|true|5|23|false|true|hidden|true|",
            "700000000|60000000|650000000|50000000|",
            "27.4|2|Profile GL|Profile GPU Renderer|2048|32768|123456|1.5|",
            "96000|6|0.004|0.017|",
            "Profile Voice|granted|fr-FR|1920|function createGain() { [native code] }"
        )
    );

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileWorkerAnswer = "pending";
            const profileWorker = new Worker(URL.createObjectURL(new Blob([
              `postMessage([
                navigator.userAgent.includes("Chrome/151.0.0.0"),
                navigator.language,
                navigator.hardwareConcurrency
              ].join("|"))`
            ], {type: "text/javascript"})));
            profileWorker.onmessage = event => profileWorkerAnswer = event.data;
            "#,
    );
    assert_eq!(evaluate(runtime, "profileWorkerAnswer"), "true|fr-FR|16");

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileWorkletAnswer = "pending";
            const profileAudio = new AudioContext();
            profileAudio.audioWorklet.addModule(URL.createObjectURL(new Blob([
              `class ProfileProcessor extends AudioWorkletProcessor {
                process() {
                  this.port.postMessage(sampleRate);
                  return false;
                }
              }
              registerProcessor("profile-processor", ProfileProcessor);`
            ], {type: "text/javascript"}))).then(() => {
              const node = new AudioWorkletNode(
                profileAudio,
                "profile-processor"
              );
              node.port.onmessage = event => profileWorkletAnswer = event.data;
            });
            "#,
    );
    assert_eq!(evaluate(runtime, "profileWorkletAnswer"), "96000");

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileStorageAnswer = "pending";
            Promise.all([
              navigator.storage.estimate(),
              navigator.storage.persisted()
            ]).then(values => {
              profileStorageAnswer = [
                values[0].quota,
                values[0].usage,
                values[1]
              ].join("|");
            });
            "#,
    );
    assert_eq!(
        evaluate(runtime, "profileStorageAnswer"),
        "2000000|125000|true"
    );

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileGpuAnswer = "pending";
            navigator.gpu.requestAdapter().then(adapter => {
              profileGpuAnswer = [
                adapter.info.vendor,
                adapter.info.device,
                adapter.limits.maxTextureDimension2D,
                adapter.features.has("bgra8unorm-storage")
              ].join("|");
            });
            "#,
    );
    assert_eq!(
        evaluate(runtime, "profileGpuAnswer"),
        "Profile GPU|GPU-151|4096|true"
    );

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileExtendedAnswer = "pending";
            Promise.all([
              queryLocalFonts(),
              ImageDecoder.isTypeSupported("image/profile"),
              ImageDecoder.isTypeSupported("image/png"),
              navigator.xr.isSessionSupported("inline"),
              navigator.xr.isSessionSupported("immersive-vr"),
              new RTCPeerConnection().createOffer(),
              new RTCPeerConnection().createAnswer()
            ]).then(values => {
              profileExtendedAnswer = [
                values[0][0].postscriptName,
                values[0][0].fullName,
                values[1],
                values[2],
                values[3],
                values[4],
                values[5].sdp.includes("ffi-offer"),
                values[6].sdp.includes("ffi-answer"),
                Function.prototype.toString.call(ImageDecoder.isTypeSupported)
              ].join("|");
            });
            "#,
    );
    assert_eq!(
        evaluate(runtime, "profileExtendedAnswer"),
        concat!(
            "FfiProfileSans-Regular|FFI Profile Sans Regular|",
            "true|false|true|false|true|true|",
            "function isTypeSupported() { [native code] }"
        )
    );

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileWebCodecsAnswer = "pending";
            Promise.all([
              AudioDecoder.isConfigSupported({
                codec: "mp4a.40.2",
                numberOfChannels: 2,
                sampleRate: 48000
              }),
              AudioEncoder.isConfigSupported({
                codec: "mp4a.40.2",
                numberOfChannels: 2,
                sampleRate: 48000
              }),
              VideoDecoder.isConfigSupported({
                codec: "avc1.42001e",
                codedWidth: 2,
                codedHeight: 2
              }),
              VideoEncoder.isConfigSupported({
                codec: "avc1.42001e",
                width: 2,
                height: 2
              })
            ]).then(values => {
              profileWebCodecsAnswer = values
                .map(value => value.supported)
                .join("|");
            });
            "#,
    );
    assert_eq!(
        evaluate(runtime, "profileWebCodecsAnswer"),
        "true|false|true|false"
    );

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileKeyboardAnswer = "pending";
            navigator.keyboard.getLayoutMap().then(layout => {
              profileKeyboardAnswer = layout.get("KeyZ");
            });
            "#,
    );
    assert_eq!(evaluate(runtime, "profileKeyboardAnswer"), "ffi-z");

    let _ = evaluate(
        runtime,
        r#"
            globalThis.profileMidiAnswer = "pending";
            navigator.requestMIDIAccess().then(access => {
              const input = access.inputs.get("ffi-midi-input");
              const output = access.outputs.get("ffi-midi-output");
              profileMidiAnswer = [
                access.sysexEnabled,
                input.name,
                input.connection,
                output.name,
                output.state
              ].join("|");
            });
            "#,
    );
    assert_eq!(
        evaluate(runtime, "profileMidiAnswer"),
        "true|FFI MIDI Input|open|FFI MIDI Output|disconnected"
    );

    let mut trace = EdgeSandboxBuffer::default();
    // SAFETY: the runtime handle and output pointers are live.
    assert!(unsafe { edge_sandbox_native_trace(runtime, &mut trace, &mut error) });
    assert!(take_buffer(&mut error).is_empty());
    let trace = take_buffer(&mut trace);
    assert!(trace.contains("window.navigator"));
    assert!(trace.contains("sampleRate"));

    let needle = "sampleRate";
    let mut matching_trace = EdgeSandboxBuffer::default();
    // SAFETY: the runtime, filter bytes, and output pointers are live.
    assert!(unsafe {
        edge_sandbox_native_trace_matching(
            runtime,
            needle.as_ptr(),
            needle.len(),
            &mut matching_trace,
            &mut error,
        )
    });
    assert!(take_buffer(&mut error).is_empty());
    let matching_trace = take_buffer(&mut matching_trace);
    assert!(matching_trace.contains("sampleRate"));
    assert!(!matching_trace.contains("window.navigator"));

    // SAFETY: both handles are live and destroyed exactly once.
    unsafe {
        edge_sandbox_destroy(runtime);
        edge_sandbox_profile_destroy(profile);
    }
}

#[test]
fn invalid_typed_profile_is_rejected_before_worker_creation() {
    let mut error = EdgeSandboxBuffer::default();
    let profile = edge_sandbox_profile_create(&mut error);
    assert!(!profile.is_null(), "{}", take_buffer(&mut error));
    set_f64(profile, profile_field::AUDIO_SAMPLE_RATE, 1.0);
    // SAFETY: the profile is live for this synchronous validation call.
    assert!(!unsafe { edge_sandbox_profile_validate(profile, &mut error) });
    assert!(take_buffer(&mut error).contains("rendering fingerprint"));
    // SAFETY: the handle is live and destroyed exactly once.
    unsafe {
        edge_sandbox_profile_destroy(profile);
    }
}
