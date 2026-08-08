use crate::{
    BluetoothDeviceFingerprint, EdgeRuntime, EdgeRuntimeOptions, Evaluation, GamepadFingerprint,
    HidDeviceFingerprint, KeyboardLayoutEntryFingerprint, LocalFontFingerprint,
    MediaDeviceFingerprint, MidiPortFingerprint, MimeTypeFingerprint, PluginFingerprint,
    RtcCodecFingerprint, SerialPortFingerprint, UsbDeviceFingerprint,
};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

fn environment_options() -> EdgeRuntimeOptions {
    let mut options = EdgeRuntimeOptions::default();
    let fingerprint = &mut options.fingerprint;

    fingerprint.screen.orientation_type = "portrait-secondary".to_owned();
    fingerprint.screen.orientation_angle = 270;
    fingerprint.screen.visual_viewport_offset_left = 4.5;
    fingerprint.screen.visual_viewport_offset_top = 5.5;
    fingerprint.screen.visual_viewport_page_left = 6.5;
    fingerprint.screen.visual_viewport_page_top = 7.5;
    fingerprint.screen.visual_viewport_scale = 1.75;

    fingerprint.rendering.canvas.text_width_scale = 1.25;
    fingerprint.rendering.canvas.actual_bounding_box_left = 1.5;
    fingerprint.rendering.canvas.actual_bounding_box_right_scale = 0.75;
    fingerprint.rendering.canvas.font_bounding_box_ascent = 10.0;
    fingerprint.rendering.canvas.font_bounding_box_descent = 3.0;
    fingerprint.rendering.canvas.actual_bounding_box_ascent = 9.0;
    fingerprint.rendering.canvas.actual_bounding_box_descent = 2.5;
    fingerprint.rendering.canvas.hanging_baseline = 7.25;
    fingerprint.rendering.canvas.alphabetic_baseline = 0.5;
    fingerprint.rendering.canvas.ideographic_baseline = -2.5;

    fingerprint.fonts.families = vec!["Configured Sans".to_owned()];
    fingerprint.fonts.allow_unknown_families = false;
    fingerprint.fonts.local_fonts = vec![LocalFontFingerprint {
        postscript_name: "ConfiguredSans-Bold".to_owned(),
        full_name: "Configured Sans Bold".to_owned(),
        family: "Configured Sans".to_owned(),
        style: "Bold".to_owned(),
    }];

    fingerprint.media.devices = vec![MediaDeviceFingerprint {
        device_id: "camera-profile-1".to_owned(),
        kind: "videoinput".to_owned(),
        label: "Configured Camera".to_owned(),
        group_id: "configured-group".to_owned(),
    }];
    fingerprint.media.image_decoder_types = vec!["image/profile".to_owned()];
    fingerprint.media.rtc_audio_codecs = vec![RtcCodecFingerprint {
        mime_type: "audio/profile".to_owned(),
        clock_rate: 32_000,
        channels: Some(1),
        sdp_fmtp_line: Some("mode=configured".to_owned()),
    }];
    fingerprint.media.rtc_video_codecs.clear();
    fingerprint.media.rtc_offer_sdp = "v=0\r\ns=configured-offer\r\n".to_owned();
    fingerprint.media.rtc_answer_sdp = "v=0\r\ns=configured-answer\r\n".to_owned();

    fingerprint.permissions.camera = "granted".to_owned();
    fingerprint.permissions.geolocation = "granted".to_owned();
    fingerprint.permissions.local_fonts = "granted".to_owned();
    fingerprint.permissions.notifications = "prompt".to_owned();
    fingerprint.battery.charging = false;
    fingerprint.battery.charging_time = f64::INFINITY;
    fingerprint.battery.discharging_time = 7200.0;
    fingerprint.battery.level = 0.42;
    fingerprint.geolocation.latitude = 31.2304;
    fingerprint.geolocation.longitude = 121.4737;
    fingerprint.geolocation.accuracy = 12.5;
    fingerprint.media_preferences.color_scheme = "dark".to_owned();
    fingerprint.media_preferences.reduced_motion = true;
    fingerprint.media_preferences.reduced_transparency = true;
    fingerprint.media_preferences.video_dynamic_range = "high".to_owned();
    fingerprint.navigator.user_activation_has_been_active = true;
    fingerprint.navigator.user_activation_is_active = true;
    fingerprint.plugins.plugins = vec![PluginFingerprint {
        name: "Configured Plugin".to_owned(),
        filename: "configured-plugin.dll".to_owned(),
        description: "Configured plugin description".to_owned(),
        mime_types: vec![MimeTypeFingerprint {
            mime_type: "application/x-profile".to_owned(),
            suffixes: "profile".to_owned(),
            description: "Configured MIME".to_owned(),
        }],
    }];

    fingerprint.hardware_devices.gamepads = vec![GamepadFingerprint {
        id: "Configured Gamepad".to_owned(),
        index: 1,
        connected: true,
        mapping: "standard".to_owned(),
        axes: vec![0.25, -0.5],
        buttons: vec![1.0, 0.25],
    }];
    fingerprint.hardware_devices.usb_devices = vec![UsbDeviceFingerprint {
        vendor_id: 0x1234,
        product_id: 0x5678,
        product_name: Some("Configured USB".to_owned()),
        ..UsbDeviceFingerprint::default()
    }];
    fingerprint.hardware_devices.hid_devices = vec![HidDeviceFingerprint {
        vendor_id: 0x1111,
        product_id: 0x2222,
        product_name: "Configured HID".to_owned(),
    }];
    fingerprint.hardware_devices.serial_ports = vec![SerialPortFingerprint {
        usb_vendor_id: 0x3333,
        usb_product_id: 0x4444,
        connected: true,
    }];
    fingerprint.hardware_devices.bluetooth_available = false;
    fingerprint.hardware_devices.bluetooth_devices = vec![BluetoothDeviceFingerprint {
        id: "bluetooth-profile".to_owned(),
        name: Some("Configured Bluetooth".to_owned()),
    }];
    fingerprint.hardware_devices.keyboard_layout = vec![
        KeyboardLayoutEntryFingerprint {
            code: "KeyA".to_owned(),
            value: "ä".to_owned(),
        },
        KeyboardLayoutEntryFingerprint {
            code: "KeyQ".to_owned(),
            value: "q-profile".to_owned(),
        },
    ];
    fingerprint.hardware_devices.device_posture = "folded".to_owned();
    fingerprint.hardware_devices.midi_inputs = vec![MidiPortFingerprint {
        id: "midi-in-profile".to_owned(),
        manufacturer: "Input Manufacturer".to_owned(),
        name: "Configured MIDI Input".to_owned(),
        version: "2.1".to_owned(),
        state: "connected".to_owned(),
        connection: "open".to_owned(),
    }];
    fingerprint.hardware_devices.midi_outputs = vec![MidiPortFingerprint {
        id: "midi-out-profile".to_owned(),
        manufacturer: "Output Manufacturer".to_owned(),
        name: "Configured MIDI Output".to_owned(),
        version: "3.2".to_owned(),
        state: "disconnected".to_owned(),
        connection: "closed".to_owned(),
    }];
    fingerprint.hardware_devices.midi_sysex_enabled = true;

    fingerprint.sensors.accelerometer = [1.0, 2.0, 3.0];
    fingerprint.sensors.gyroscope = [4.0, 5.0, 6.0];
    fingerprint.sensors.absolute_orientation_quaternion = [0.1, 0.2, 0.3, 0.9];
    fingerprint.xr.supported_session_modes = vec!["inline".to_owned()];
    fingerprint.memory.performance_js_heap_size_limit = 900_000_000;
    fingerprint.memory.performance_total_js_heap_size = 120_000_000;
    fingerprint.memory.performance_used_js_heap_size = 80_000_000;
    fingerprint.memory.console_js_heap_size_limit = 800_000_000;
    fingerprint.memory.console_total_js_heap_size = 110_000_000;
    fingerprint.memory.console_used_js_heap_size = 70_000_000;
    options
}

#[test]
fn media_capability_matching_keeps_container_and_codec_support_separate() {
    let patterns = vec![
        "video/webm".to_owned(),
        "video/webm;codecs=vp8*".to_owned(),
        "video/mp4;codecs=avc1.*".to_owned(),
    ];
    assert!(crate::fingerprint_environment::media_capability_matches(
        &patterns,
        "video/webm"
    ));
    assert!(crate::fingerprint_environment::media_capability_matches(
        &patterns,
        "video/webm; codecs=\"vp8\""
    ));
    assert!(crate::fingerprint_environment::media_capability_matches(
        &patterns,
        "video/mp4; codecs=\"avc1.64003E, opus\""
    ));
    assert!(!crate::fingerprint_environment::media_capability_matches(
        &patterns,
        "video/webm;codecs=daala"
    ));
    assert!(!crate::fingerprint_environment::media_capability_matches(
        &patterns,
        "video/webm;codecs=h264"
    ));
}

#[test]
fn prompt_permissions_do_not_disclose_profiled_devices_location_or_fonts() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.permissions.camera = "prompt".to_owned();
    options.fingerprint.permissions.microphone = "prompt".to_owned();
    options.fingerprint.permissions.geolocation = "prompt".to_owned();
    options.fingerprint.permissions.local_fonts = "prompt".to_owned();
    options.fingerprint.media.devices = vec![MediaDeviceFingerprint {
        device_id: "secret-microphone-id".to_owned(),
        kind: "audioinput".to_owned(),
        label: "Secret Microphone".to_owned(),
        group_id: "secret-group".to_owned(),
    }];
    let mut runtime = EdgeRuntime::with_options(options).expect("permission-gated runtime");
    text(
        &mut runtime,
        r#"
        globalThis.permissionAnswer = "pending";
        const geolocation = new Promise(resolve => {
          navigator.geolocation.getCurrentPosition(
            () => resolve("success"),
            error => resolve(`error:${error.code}`)
          );
        });
        Promise.all([
          geolocation,
          navigator.mediaDevices.getUserMedia({audio: true}).then(
            () => "success",
            error => `error:${error.name}`
          ),
          queryLocalFonts().then(
            () => "success",
            error => `error:${error.name}`
          ),
          navigator.mediaDevices.enumerateDevices().then(devices =>
            devices.map(device => [
              device.kind, device.deviceId, device.label, device.groupId
            ].join(":")).join(",")
          )
        ]).then(values => permissionAnswer = values.join("|"));
        "#,
    );
    assert_eq!(
        text(&mut runtime, "permissionAnswer"),
        "error:1|error:NotAllowedError|error:NotAllowedError|audioinput:::"
    );
}

#[test]
fn environment_fingerprint_drives_each_exposed_api() {
    let mut runtime =
        EdgeRuntime::with_options(environment_options()).expect("configured environment runtime");
    let synchronous = text(
        &mut runtime,
        r#"
        (() => {
          const metrics = document
            .createElement("canvas")
            .getContext("2d")
            .measureText("abcd");
          const gamepad = navigator.getGamepads()[1];
          const accelerometer = new Accelerometer();
          const gyroscope = new Gyroscope();
          const orientation = new AbsoluteOrientationSensor();
          return [
            metrics.width,
            metrics.actualBoundingBoxLeft,
            metrics.actualBoundingBoxRight,
            metrics.fontBoundingBoxAscent,
            metrics.fontBoundingBoxDescent,
            metrics.actualBoundingBoxAscent,
            metrics.actualBoundingBoxDescent,
            metrics.hangingBaseline,
            metrics.alphabeticBaseline,
            metrics.ideographicBaseline,
            document.fonts.check("12px Configured Sans"),
            document.fonts.check("12px Missing Sans"),
            matchMedia("(prefers-color-scheme: dark)").matches,
            matchMedia("(prefers-reduced-motion: reduce)").matches,
            matchMedia("(prefers-reduced-transparency: reduce)").matches,
            matchMedia("(video-dynamic-range: high)").matches,
            matchMedia("(device-posture: folded)").matches,
            matchMedia("(device-posture: continuous)").matches,
            navigator.userActivation.hasBeenActive,
            navigator.userActivation.isActive,
            navigator.plugins.length,
            navigator.plugins[0].name,
            navigator.mimeTypes[0].type,
            navigator.mimeTypes[0].enabledPlugin === navigator.plugins[0],
            gamepad.id,
            gamepad.axes.join(","),
            gamepad.buttons.map(button => button.value).join(","),
            accelerometer.x,
            accelerometer.y,
            accelerometer.z,
            gyroscope.x,
            gyroscope.y,
            gyroscope.z,
            orientation.quaternion.join(","),
            Notification.permission,
            screen.orientation.type,
            screen.orientation.angle,
            visualViewport.offsetLeft,
            visualViewport.offsetTop,
            visualViewport.pageLeft,
            visualViewport.pageTop,
            visualViewport.scale,
            navigator.devicePosture.type,
            performance.memory.jsHeapSizeLimit,
            performance.memory.totalJSHeapSize,
            performance.memory.usedJSHeapSize,
            console.memory.jsHeapSizeLimit,
            console.memory.totalJSHeapSize,
            console.memory.usedJSHeapSize
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        synchronous,
        concat!(
            "28.555221557617188|1.5|21.216419219970703|10|3|9|2.5|7.25|0.5|-2.5|",
            "true|true|true|true|true|true|true|false|true|true|",
            "1|Configured Plugin|application/x-profile|true|",
            "Configured Gamepad|0.25,-0.5|1,0.25|",
            "1|2|3|4|5|6|0.1,0.2,0.3,0.9|default|",
            "portrait-secondary|270|4.5|5.5|6.5|7.5|1.75|folded|",
            "900000000|120000000|80000000|800000000|110000000|70000000"
        )
    );

    assert_eq!(
        text(
            &mut runtime,
            r#"
            globalThis.environmentAsyncAnswer = "pending";
            Promise.all([
              navigator.permissions.query({name: "camera"}),
              navigator.mediaDevices.enumerateDevices(),
              navigator.getBattery(),
              queryLocalFonts(),
              ImageDecoder.isTypeSupported("image/profile"),
              ImageDecoder.isTypeSupported("image/png"),
              navigator.xr.isSessionSupported("inline"),
              navigator.xr.isSessionSupported("immersive-vr"),
              new RTCPeerConnection().createOffer(),
              new RTCPeerConnection().createAnswer(),
              navigator.usb.getDevices(),
              navigator.hid.getDevices(),
              navigator.serial.getPorts(),
              navigator.bluetooth.getAvailability(),
              new Promise(resolve =>
                navigator.geolocation.getCurrentPosition(resolve)
              )
            ]).then(values => {
              environmentAsyncAnswer = [
                values[0].state,
                values[1][0].deviceId,
                values[1][0].label,
                values[2].charging,
                values[2].dischargingTime,
                values[2].level,
                values[3][0].postscriptName,
                values[3][0].fullName,
                values[3][0].family,
                values[3][0].style,
                values[4],
                values[5],
                values[6],
                values[7],
                values[8].sdp.includes("configured-offer"),
                values[9].sdp.includes("configured-answer"),
                values[10][0].vendorId,
                values[10][0].productId,
                values[11][0].vendorId,
                values[11][0].productId,
                values[12][0].getInfo().usbVendorId,
                values[12][0].getInfo().usbProductId,
                values[13],
                values[14].coords.latitude,
                values[14].coords.longitude,
                values[14].coords.accuracy
              ].join("|");
            });
            "scheduled"
            "#,
        ),
        "scheduled"
    );
    let mut async_answer = text(&mut runtime, "environmentAsyncAnswer");
    if async_answer == "pending" {
        async_answer = text(&mut runtime, "environmentAsyncAnswer");
    }
    assert_eq!(
        async_answer,
        concat!(
            "granted|camera-profile-1|Configured Camera|false|7200|0.42|",
            "ConfiguredSans-Bold|Configured Sans Bold|Configured Sans|Bold|",
            "true|false|true|false|true|true|",
            "4660|22136|4369|8738|13107|17476|false|",
            "31.2304|121.4737|12.5"
        )
    );

    assert_eq!(
        text(
            &mut runtime,
            r#"
            globalThis.keyboardProfileAnswer = "pending";
            navigator.keyboard.getLayoutMap().then(layout => {
              keyboardProfileAnswer = [
                layout.get("KeyA"),
                layout.get("KeyQ"),
                Array.from(layout.keys()).join(",")
              ].join("|");
            });
            "scheduled"
            "#,
        ),
        "scheduled"
    );
    assert_eq!(
        text(&mut runtime, "keyboardProfileAnswer"),
        "ä|q-profile|KeyA,KeyQ"
    );

    assert_eq!(
        text(
            &mut runtime,
            r#"
            globalThis.midiProfileAnswer = "pending";
            navigator.requestMIDIAccess().then(access => {
              const input = access.inputs.get("midi-in-profile");
              const output = access.outputs.get("midi-out-profile");
              midiProfileAnswer = [
                access.sysexEnabled,
                input.id,
                input.manufacturer,
                input.name,
                input.version,
                input.state,
                input.connection,
                output.id,
                output.manufacturer,
                output.name,
                output.version,
                output.state,
                output.connection,
                Array.from(access.inputs.keys()).join(","),
                Array.from(access.outputs.keys()).join(",")
              ].join("|");
            });
            "scheduled"
            "#,
        ),
        "scheduled"
    );
    assert_eq!(
        text(&mut runtime, "midiProfileAnswer"),
        concat!(
            "true|midi-in-profile|Input Manufacturer|Configured MIDI Input|",
            "2.1|connected|open|midi-out-profile|Output Manufacturer|",
            "Configured MIDI Output|3.2|disconnected|closed|",
            "midi-in-profile|midi-out-profile"
        )
    );
}

#[test]
fn unavailable_desktop_sensors_and_permissions_keep_the_captured_error_semantics() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.sensors.available = false;
    options.fingerprint.permissions.speaker_selection = "unsupported".to_owned();
    options.fingerprint.permissions.top_level_storage_access = "invalid-origin".to_owned();
    let mut runtime = EdgeRuntime::with_options(options).expect("unavailable sensor profile");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            Promise.all([
              navigator.permissions.query({name: "speaker-selection"}).then(
                value => `ok:${value.state}`,
                error => `${error.name}:${error.message}`
              ),
              navigator.permissions.query({name: "top-level-storage-access"}).then(
                value => `ok:${value.state}`,
                error => `${error.name}:${error.message}`
              )
            ]).then(values => {
              let sensorError = "missing";
              const sensor = new Accelerometer();
              sensor.onerror = event => {
                sensorError = `${event.error.name}:${event.error.message}`;
              };
              sensor.start();
              return [
                ...values,
                sensorError,
                sensor.activated,
                sensor.hasReading,
                sensor.timestamp
              ].join("|");
            })
            "#,
        ),
        concat!(
            "TypeError:Failed to execute 'query' on 'Permissions': ",
            "The Speaker Selection API is not enabled.|",
            "TypeError:Failed to execute 'query' on 'Permissions': ",
            "The requested origin is invalid.|",
            "NotReadableError:Could not connect to a sensor|false|false|"
        )
    );
}

#[test]
fn configured_environment_crosses_iframe_worker_and_native_trace_without_shape_drift() {
    let mut runtime =
        EdgeRuntime::with_options(environment_options()).expect("configured environment runtime");
    runtime.enable_native_trace().expect("enable native trace");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            const frame = document.createElement("iframe");
            document.body.appendChild(frame);
            globalThis.environmentFrameAnswer = [
              frame.contentWindow.matchMedia("(prefers-color-scheme: dark)").matches,
              frame.contentWindow.navigator.plugins[0].name,
              frame.contentWindow.document
                .createElement("canvas")
                .getContext("2d")
                .measureText("ab").actualBoundingBoxLeft
            ].join("|");

            globalThis.environmentWorkerAnswer = "pending";
            const worker = new Worker(URL.createObjectURL(new Blob([`
              Promise.all([
                ImageDecoder.isTypeSupported("image/profile"),
                navigator.permissions.query({name: "camera"})
              ]).then(values => {
                const metrics = new OffscreenCanvas(8, 8)
                  .getContext("2d")
                  .measureText("ab");
                postMessage([
                  values[0],
                  values[1].state,
                  metrics.actualBoundingBoxLeft,
                  Function.prototype.toString.call(
                    OffscreenCanvasRenderingContext2D.prototype.measureText
                  )
                ].join("|"));
              });
            `], {type: "text/javascript"})));
            worker.onmessage = event => environmentWorkerAnswer = event.data;

            [
              Function.prototype.toString.call(queryLocalFonts),
              Function.prototype.toString.call(
                CanvasRenderingContext2D.prototype.measureText
              ),
              Function.prototype.toString.call(ImageDecoder.isTypeSupported),
              Function.prototype.toString.call(XRSystem.prototype.isSessionSupported)
            ].join("|");
            "#,
        ),
        concat!(
            "function queryLocalFonts() { [native code] }|",
            "function measureText() { [native code] }|",
            "function isTypeSupported() { [native code] }|",
            "function isSessionSupported() { [native code] }"
        )
    );
    assert_eq!(
        text(&mut runtime, "environmentFrameAnswer"),
        "true|Configured Plugin|1.5"
    );
    assert_eq!(
        text(&mut runtime, "environmentWorkerAnswer"),
        concat!(
            "true|granted|1.5|",
            "function measureText() { [native code] }"
        )
    );

    let trace = runtime.native_trace();
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".measureText") })
    );
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".isTypeSupported") })
    );
}

#[test]
fn invalid_environment_fingerprint_is_rejected_before_v8_creation() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.xr.supported_session_modes = vec!["unsupported".to_owned()];
    assert!(EdgeRuntime::with_options(options).is_err());

    let mut options = EdgeRuntimeOptions::default();
    options
        .fingerprint
        .rendering
        .canvas
        .font_bounding_box_ascent = f64::NAN;
    assert!(EdgeRuntime::with_options(options).is_err());
}
