use crate::{
    EdgeFingerprint, EdgeRuntime, EdgeRuntimeOptions, Evaluation, NetworkReplayEntry, PageInit,
    PerformanceEntryFingerprint, SpeechVoiceFingerprint,
};

#[test]
fn typed_performance_profile_preserves_order_subtypes_and_compressed_body_sizes() {
    let mut navigation = PerformanceEntryFingerprint {
        name: "https://profile.example/page".to_owned(),
        entry_type: "navigation".to_owned(),
        duration: 3368.9,
        initiator_type: "navigation".to_owned(),
        next_hop_protocol: "h2".to_owned(),
        content_type: "text/html".to_owned(),
        content_encoding: "zstd".to_owned(),
        encoded_body_size: Some(587),
        decoded_body_size: Some(847),
        response_status: Some(429),
        dom_complete: 3368.9,
        load_event_end: 3368.9,
        ..PerformanceEntryFingerprint::default()
    };
    navigation.response_end = 425.9;
    let visible = PerformanceEntryFingerprint {
        name: "visible".to_owned(),
        entry_type: "visibility-state".to_owned(),
        ..PerformanceEntryFingerprint::default()
    };
    let resource = PerformanceEntryFingerprint {
        name: "https://profile.example/ips.js".to_owned(),
        entry_type: "resource".to_owned(),
        start_time: 431.3,
        duration: 2121.2,
        initiator_type: "script".to_owned(),
        next_hop_protocol: "h2".to_owned(),
        content_type: "text/javascript".to_owned(),
        content_encoding: "zstd".to_owned(),
        encoded_body_size: Some(291181),
        decoded_body_size: Some(609863),
        response_status: Some(200),
        response_end: 2552.5,
        ..PerformanceEntryFingerprint::default()
    };
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.performance.entries = Some(vec![navigation, visible, resource]);
    let mut runtime = EdgeRuntime::with_options(options).expect("profiled performance runtime");
    let value = runtime
        .evaluate_with_source_url(
            r#"
            performance.getEntries().map(entry => [
              entry.constructor.name,
              entry.entryType,
              entry.transferSize ?? "-",
              entry.encodedBodySize ?? "-",
              entry.decodedBodySize ?? "-",
              entry.contentEncoding ?? "-"
            ].join(",")).join("|")
            "#,
            "https://profile.example/ips.js",
        )
        .expect("profiled performance evaluation")
        .to_string();
    assert_eq!(
        value,
        "PerformanceNavigationTiming,navigation,887,587,847,zstd|\
         VisibilityStateEntry,visibility-state,-,-,-,-|\
         PerformanceResourceTiming,resource,291481,291181,609863,zstd"
    );
}

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Number(value) | Evaluation::Other(value) => value,
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
    }
}

#[test]
fn iframe_owns_an_ecmascript_realm_and_dom_relationships() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        const frame = document.createElement("iframe");
        frame.srcdoc = "<main id='inside'>child</main><script>window.childValue = 41 + 1<\/script>";
        document.body.appendChild(frame);
        [
          frame.contentWindow !== window,
          frame.contentWindow.Array !== Array,
          Object.getPrototypeOf(frame.contentWindow) === Window.prototype,
          frame.contentWindow.parent === window,
          frame.contentWindow.top === window,
          frame.contentWindow.frameElement === frame,
          frame.contentDocument.defaultView === frame.contentWindow,
          frame.contentDocument.getElementById("inside").textContent,
          frame.contentWindow.childValue,
          frame.contentDocument.URL
        ].join("|")
        "#,
    );
    assert_eq!(
        answer,
        "true|true|false|true|true|true|true|child|42|about:srcdoc"
    );
}

#[test]
fn audio_worklet_executes_module_and_connects_processor() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.audioWorkletAnswer = "pending";
            const context = new AudioContext({sampleRate: 44100});
            context.audioWorklet.addModule(
              "data:text/javascript," +
              encodeURIComponent(`
                class MeterProcessor extends AudioWorkletProcessor {
                  static get parameterDescriptors() {
                    return [{
                      name: "gain",
                      defaultValue: 0.25,
                      minValue: 0,
                      maxValue: 1,
                      automationRate: "k-rate"
                    }];
                  }
                  process() {
                    this.port.postMessage(
                      [sampleRate, currentFrame, currentTime, renderQuantumSize].join("|")
                    );
                    return false;
                  }
                }
                registerProcessor("meter", MeterProcessor);
              `)
            ).then(() => {
              const node = new AudioWorkletNode(context, "meter", {
                parameterData: {gain: 0.75},
                processorOptions: {mode: "test"}
              });
              node.port.onmessage = event => {
                const gain = node.parameters.get("gain");
                audioWorkletAnswer = [
                  event.data,
                  gain.value,
                  gain.defaultValue,
                  gain.minValue,
                  gain.maxValue,
                  gain.automationRate,
                  node.port instanceof MessagePort
                ].join("|");
              };
            });
            "#,
        )
        .expect("AudioWorklet setup");
    assert_eq!(
        text(&mut runtime, "audioWorkletAnswer"),
        "44100|0|0|128|0.75|0.25|0|1|k-rate|true"
    );
}

#[test]
fn audio_worklet_receives_edge_render_quantum_topology_and_parameter_arrays() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.audioWorkletQuantumAnswer = "pending";
            const context = new AudioContext({sampleRate: 48000});
            context.audioWorklet.addModule(
              "data:text/javascript," +
              encodeURIComponent(`
                class QuantumProcessor extends AudioWorkletProcessor {
                  static get parameterDescriptors() {
                    return [{
                      name: "level",
                      defaultValue: 0.5,
                      minValue: 0,
                      maxValue: 1,
                      automationRate: "k-rate"
                    }];
                  }
                  process(inputs, outputs, parameters) {
                    this.port.postMessage([
                      inputs.length,
                      inputs[0].length,
                      inputs[0][0].length,
                      outputs.length,
                      outputs[0].length,
                      outputs[0][0].length,
                      parameters.level.length,
                      parameters.level[0]
                    ].join("|"));
                    return false;
                  }
                }
                registerProcessor("quantum", QuantumProcessor);
              `)
            ).then(() => {
              const node = new AudioWorkletNode(context, "quantum", {
                numberOfInputs: 2,
                numberOfOutputs: 1,
                outputChannelCount: [2],
                parameterData: {level: 0.75}
              });
              node.port.onmessage = event => audioWorkletQuantumAnswer = event.data;
            });
            "#,
        )
        .expect("AudioWorklet quantum setup");
    assert_eq!(
        text(&mut runtime, "audioWorkletQuantumAnswer"),
        "2|1|128|1|2|128|1|0.75"
    );
}

#[test]
fn base_audio_context_factories_create_their_concrete_node_types_and_state() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const context = new AudioContext();
              const merger = context.createChannelMerger(4);
              const splitter = context.createChannelSplitter(3);
              const constant = context.createConstantSource();
              const convolver = context.createConvolver();
              const delay = context.createDelay(2.5);
              const compressor = context.createDynamicsCompressor();
              const gain = context.createGain();
              const stereo = context.createStereoPanner();
              const wave = context.createWaveShaper();
              const checks = [
                merger instanceof ChannelMergerNode,
                merger instanceof AudioNode,
                merger.numberOfInputs === 4,
                Object.getPrototypeOf(merger) === ChannelMergerNode.prototype,
                splitter instanceof ChannelSplitterNode,
                splitter instanceof AudioNode,
                splitter.numberOfOutputs === 3,
                Object.getPrototypeOf(splitter) === ChannelSplitterNode.prototype,
                constant instanceof ConstantSourceNode,
                constant instanceof AudioScheduledSourceNode,
                constant.offset instanceof AudioParam,
                convolver instanceof ConvolverNode,
                convolver.buffer === null,
                convolver.normalize === true,
                delay instanceof DelayNode,
                delay.delayTime instanceof AudioParam,
                compressor instanceof DynamicsCompressorNode,
                compressor.reduction === 0,
                compressor.threshold instanceof AudioParam,
                gain instanceof GainNode,
                gain.gain instanceof AudioParam,
                gain.gain.value === 1,
                stereo instanceof StereoPannerNode,
                stereo.pan instanceof AudioParam,
                stereo.pan.value === 0,
                wave instanceof WaveShaperNode,
                wave.curve === null,
                wave.oversample === "none",
                Object.getPrototypeOf(GainNode.prototype) === AudioNode.prototype
              ];
              try {
                context.createChannelMerger(0);
                checks.push(false);
              } catch (_) {
                checks.push(true);
              }
              try {
                context.createDelay(0);
                checks.push(false);
              } catch (_) {
                checks.push(true);
              }
              return checks.every(Boolean);
            })()
            "#,
        ),
        "true"
    );
}

#[test]
fn web_audio_clock_graph_automation_and_source_lifecycle_match_edge_semantics() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.webAudioLifecycleAnswer = "pending";
            (() => {
              const context = new AudioContext({sampleRate: 44100});
              const other = new AudioContext({sampleRate: 44100});
              const oscillator = context.createOscillator();
              const gain = context.createGain();
              const checks = [];

              gain.gain.setValueAtTime(0.25, context.currentTime + 10);
              gain.gain.linearRampToValueAtTime(0.75, context.currentTime + 20);
              checks.push(gain.gain.value === 1);
              checks.push(oscillator.connect(gain) === gain);
              checks.push(oscillator.connect(gain.gain) === undefined);
              oscillator.disconnect(gain.gain);
              try {
                oscillator.disconnect(gain.gain);
                checks.push(false);
              } catch (error) {
                checks.push(error.name === "InvalidAccessError");
              }
              try {
                oscillator.connect(other.destination);
                checks.push(false);
              } catch (error) {
                checks.push(error.name === "InvalidAccessError");
              }
              try {
                oscillator.connect(gain, 1, 0);
                checks.push(false);
              } catch (error) {
                checks.push(error.name === "IndexSizeError");
              }

              const source = context.createBufferSource();
              source.buffer = context.createBuffer(1, 128, 44100);
              let endedByHandler = false;
              let endedByListener = false;
              const ended = new Promise(resolve => {
                source.onended = () => {
                  endedByHandler = true;
                };
                source.addEventListener("ended", () => {
                  endedByListener = true;
                  resolve();
                });
              });
              source.start();

              const offline = new OfflineAudioContext(1, 256, 44100);
              return offline.startRendering().then(buffer => {
                checks.push(offline.state === "closed");
                checks.push(offline.currentTime === 256 / 44100);
                checks.push(buffer.length === 256);
                return other.close().then(() =>
                  other.resume().then(
                    () => checks.push(false),
                    error => checks.push(error.name === "InvalidStateError")
                  )
                );
              }).then(() => ended).then(() => {
                checks.push(endedByHandler);
                checks.push(endedByListener);
                webAudioLifecycleAnswer = checks.every(Boolean)
                  ? "true"
                  : checks.map((value, index) => `${index}:${value}`).join(",");
                return context.close();
              });
            })();
            "#,
        )
        .expect("Web Audio lifecycle setup");
    assert_eq!(text(&mut runtime, "webAudioLifecycleAnswer"), "true");
}

#[test]
fn offline_audio_context_renders_connected_sources_and_audio_param_automation() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.offlineAudioRenderAnswer = "pending";
            const offline = new OfflineAudioContext(1, 4410, 44100);
            const source = offline.createConstantSource();
            const gain = offline.createGain();
            source.offset.value = 1;
            gain.gain.setValueAtTime(0, 0);
            gain.gain.linearRampToValueAtTime(1, 0.1);
            source.connect(gain).connect(offline.destination);
            source.start(0);
            offline.startRendering().then(buffer => {
              const samples = buffer.getChannelData(0);
              let first = 0;
              let last = 0;
              for (let index = 0; index < 256; index++) {
                first += Math.abs(samples[index]);
                last += Math.abs(samples[samples.length - 1 - index]);
              }
              offlineAudioRenderAnswer = [
                first < last,
                samples[0] < samples[2205],
                samples[2205] < samples[4409],
                offline.currentTime === buffer.duration,
                offline.state === "closed"
              ].every(Boolean);
            });
            "#,
        )
        .expect("Offline Web Audio rendering setup");
    assert_eq!(text(&mut runtime, "offlineAudioRenderAnswer"), "true");
}

#[test]
fn offline_audio_triangle_compressor_matches_edge_rendering_kernel() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.offlineAudioFingerprintAnswer = "pending";
            const context = new OfflineAudioContext(1, 44100, 44100);
            const oscillator = context.createOscillator();
            oscillator.type = "triangle";
            oscillator.frequency.setValueAtTime(10000, 0);
            const compressor = context.createDynamicsCompressor();
            compressor.threshold.setValueAtTime(-50, 0);
            compressor.knee.setValueAtTime(40, 0);
            compressor.ratio.setValueAtTime(12, 0);
            compressor.attack.setValueAtTime(0, 0);
            compressor.release.setValueAtTime(0.25, 0);
            oscillator.connect(compressor).connect(context.destination);
            oscillator.start(0);
            context.startRendering().then(buffer => {
              const samples = buffer.getChannelData(0);
              let sum = 0;
              for (let index = 4500; index < 5000; ++index)
                sum += Math.abs(samples[index]);
              offlineAudioFingerprintAnswer = sum;
            });
            "#,
        )
        .expect("Offline Web Audio fingerprint setup");
    let actual = text(&mut runtime, "offlineAudioFingerprintAnswer")
        .parse::<f64>()
        .expect("numeric audio fingerprint");
    assert!(
        (actual - 124.043_446_115_174_45).abs() < 0.000_01,
        "unexpected Web Audio fingerprint: {actual}"
    );
}

#[test]
fn offline_audio_fingerprint_noise_perturbs_instead_of_replacing_rendered_samples() {
    const SOURCE: &str = r#"
        (() => {
          const context = new OfflineAudioContext(1, 256, 44100);
          const oscillator = context.createOscillator();
          oscillator.connect(context.destination);
          oscillator.start();
          return context.startRendering().then(buffer => {
            const data = buffer.getChannelData(0);
            let total = 0;
            for (let index = 0; index < data.length; ++index) {
              total += Math.abs(data[index]);
            }
            return total;
          });
        })()
    "#;

    let mut baseline = EdgeRuntime::new().expect("baseline offline audio");
    let baseline = text(&mut baseline, SOURCE)
        .parse::<f64>()
        .expect("baseline aggregate");

    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.rendering.audio.channel_noise_amplitude = 0.000_01;
    let mut noisy = EdgeRuntime::with_options(options).expect("noisy offline audio");
    let noisy = text(&mut noisy, SOURCE)
        .parse::<f64>()
        .expect("noisy aggregate");

    assert!(baseline > 1.0, "baseline aggregate was {baseline}");
    assert!(noisy > 1.0, "noise replaced rendered samples: {noisy}");
    assert!((noisy - baseline).abs() < 0.1);
    assert_ne!(noisy, baseline);
}

#[test]
fn oscillator_triangle_uses_the_edge_band_limited_waveform() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.bandLimitedTriangleAnswer = "pending";
            const context = new OfflineAudioContext(1, 64, 44100);
            const oscillator = context.createOscillator();
            oscillator.type = "triangle";
            oscillator.frequency.setValueAtTime(10000, 0);
            oscillator.connect(context.destination);
            oscillator.start(0);
            context.startRendering().then(buffer => {
              bandLimitedTriangleAnswer = Array.from(
                buffer.getChannelData(0).slice(0, 4)
              ).join(",");
            });
            "#,
        )
        .expect("band-limited triangle setup");
    let samples = text(&mut runtime, "bandLimitedTriangleAnswer")
        .split(',')
        .map(|value| value.parse::<f64>().expect("numeric oscillator sample"))
        .collect::<Vec<_>>();
    let edge = [
        0.0,
        0.802_099_764_347_076_4,
        0.233_441_948_890_686_04,
        -0.734_159_171_581_268_3,
    ];
    for (actual, expected) in samples.iter().zip(edge) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "{actual} != {expected}"
        );
    }
}

#[test]
fn input_event_uses_webidl_dictionary_and_sequence_conversion_order() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const log = [];
              const sequence = new Proxy({
                [Symbol.iterator]() { return [][Symbol.iterator](); }
              }, {
                get(target, key, receiver) {
                  log.push(String(key));
                  return Reflect.get(target, key, receiver);
                }
              });
              const init = new Proxy({targetRanges: sequence}, {
                get(target, key, receiver) {
                  log.push(String(key));
                  return Reflect.get(target, key, receiver);
                }
              });
              new InputEvent("input", init);
              return log.join(",");
            })()
            "#,
        ),
        "bubbles,cancelable,composed,detail,sourceCapabilities,view,data,dataTransfer,inputType,isComposing,targetRanges,Symbol(Symbol.iterator)"
    );
}

#[test]
fn blob_uses_webidl_sequence_conversion_before_blob_property_bag() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const log = [];
              const parts = new Proxy({}, {
                get(target, key, receiver) {
                  log.push(`g:${String(key)}`);
                  return Reflect.get(target, key, receiver);
                }
              });
              const options = new Proxy({}, {
                get(target, key, receiver) {
                  log.push(`o:${String(key)}`);
                  return Reflect.get(target, key, receiver);
                }
              });
              try { new Blob(parts, options); } catch (_) {}
              return log.join(",");
            })()
            "#,
        ),
        "g:Symbol(Symbol.iterator)"
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const blob = new Blob(["edge", new Uint8Array([33])], {type: "TEXT/PLAIN"});
              return `${blob.size}|${blob.type}`;
            })()
            "#,
        ),
        "5|text/plain"
    );
}

#[test]
fn request_init_is_snapshotted_before_url_validation_in_webidl_order() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const log = [];
              const init = new Proxy({}, {
                get(target, key, receiver) {
                  log.push(`g:${String(key)}`);
                  return Reflect.get(target, key, receiver);
                },
                has(target, key) {
                  log.push(`h:${String(key)}`);
                  return Reflect.has(target, key);
                }
              });
              try { new Request("ftp:", init); } catch (_) {}
              return log.join(",");
            })()
            "#,
        ),
        "g:adAuctionHeaders,g:attributionReporting,h:attributionReporting,g:body,g:browsingTopics,g:cache,g:credentials,g:duplex,g:headers,g:integrity,g:keepalive,g:method,g:mode,g:priority,g:privateToken,g:redirect,g:referrer,g:referrerPolicy,g:sharedStorageWritable,g:signal,g:targetAddressSpace"
    );
}

#[test]
fn fetch_init_is_snapshotted_before_url_validation_in_webidl_order() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const log = [];
              const init = new Proxy({}, {
                get(target, key, receiver) {
                  log.push(`g:${String(key)}`);
                  return Reflect.get(target, key, receiver);
                },
                has(target, key) {
                  log.push(`h:${String(key)}`);
                  return Reflect.has(target, key);
                }
              });
              fetch("ftp:", init).catch(() => {});
              return log.join(",");
            })()
            "#,
        ),
        "g:adAuctionHeaders,g:attributionReporting,h:attributionReporting,g:body,g:browsingTopics,g:cache,g:credentials,g:duplex,g:headers,g:integrity,g:keepalive,g:method,g:mode,g:priority,g:privateToken,g:redirect,g:referrer,g:referrerPolicy,g:sharedStorageWritable,g:signal,g:targetAddressSpace"
    );
}

#[test]
fn custom_android_app_request_init_uses_webview_feature_gates() {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.navigator.user_agent =
        "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.app_version =
        "8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.user_agent_data.mobile = true;
    fingerprint.navigator.user_agent_data.platform = "Android".to_owned();
    fingerprint.navigator.user_agent_data.ua_full_version = "149.0.7827.155".to_owned();
    fingerprint.navigator.user_agent_data.form_factors =
        vec!["Mobile".to_owned(), "WebView".to_owned()];
    let mut runtime = EdgeRuntime::with_fingerprint(fingerprint).expect("Android App runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const log = [];
              const init = new Proxy({}, {
                get(target, key, receiver) {
                  log.push(`g:${String(key)}`);
                  return Reflect.get(target, key, receiver);
                },
                has(target, key) {
                  log.push(`h:${String(key)}`);
                  return Reflect.has(target, key);
                }
              });
              try { new Request("ftp:", init); } catch (_) {}
              return log.join(",");
            })()
            "#,
        ),
        "g:attributionReporting,g:body,g:cache,g:credentials,g:duplex,g:headers,g:integrity,g:keepalive,g:method,g:mode,g:priority,g:privateToken,g:redirect,g:referrer,g:referrerPolicy,g:signal"
    );
}

#[test]
fn android_webview_console_preview_observes_only_native_to_string_tag_paths() {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.navigator.user_agent =
        "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.app_version =
        "8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.user_agent_data.mobile = true;
    fingerprint.navigator.user_agent_data.platform = "Android".to_owned();
    fingerprint.navigator.user_agent_data.ua_full_version = "136.0.0.0".to_owned();
    fingerprint.navigator.user_agent_data.form_factors =
        vec!["Mobile".to_owned(), "WebView".to_owned()];
    let mut runtime = EdgeRuntime::with_fingerprint(fingerprint).expect("Android WebView runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const run = method => {
                const tagged = value => {
                  let hits = 0;
                  Object.defineProperty(value, Symbol.toStringTag, {
                    configurable: true,
                    get() { hits += 1; return "Observed"; }
                  });
                  console[method](value);
                  return hits;
                };
                let proxyGets = 0;
                const prototype = new Proxy({}, {
                  get(target, key, receiver) {
                    if (key === Symbol.toStringTag) proxyGets += 1;
                    return Reflect.get(target, key, receiver);
                  }
                });
                console[method](Object.create(prototype));
                let directProxyGets = 0;
                console[method](new Proxy({}, {
                  get(target, key, receiver) {
                    directProxyGets += 1;
                    return Reflect.get(target, key, receiver);
                  }
                }));
                let nestedObject = 0;
                const child = {};
                Object.defineProperty(child, Symbol.toStringTag, {
                  get() { nestedObject += 1; return "Child"; }
                });
                console[method]({ child });
                let nestedArray = 0;
                const arrayChild = {};
                Object.defineProperty(arrayChild, Symbol.toStringTag, {
                  get() { nestedArray += 1; return "ArrayChild"; }
                });
                console[method]([arrayChild]);
                let throwingHits = 0;
                let escaped = false;
                const throwing = {};
                Object.defineProperty(throwing, Symbol.toStringTag, {
                  get() { throwingHits += 1; throw new RangeError("tag"); }
                });
                try { console[method](throwing); } catch (_) { escaped = true; }
                return [
                  tagged({}), tagged(new Uint8Array(1)), tagged(new Map()),
                  tagged(new Error("x")), tagged([]), tagged(function () {}),
                  tagged(new Date(0)), tagged(/x/), proxyGets, directProxyGets,
                  nestedObject, nestedArray, throwingHits, escaped
                ].join(",");
              };
              return run("log") + "|" + run("debug");
            })()
            "#,
        ),
        "1,1,1,0,0,0,0,0,1,0,0,1,2,false|1,1,1,0,0,0,0,0,1,0,0,1,2,false"
    );
}

#[test]
fn android_webview_officially_absent_get_details_globals_are_not_observable() {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.navigator.user_agent =
        "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.app_version =
        "8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.user_agent_data.mobile = true;
    fingerprint.navigator.user_agent_data.platform = "Android".to_owned();
    fingerprint.navigator.user_agent_data.ua_full_version = "136.0.0.0".to_owned();
    fingerprint.navigator.user_agent_data.form_factors =
        vec!["Mobile".to_owned(), "WebView".to_owned()];
    let mut runtime = EdgeRuntime::with_fingerprint(fingerprint).expect("Android WebView runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            [
              Object.getOwnPropertyNames(window).includes("chrome"),
              "chrome" in window,
              typeof window.chrome,
              Object.getOwnPropertyDescriptor(window, "chrome") === undefined,
              Object.getOwnPropertyNames(window).includes("getDigitalGoodsService"),
              "getDigitalGoodsService" in window,
              typeof window.getDigitalGoodsService,
              Object.getOwnPropertyDescriptor(window, "getDigitalGoodsService") === undefined,
              Object.getOwnPropertyNames(Performance.prototype).includes("interactionCount"),
              typeof Permissions,
              typeof PermissionStatus
            ].join("|")
            "#,
        ),
        "false|false|undefined|true|false|false|undefined|true|true|function|function"
    );
}

#[test]
fn iframe_outer_dimensions_can_be_configured_independently_from_root_window() {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.screen.viewport_width = 900.0;
    fingerprint.screen.viewport_height = 700.0;
    fingerprint.screen.outer_width = 1200.0;
    fingerprint.screen.outer_height = 800.0;
    fingerprint.screen.iframe_viewport_width = Some(0.0);
    fingerprint.screen.iframe_viewport_height = Some(0.0);
    fingerprint.screen.iframe_outer_width = Some(392.0);
    fingerprint.screen.iframe_outer_height = Some(654.0);
    let mut runtime = EdgeRuntime::with_fingerprint(fingerprint).expect("configured runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const first = document.createElement("iframe");
              const second = document.createElement("iframe");
              document.body.append(first, second);
              const before = [
                outerWidth,
                outerHeight,
                innerWidth,
                innerHeight,
                first.contentWindow.outerWidth,
                first.contentWindow.outerHeight,
                first.contentWindow.innerWidth,
                first.contentWindow.innerHeight,
                second.contentWindow.outerWidth,
                second.contentWindow.outerHeight,
                second.contentWindow.innerWidth,
                second.contentWindow.innerHeight
              ];
              first.contentWindow.resizeTo(500, 600);
              return before.concat([
                outerWidth,
                outerHeight,
                innerWidth,
                innerHeight,
                first.contentWindow.outerWidth,
                first.contentWindow.outerHeight,
                first.contentWindow.innerWidth,
                first.contentWindow.innerHeight,
                second.contentWindow.outerWidth,
                second.contentWindow.outerHeight,
                second.contentWindow.innerWidth,
                second.contentWindow.innerHeight
              ]).join("|");
            })()
            "#,
        ),
        concat!(
            "1200|800|900|700|392|654|0|0|392|654|0|0|",
            "1200|800|900|700|500|600|500|600|392|654|0|0"
        )
    );
}

#[test]
fn android_webview_136_plural_rules_matches_version_and_locale_fallback() {
    let mut fingerprint = EdgeFingerprint::default();
    fingerprint.locale.locale = "zh-CN".to_owned();
    fingerprint.navigator.user_agent =
        "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.app_version =
        "8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)".to_owned();
    fingerprint.navigator.language = "gn-PY".to_owned();
    fingerprint.navigator.languages = vec!["gn-PY".to_owned(), "es-PY".to_owned()];
    fingerprint.navigator.user_agent_data.mobile = true;
    fingerprint.navigator.user_agent_data.platform = "Android".to_owned();
    fingerprint.navigator.user_agent_data.ua_full_version = "136.0.0.0".to_owned();
    fingerprint.navigator.user_agent_data.form_factors =
        vec!["Mobile".to_owned(), "WebView".to_owned()];
    let mut runtime = EdgeRuntime::with_fingerprint(fingerprint).expect("Android WebView runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const defaults = new Intl.PluralRules().resolvedOptions();
              const requested = new Intl.PluralRules("gn-PY").resolvedOptions();
              const chinese = new Intl.PluralRules("zh-CN").resolvedOptions();
              const method = Intl.PluralRules.prototype.resolvedOptions;
              return [
                defaults.locale,
                "notation" in defaults,
                requested.locale,
                "notation" in requested,
                chinese.locale,
                "notation" in chinese,
                method.name,
                method.length,
                Function.prototype.toString.call(method)
              ].join("|");
            })()
            "#,
        ),
        "gn-PY|false|gn-PY|false|zh|false|resolvedOptions|0|function resolvedOptions() { [native code] }"
    );
}

#[test]
fn element_animate_starts_at_zero_overall_progress() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const animation = document.createElement("div").animate(
                [{ opacity: 0 }, { opacity: 1 }],
                { duration: 1000 }
              );
              const initial = animation.effect.getComputedTiming();
              const first = [
                animation.timeline.constructor.name,
                animation.playState,
                animation.currentTime,
                animation.overallProgress,
                initial.localTime,
                initial.progress,
                initial.currentIteration
              ].join("|");
              animation.currentTime = 500;
              const positioned = animation.effect.getComputedTiming();
              return `${first}|${animation.overallProgress}|${positioned.localTime}|${positioned.progress}|${positioned.currentIteration}`;
            })()
            "#,
        ),
        "DocumentTimeline|running|0|0|0|0|0|0.5|500|0.5|0"
    );
}

#[test]
fn document_timeline_tracks_the_realm_clock_and_origin_time() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let first = text(&mut runtime, "String(document.timeline.currentTime)")
        .parse::<f64>()
        .expect("initial document timeline time");
    std::thread::sleep(std::time::Duration::from_millis(20));
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const current = document.timeline.currentTime;
              const shifted = new DocumentTimeline({ originTime: 100 });
              const animation = document.createElement("div").animate(
                [{ opacity: 0 }, { opacity: 1 }],
                { duration: 1000 }
              );
              return [
                current > 0,
                Math.abs((current - 100) - shifted.currentTime) < 5,
                document.timeline.duration === null,
                animation.timeline === document.timeline,
                animation.currentTime === 0
              ].join("|");
            })()
            "#,
        ),
        "true|true|true|true|true"
    );
    let second = text(&mut runtime, "String(document.timeline.currentTime)")
        .parse::<f64>()
        .expect("later document timeline time");
    assert!(
        second > first,
        "document timeline must advance: {first} -> {second}"
    );
}

#[test]
fn desktop_document_create_event_rejects_touch_event() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              let failure;
              try { document.createEvent("TouchEvent"); }
              catch (error) { failure = `${error.name}|${error.code}|${error.message}`; }
              const constructed = new TouchEvent("touchstart");
              return `${failure}|${constructed instanceof TouchEvent}|${constructed.type}`;
            })()
            "#,
        ),
        "NotSupportedError|9|Failed to execute 'createEvent' on 'Document': The provided event type ('TouchEvent') is invalid.|true|touchstart"
    );
}

#[test]
fn android_document_create_event_supports_touch_and_touch_event() {
    for (label, user_agent, full_version, form_factors) in [
        (
            "Android Chrome",
            "Mozilla/5.0 (Linux; Android 15; Pixel 9 Pro) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Mobile Safari/537.36",
            "150.0.0.0",
            vec!["Mobile".to_owned()],
        ),
        (
            "Android WebView",
            "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)",
            "136.0.0.0",
            vec!["Mobile".to_owned(), "WebView".to_owned()],
        ),
    ] {
        let mut fingerprint = EdgeFingerprint::default();
        fingerprint.navigator.user_agent = user_agent.to_owned();
        fingerprint.navigator.app_version = user_agent.to_owned();
        fingerprint.navigator.user_agent_data.mobile = true;
        fingerprint.navigator.user_agent_data.platform = "Android".to_owned();
        fingerprint.navigator.user_agent_data.ua_full_version = full_version.to_owned();
        fingerprint.navigator.user_agent_data.form_factors = form_factors;
        let mut runtime = EdgeRuntime::with_fingerprint(fingerprint)
            .unwrap_or_else(|error| panic!("{label} runtime: {error}"));
        assert_eq!(
            text(
                &mut runtime,
                r##"
                (() => {
                  const inspect = document => ["Touch", "TouchEvent", "tOuCh"].map(name => {
                    const event = document.createEvent(name);
                    const initial = [
                      event.constructor.name,
                      event instanceof document.defaultView.TouchEvent,
                      event.type,
                      event.touches.length,
                      event.targetTouches.length,
                      event.changedTouches.length,
                      event.altKey,
                      event.metaKey,
                      event.ctrlKey,
                      event.shiftKey,
                      event.bubbles,
                      event.cancelable,
                      event.composed,
                      event.isTrusted
                    ].join(",");
                    event.initEvent("touchstart", true, true);
                    return initial + `:${event.type},${event.bubbles},${event.cancelable}`;
                  }).join("|");
                  const frame = document.createElement("iframe");
                  document.body.appendChild(frame);
                  return inspect(document) + "#" + inspect(frame.contentDocument);
                })()
                "##,
            ),
            concat!(
                "TouchEvent,true,,0,0,0,false,false,false,false,false,false,false,false:touchstart,true,true|",
                "TouchEvent,true,,0,0,0,false,false,false,false,false,false,false,false:touchstart,true,true|",
                "TouchEvent,true,,0,0,0,false,false,false,false,false,false,false,false:touchstart,true,true#",
                "TouchEvent,true,,0,0,0,false,false,false,false,false,false,false,false:touchstart,true,true|",
                "TouchEvent,true,,0,0,0,false,false,false,false,false,false,false,false:touchstart,true,true|",
                "TouchEvent,true,,0,0,0,false,false,false,false,false,false,false,false:touchstart,true,true"
            ),
            "{label}",
        );
    }
}

#[test]
fn media_can_play_type_does_not_infer_unknown_codecs_from_a_container() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const media = document.createElement("video");
              return [
                media.canPlayType('video/mp4; codecs="avc1.42E01E"'),
                media.canPlayType('video/mp4'),
                media.canPlayType('video/mp4; codecs=bogus'),
                media.canPlayType('video/ogg; codecs=opus'),
                media.canPlayType('video/ogg; codecs=theora')
              ].join("|");
            })()
            "#,
        ),
        "probably|maybe||probably|"
    );
}

#[test]
fn offline_audio_context_suspends_on_render_quantum_and_resumes_to_completion() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.offlineSuspendAnswer = "pending";
            const context = new OfflineAudioContext(1, 1024, 44100);
            const states = [];
            const suspension = context.suspend(512 / 44100).then(() => {
              states.push(context.state);
              states.push(context.currentTime === 512 / 44100);
              return context.resume();
            });
            const rendering = context.startRendering();
            Promise.all([suspension, rendering]).then(([, buffer]) => {
              states.push(context.state);
              states.push(context.currentTime === 1024 / 44100);
              states.push(buffer.length === 1024);
              return context.resume().then(
                () => states.push(false),
                error => states.push(error.name === "InvalidStateError")
              );
            }).then(() => {
              offlineSuspendAnswer = [
                states[0] === "suspended",
                states[1] === true,
                states[2] === "closed",
                states[3] === true,
                states[4] === true,
                states[5] === true
              ].every(Boolean);
            });
            "#,
        )
        .expect("Offline suspension setup");
    assert_eq!(text(&mut runtime, "offlineSuspendAnswer"), "true");
}

#[test]
fn offline_web_audio_dsp_processes_delay_waveshaper_convolver_and_analyser() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.offlineDspAnswer = "pending";

            const delayedContext = new OfflineAudioContext(1, 384, 44100);
            const delayedSource = delayedContext.createConstantSource();
            const delay = delayedContext.createDelay(1);
            const analyser = delayedContext.createAnalyser();
            delay.delayTime.value = 128 / 44100;
            delayedSource.connect(delay).connect(analyser).connect(delayedContext.destination);
            delayedSource.start();

            const shapedContext = new OfflineAudioContext(1, 32, 44100);
            const shapedSource = shapedContext.createConstantSource();
            const shaper = shapedContext.createWaveShaper();
            shapedSource.offset.value = 0.5;
            shaper.curve = new Float32Array([-1, -0.25, 0, 0.25, 1]);
            shapedSource.connect(shaper).connect(shapedContext.destination);
            shapedSource.start();

            const convolvedContext = new OfflineAudioContext(1, 32, 44100);
            const sourceBuffer = convolvedContext.createBuffer(1, 32, 44100);
            sourceBuffer.getChannelData(0)[0] = 1;
            const impulse = convolvedContext.createBuffer(1, 2, 44100);
            impulse.getChannelData(0).set([0.5, 0.25]);
            const bufferSource = convolvedContext.createBufferSource();
            const convolver = convolvedContext.createConvolver();
            bufferSource.buffer = sourceBuffer;
            convolver.buffer = impulse;
            convolver.normalize = false;
            bufferSource.connect(convolver).connect(convolvedContext.destination);
            bufferSource.start();

            Promise.all([
              delayedContext.startRendering(),
              shapedContext.startRendering(),
              convolvedContext.startRendering()
            ]).then(([delayed, shaped, convolved]) => {
              const delaySamples = delayed.getChannelData(0);
              const analyserData = new Float32Array(analyser.fftSize);
              analyser.getFloatTimeDomainData(analyserData);
              const shapedSamples = shaped.getChannelData(0);
              const convolvedSamples = convolved.getChannelData(0);
              offlineDspAnswer = [
                Math.abs(delaySamples[0]) < 0.01,
                delaySamples[128] > 0.9,
                analyserData.some(value => value > 0.9),
                shapedSamples[8] > 0.2 && shapedSamples[8] < 0.3,
                Math.abs(convolvedSamples[0] - 0.5) < 0.01,
                Math.abs(convolvedSamples[1] - 0.25) < 0.01
              ].every(Boolean);
            });
            "#,
        )
        .expect("Offline DSP setup");
    assert_eq!(text(&mut runtime, "offlineDspAnswer"), "true");
}

#[test]
fn decode_audio_data_decodes_pcm_wave_resamples_and_invokes_callback() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.decodeAudioAnswer = "pending";
            globalThis.decodeAudioCallback = false;
            const samples = new Int16Array([0, 16384, -16384, 32767]);
            const wave = new ArrayBuffer(44 + samples.byteLength);
            const view = new DataView(wave);
            const text = (offset, value) => {
              for (let index = 0; index < value.length; index++) {
                view.setUint8(offset + index, value.charCodeAt(index));
              }
            };
            text(0, "RIFF");
            view.setUint32(4, 36 + samples.byteLength, true);
            text(8, "WAVE");
            text(12, "fmt ");
            view.setUint32(16, 16, true);
            view.setUint16(20, 1, true);
            view.setUint16(22, 1, true);
            view.setUint32(24, 22050, true);
            view.setUint32(28, 44100, true);
            view.setUint16(32, 2, true);
            view.setUint16(34, 16, true);
            text(36, "data");
            view.setUint32(40, samples.byteLength, true);
            new Int16Array(wave, 44).set(samples);

            const context = new AudioContext({sampleRate: 44100});
            context.decodeAudioData(
              wave,
              () => decodeAudioCallback = true
            ).then(buffer => {
              const channel = buffer.getChannelData(0);
              decodeAudioAnswer = [
                buffer.numberOfChannels === 1,
                buffer.sampleRate === 44100,
                buffer.length === 8,
                channel[2] > 0.49 && channel[2] < 0.51,
                channel[4] < -0.49 && channel[4] > -0.51,
                decodeAudioCallback
              ].every(Boolean);
              return context.close();
            });
            "#,
        )
        .expect("decodeAudioData setup");
    assert_eq!(text(&mut runtime, "decodeAudioAnswer"), "true");
}

#[test]
fn proxy_trace_records_concrete_audio_nodes_without_shape_drift() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let shape_source = r#"
        [
          Function.prototype.toString.call(BaseAudioContext.prototype.createGain),
          Function.prototype.toString.call(
            Object.getOwnPropertyDescriptor(GainNode.prototype, "gain").get
          ),
          Object.getOwnPropertyNames(GainNode.prototype).join(","),
          Object.getOwnPropertyNames(DynamicsCompressorNode.prototype).join(","),
          Function.prototype.toString.call(AudioNode.prototype.connect),
          Function.prototype.toString.call(AudioParam.prototype.setValueAtTime),
          Object.getPrototypeOf(GainNode.prototype) === AudioNode.prototype
        ].join("|")
    "#;
    let before = text(&mut runtime, shape_source);
    runtime.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut runtime, shape_source), before);
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const context = new AudioContext();
              const gain = context.createGain();
              const compressor = context.createDynamicsCompressor();
              gain.gain.value = 0.5;
              gain.gain.setValueAtTime(0.25, context.currentTime + 1);
              gain.connect(compressor);
              return [
                gain instanceof GainNode,
                compressor instanceof DynamicsCompressorNode,
                gain.gain.value,
                compressor.reduction,
                Function.prototype.toString.call(
                  BaseAudioContext.prototype.createGain
                )
              ].join("|");
            })()
            "#,
        ),
        "true|true|0.5|0|function createGain() { [native code] }"
    );
    let trace = runtime.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.ends_with("createGain")
            && entry.result.contains("GainNode")
    }));
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "get" && entry.api.ends_with("createGain().gain") })
    );
    assert!(trace.iter().any(|entry| {
        entry.operation == "set" && entry.api.ends_with("createGain().gain.value")
    }));
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "get" && entry.api.ends_with(".reduction") })
    );
    assert!(
        trace.iter().any(|entry| {
            entry.operation == "call" && entry.api.ends_with(".gain.setValueAtTime")
        })
    );
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "call" && entry.api.ends_with(".connect"))
    );
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "get" && entry.api.ends_with(".currentTime"))
    );
}

#[test]
fn performance_timeline_observer_queue_and_measure_options_are_functional() {
    let setup = r#"
        (() => {
          globalThis.performanceObserverAnswer = "pending";
          globalThis.performanceBufferedAnswer = "pending";
          globalThis.performanceTakeRecordsCallbackCount = 0;
          const observer = new PerformanceObserver(
            (list, current, options) => {
              performanceObserverAnswer = [
                list.getEntries().map(entry => entry.name).join(","),
                current === observer,
                options.droppedEntriesCount
              ].join("|");
            }
          );
          observer.observe({ entryTypes: ["mark", "measure"] });
          const first = performance.mark("a", {
            startTime: 2,
            detail: "first-detail"
          });
          performance.mark("b", { startTime: 7 });
          const span = performance.measure("span", {
            start: "a",
            end: "b",
            detail: "measure-detail"
          });
          const takeObserver = new PerformanceObserver(
            () => performanceTakeRecordsCallbackCount++
          );
          takeObserver.observe({ type: "mark" });
          performance.mark("taken", { startTime: 9 });
          const taken = takeObserver.takeRecords();
          const bufferedObserver = new PerformanceObserver(list => {
            performanceBufferedAnswer =
              list.getEntries().map(entry => entry.name).join(",");
          });
          bufferedObserver.observe({ type: "measure", buffered: true });
          return [
            performanceObserverAnswer === "pending",
            performance.getEntriesByType("mark").length,
            performance.getEntriesByName("span", "measure").length,
            first instanceof PerformanceMark,
            span instanceof PerformanceMeasure,
            span.startTime,
            span.duration,
            span.detail,
            taken.length,
            taken[0] instanceof PerformanceMark,
            Function.prototype.toString.call(
              PerformanceObserver.prototype.observe
            )
          ].join("|");
        })()
    "#;
    let expected_immediate = concat!(
        "true|3|1|true|true|2|5|measure-detail|1|true|",
        "function observe() { [native code] }"
    );
    let expected_callback = "a,span,b,taken|true|0";

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    assert_eq!(text(&mut direct, setup), expected_immediate);
    assert_eq!(
        text(
            &mut direct,
            "[performanceObserverAnswer,performanceTakeRecordsCallbackCount,performanceBufferedAnswer].join('!')"
        ),
        format!("{expected_callback}!0!span")
    );
    assert_eq!(
        text(
            &mut direct,
            "performance.clearMarks(); performance.clearMeasures(); performance.getEntries().length"
        ),
        "2"
    );

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, setup), expected_immediate);
    assert_eq!(
        text(
            &mut traced,
            "[performanceObserverAnswer,performanceTakeRecordsCallbackCount,performanceBufferedAnswer].join('!')"
        ),
        format!("{expected_callback}!0!span")
    );
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.ends_with(".performance.mark")
            && entry.result.contains("PerformanceMark")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.ends_with(".performance.measure")
            && entry.result.contains("PerformanceMeasure")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("PerformanceObserver")
            && entry.api.ends_with(".takeRecords")
    }));
}

#[test]
fn user_timing_matches_edge_errors_clone_semantics_and_chronological_order() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const error = callback => {
            try { callback(); return "ok"; }
            catch (value) {
              return [value.name, value.constructor.name, value.code ?? "-"].join(":");
            }
          };
          performance.clearMarks();
          performance.clearMeasures();
          const output = [
            error(() => performance.measure("missing", "absent-mark")),
            error(() => performance.measure("duration-only", {duration: 1})),
            error(() => performance.measure("all", {start: 0, duration: 1, end: 2})),
            error(() => performance.measure("negative", {start: -1, end: 2})),
            error(() => performance.mark("navigationStart")),
            error(() => new PerformanceMark("navigationStart")),
            error(() => performance.mark("nan", {startTime: NaN})),
            error(() => performance.mark("function-detail", {detail() {}})),
            error(() => performance.measure("function-detail", {start: 0, end: 1, detail() {}}))
          ];

          const cycle = {};
          cycle.self = cycle;
          const cloned = performance.mark("cycle", {detail: cycle}).detail;
          output.push(cloned !== cycle && cloned.self === cloned);

          const markOrder = [];
          const markOptions = {};
          Object.defineProperties(markOptions, {
            startTime: {get() { markOrder.push("startTime"); return 3; }},
            detail: {get() { markOrder.push("detail"); return null; }}
          });
          performance.mark("getter-mark", markOptions);
          output.push(markOrder.join(","));

          const measureOrder = [];
          const measureOptions = {};
          for (const [name, value] of [
            ["start", 1], ["end", 2], ["duration", undefined], ["detail", null]
          ]) {
            Object.defineProperty(measureOptions, name, {
              get() { measureOrder.push(name); return value; }
            });
          }
          performance.measure("getter-measure", measureOptions);
          output.push(measureOrder.join(","));

          performance.clearMarks();
          performance.mark("empty-options-end", {startTime: 4});
          output.push(performance.measure("empty-options", {}, "empty-options-end").duration);
          performance.clearMarks();
          performance.mark("sort-late", {startTime: 20});
          performance.mark("sort-early", {startTime: 10});
          output.push(performance.getEntriesByType("mark").map(v => v.name).join(","));
          performance.mark("same", {startTime: 30});
          performance.mark("same", {startTime: 5});
          output.push(performance.getEntriesByName("same", "mark").map(v => v.startTime).join(","));
          output.push(performance.getEntries().filter(v => v.name.startsWith("sort-"))
            .map(v => v.name).join(","));
          return output.join("|");
        })()
        "#,
    );
    assert_eq!(
        answer,
        concat!(
            "SyntaxError:DOMException:12|TypeError:TypeError:-|TypeError:TypeError:-|",
            "TypeError:TypeError:-|SyntaxError:DOMException:12|SyntaxError:DOMException:12|",
            "TypeError:TypeError:-|",
            "DataCloneError:DOMException:25|DataCloneError:DOMException:25|true|",
            "detail,startTime|detail,duration,end,start|4|sort-early,sort-late|5,30|",
            "sort-early,sort-late"
        )
    );
}

#[test]
fn url_search_params_uses_webidl_union_conversion_and_live_pair_iterators() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const errorName = callback => {
            try { callback(); return "ok"; }
            catch (error) { return error.name; }
          };
          const output = [];
          output.push(Reflect.ownKeys(URLSearchParams.prototype).map(String).join(","));
          output.push(new URLSearchParams(new Map([["a", "1"], ["a", "2"]])).toString());
          output.push(new URLSearchParams((function* () {
            yield (function* () { yield "x"; yield "7"; })();
          })()).toString());
          output.push(errorName(() => new URLSearchParams(["ab"])));
          output.push(errorName(() => new URLSearchParams([["a", "1", "extra"]])));
          output.push(errorName(() => new URLSearchParams([["a"]])));
          output.push(errorName(() => new URLSearchParams([{}])));
          output.push(new URLSearchParams(null).toString());
          output.push(new URLSearchParams(12).toString());

          const record = {};
          Object.defineProperty(record, "hidden", {value: "no", enumerable: false});
          record.visible = "yes";
          output.push(new URLSearchParams(record).toString());
          output.push(errorName(() => new URLSearchParams({[Symbol("key")]: "value"})));

          const live = new URLSearchParams("a=1&b=2");
          const iterator = live.entries();
          const prototype = Object.getPrototypeOf(iterator);
          const next = Object.getOwnPropertyDescriptor(prototype, "next");
          const tag = Object.getOwnPropertyDescriptor(prototype, Symbol.toStringTag);
          output.push([
            Object.prototype.toString.call(iterator),
            Reflect.ownKeys(prototype).map(String).join(","),
            [next.writable, next.enumerable, next.configurable, next.value.name, next.value.length].join(","),
            [tag.writable, tag.enumerable, tag.configurable].join(","),
            iterator[Symbol.iterator]() === iterator,
            Object.getPrototypeOf(prototype) ===
              Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]())),
            Function.prototype.toString.call(next.value)
          ].join("~"));
          output.push(iterator.next().value.join(":"));
          live.append("c", "3");
          output.push(iterator.next().value.join(":"));
          output.push(iterator.next().value.join(":"));

          const deleted = new URLSearchParams("a=1&b=2");
          const deletedIterator = deleted.keys();
          deletedIterator.next();
          deleted.delete("b");
          output.push(JSON.stringify(deletedIterator.next()));

          const each = new URLSearchParams("a=1&b=2");
          const calls = [];
          each.forEach((value, name, receiver) => {
            calls.push(`${name}:${value}:${receiver === each}`);
            if (name === "a") each.append("c", "3");
          });
          output.push(calls.join(","));

          const deleteDuringEach = new URLSearchParams("a=1&b=2&c=3");
          const deleteCalls = [];
          deleteDuringEach.forEach((value, name) => {
            deleteCalls.push(name);
            if (name === "a") deleteDuringEach.delete("b");
          });
          output.push(deleteCalls.join(","));
          return output.join("|");
        })()
        "#,
    );
    assert_eq!(
        answer,
        concat!(
            "size,append,delete,get,getAll,has,set,sort,toString,entries,forEach,keys,values,",
            "constructor,Symbol(Symbol.toStringTag),Symbol(Symbol.iterator)|",
            "a=2|x=7|TypeError|TypeError|TypeError|TypeError|null=|12=|visible=yes|TypeError|",
            "[object URLSearchParams Iterator]~next,Symbol(Symbol.toStringTag)~",
            "true,true,true,next,0~false,false,true~true~true~function next() { [native code] }|",
            "a:1|b:2|c:3|{\"done\":true}|",
            "a:1:true,b:2:true,c:3:true|a,c"
        )
    );

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(
        text(
            &mut traced,
            "new URLSearchParams('trace=live').entries().next().value.join('=')"
        ),
        "trace=live"
    );
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("URLSearchParams")
            && entry.api.ends_with(".entries")
    }));
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".next") })
    );
}

#[test]
fn headers_validate_bytestrings_and_expose_live_sorted_pair_iterators() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const errorName = callback => {
            try { callback(); return "ok"; }
            catch (error) { return error.name; }
          };
          const output = [];
          output.push(Reflect.ownKeys(Headers.prototype).map(String).join(","));
          output.push(JSON.stringify(Array.from(
            new Headers(new Map([["B", "2"], ["a", "1"]]))
          )));
          output.push(JSON.stringify(Array.from(new Headers((function* () {
            yield (function* () { yield "X-One"; yield "  value\t"; })();
          })()))));
          output.push(errorName(() => new Headers([["a", "1", "extra"]])));
          output.push(errorName(() => new Headers([["a"]])));
          output.push(errorName(() => new Headers([{}])));
          output.push(errorName(() => new Headers(null)));
          output.push(errorName(() => new Headers(12)));
          output.push(errorName(() => new Headers([[" x ", "1"]])));
          output.push(errorName(() => new Headers([["x", "a\nb"]])));
          output.push(errorName(() => new Headers([["x", "a\0b"]])));
          output.push(errorName(() => new Headers([["😀", "1"]])));
          output.push(errorName(() => new Headers([["x", "😀"]])));
          output.push(errorName(() => new Headers().append()));
          output.push(errorName(() => new Headers().set("x")));
          output.push(new Headers([["x", "é"]]).get("x"));

          const record = {};
          Object.defineProperty(record, "hidden", {value: "no", enumerable: false});
          record.Visible = "  yes\t";
          output.push(JSON.stringify(Array.from(new Headers(record))));
          output.push(errorName(() => new Headers({[Symbol("key")]: "value"})));

          const duplicates = new Headers();
          duplicates.append("X-B", "2");
          duplicates.append("x-a", "1");
          duplicates.append("X-B", "3");
          duplicates.append("Set-Cookie", "a=1");
          duplicates.append("set-cookie", "b=2");
          output.push(duplicates.get("x-b"));
          output.push(duplicates.getSetCookie().join(","));
          output.push(JSON.stringify(Array.from(duplicates)));

          const live = new Headers([["a", "1"], ["c", "3"]]);
          const iterator = live.entries();
          const prototype = Object.getPrototypeOf(iterator);
          const next = Object.getOwnPropertyDescriptor(prototype, "next");
          const tag = Object.getOwnPropertyDescriptor(prototype, Symbol.toStringTag);
          output.push([
            Object.prototype.toString.call(iterator),
            Reflect.ownKeys(prototype).map(String).join(","),
            [next.writable, next.enumerable, next.configurable, next.value.name, next.value.length].join(","),
            [tag.writable, tag.enumerable, tag.configurable].join(","),
            iterator[Symbol.iterator]() === iterator,
            Object.getPrototypeOf(prototype) ===
              Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]())),
            Function.prototype.toString.call(next.value)
          ].join("~"));
          output.push(iterator.next().value.join(":"));
          live.append("b", "2");
          output.push(iterator.next().value.join(":"));
          output.push(iterator.next().value.join(":"));

          const each = new Headers([["a", "1"], ["c", "3"]]);
          const calls = [];
          each.forEach((value, name, receiver) => {
            calls.push(`${name}:${value}:${receiver === each}`);
            if (name === "a") each.append("b", "2");
          });
          output.push(calls.join(","));
          return output.join("|");
        })()
        "#,
    );
    assert_eq!(
        answer,
        concat!(
            "append,delete,get,getSetCookie,has,set,entries,forEach,keys,values,constructor,",
            "Symbol(Symbol.toStringTag),Symbol(Symbol.iterator)|",
            "[[\"a\",\"1\"],[\"b\",\"2\"]]|[[\"x-one\",\"value\"]]|",
            "TypeError|TypeError|TypeError|TypeError|TypeError|TypeError|TypeError|TypeError|",
            "TypeError|TypeError|TypeError|TypeError|é|[[\"visible\",\"yes\"]]|",
            "TypeError|2, 3|a=1,b=2|",
            "[[\"set-cookie\",\"a=1\"],[\"set-cookie\",\"b=2\"],[\"x-a\",\"1\"],[\"x-b\",\"2, 3\"]]|",
            "[object Headers Iterator]~next,Symbol(Symbol.toStringTag)~",
            "true,true,true,next,0~false,false,true~true~true~function next() { [native code] }|",
            "a:1|b:2|c:3|a:1:true,b:2:true,c:3:true"
        )
    );

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(
        text(
            &mut traced,
            "new Headers([['trace','live']]).entries().next().value.join('=')"
        ),
        "trace=live"
    );
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("Headers")
            && entry.api.ends_with(".entries")
    }));
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "call" && entry.api.ends_with(".next"))
    );
}

#[test]
fn edge150_headers_guards_filter_contextual_names_and_preserve_clone_state() {
    let source = r#"
      (async () => {
        const outcome = callback => {
          try { callback(); return ['return', null]; }
          catch (error) { return [error.name, error.message]; }
        };
        const mutate = (headers, operation, name, value) => {
          const result = outcome(() => operation === 'delete'
            ? headers.delete(name) : headers[operation](name, value));
          return [result, Array.from(headers)];
        };

        const request = new Request('https://audit.example/request', {
          headers: [['x-start','one'],['cookie','drop'],['sec-start','drop']]
        });
        const requestInitial = Array.from(request.headers);
        const requestMutations = [
          mutate(request.headers,'set','x-ok','two'),
          mutate(request.headers,'set','cookie','drop'),
          mutate(request.headers,'append','host','drop'),
          mutate(request.headers,'set','content-length','3')
        ];
        const requestCopy = mutate(new Headers(request.headers),'set','cookie','copy');
        const requestClone = mutate(request.clone().headers,'set','cookie','clone');

        const noCors = new Request('https://audit.example/no-cors', {
          mode:'no-cors', headers:[['accept','text/plain'],['x-drop','one'],
            ['content-type','text/plain;charset=UTF-8']]
        });
        const noCorsInitial = Array.from(noCors.headers);
        const noCorsMutations = [
          mutate(noCors.headers,'set','accept-language','en-US'),
          mutate(noCors.headers,'set','x-test','drop'),
          mutate(noCors.headers,'set','content-type','application/json'),
          mutate(noCors.headers,'set','content-type','multipart/form-data; boundary=x'),
          mutate(noCors.headers,'set','range','bytes=0-10')
        ];

        const response = new Response('ok', {headers:[['x-start','one'],['set-cookie','a=1']]});
        const responseInitial = [Array.from(response.headers), response.headers.getSetCookie()];
        const responseMutations = [
          mutate(response.headers,'set','x-ok','two'),
          mutate(response.headers,'set','set-cookie','drop')
        ];

        const fetched = await fetch('data:text/plain;charset=utf-8,hello');
        const immutable = [
          mutate(fetched.headers,'set','x-test','one'),
          outcome(() => fetched.headers.set('bad name','x')),
          outcome(() => fetched.headers.set('x-test','a\nb')),
          mutate(new Headers(fetched.headers),'set','x-test','copy')
        ];
        const staticGuards = [
          mutate(Response.error().headers,'set','x-test','error'),
          mutate(Response.redirect('https://audit.example/next').headers,'set','x-test','redirect'),
          mutate(Response.json({ok:true}).headers,'set','x-test','json')
        ];
        return JSON.stringify({requestInitial,requestMutations,requestCopy,requestClone,
          noCorsInitial,noCorsMutations,responseInitial,responseMutations,immutable,staticGuards});
      })()
    "#;
    let expected = r####"{"requestInitial":[["x-start","one"]],"requestMutations":[[["return",null],[["x-ok","two"],["x-start","one"]]],[["return",null],[["x-ok","two"],["x-start","one"]]],[["return",null],[["x-ok","two"],["x-start","one"]]],[["return",null],[["x-ok","two"],["x-start","one"]]]],"requestCopy":[["return",null],[["cookie","copy"],["x-ok","two"],["x-start","one"]]],"requestClone":[["return",null],[["x-ok","two"],["x-start","one"]]],"noCorsInitial":[["accept","text/plain"],["content-type","text/plain;charset=UTF-8"]],"noCorsMutations":[[["return",null],[["accept","text/plain"],["accept-language","en-US"],["content-type","text/plain;charset=UTF-8"]]],[["return",null],[["accept","text/plain"],["accept-language","en-US"],["content-type","text/plain;charset=UTF-8"]]],[["return",null],[["accept","text/plain"],["accept-language","en-US"],["content-type","text/plain;charset=UTF-8"]]],[["return",null],[["accept","text/plain"],["accept-language","en-US"],["content-type","multipart/form-data; boundary=x"]]],[["return",null],[["accept","text/plain"],["accept-language","en-US"],["content-type","multipart/form-data; boundary=x"]]]],"responseInitial":[[["content-type","text/plain;charset=UTF-8"],["x-start","one"]],[]],"responseMutations":[[["return",null],[["content-type","text/plain;charset=UTF-8"],["x-ok","two"],["x-start","one"]]],[["return",null],[["content-type","text/plain;charset=UTF-8"],["x-ok","two"],["x-start","one"]]]],"immutable":[[["TypeError","Failed to execute 'set' on 'Headers': Headers are immutable"],[["content-type","text/plain;charset=utf-8"]]],["TypeError","Failed to execute 'set' on 'Headers': Invalid name"],["TypeError","Failed to execute 'set' on 'Headers': Invalid value"],[["return",null],[["content-type","text/plain;charset=utf-8"],["x-test","copy"]]]],"staticGuards":[[["TypeError","Failed to execute 'set' on 'Headers': Headers are immutable"],[]],[["TypeError","Failed to execute 'set' on 'Headers': Headers are immutable"],[["location","https://audit.example/next"]]],[["return",null],[["content-type","application/json"],["x-test","json"]]]]}"####;

    let mut direct = EdgeRuntime::new().expect("direct Headers guard runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::new().expect("traced Headers guard runtime");
    traced.enable_proxy_trace().expect("enable native trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn network_replay_drives_fetch_xhr_and_module_loading() {
    let mut options = EdgeRuntimeOptions::default();
    let mut fetch_entry =
        NetworkReplayEntry::get("https://sandbox.test/data.txt", b"offline-body".to_vec());
    fetch_entry
        .headers
        .push(("content-type".to_owned(), "text/plain".to_owned()));
    options.network_replay.push(fetch_entry);
    let mut runtime = EdgeRuntime::with_options(options).expect("replay runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.fetchReplayAnswer = "pending";
            fetch("https://sandbox.test/data.txt")
              .then(response => response.text())
              .then(text => fetchReplayAnswer = text);
            "#,
        )
        .expect("fetch replay");
    assert_eq!(text(&mut runtime, "fetchReplayAnswer"), "offline-body");

    let answer = text(
        &mut runtime,
        r#"
        const xhr = new XMLHttpRequest();
        xhr.open("GET", "https://sandbox.test/data.txt", false);
        xhr.send();
        [xhr.status, xhr.responseText, xhr.getResponseHeader("content-type")].join("|")
        "#,
    );
    assert_eq!(answer, "200|offline-body|text/plain");
}

#[test]
fn image_loading_decode_events_and_resource_timing_match_edge_state_transitions() {
    let image_body =
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="7" height="11"></svg>"#.to_vec();
    let broken_body = b"not-an-image".to_vec();
    let mut options = EdgeRuntimeOptions::default();
    let mut image_entry =
        NetworkReplayEntry::get("https://sandbox.test/assets/image.svg", image_body.clone());
    image_entry
        .headers
        .push(("content-type".to_owned(), "image/svg+xml".to_owned()));
    let mut broken_entry = NetworkReplayEntry::get(
        "https://sandbox.test/assets/broken.png",
        broken_body.clone(),
    );
    broken_entry
        .headers
        .push(("content-type".to_owned(), "image/png".to_owned()));
    options.network_replay = vec![image_entry, broken_entry];
    let source = r##"
        (async () => {
          const empty = new Image();
          let emptyDecode = "";
          try {
            await empty.decode();
            emptyDecode = "resolved";
          } catch (error) {
            emptyDecode = error.name + ":" + error.message;
          }

          const image = new Image();
          const imageEvents = [];
          image.addEventListener("load", () => imageEvents.push("load"));
          image.addEventListener("error", () => imageEvents.push("error"));
          image.src = "/assets/image.svg";
          const immediate = [
            image.complete,
            image.currentSrc,
            image.naturalWidth,
            image.naturalHeight
          ].join(",");
          await image.decode().then(() => imageEvents.push("decode"));
          await new Promise(resolve => setTimeout(resolve, 0));
          const resource = performance.getEntriesByName(
            "https://sandbox.test/assets/image.svg",
            "resource"
          )[0];

          const broken = new Image();
          const brokenEvents = [];
          broken.addEventListener("load", () => brokenEvents.push("load"));
          broken.addEventListener("error", () => brokenEvents.push("error"));
          broken.src = "/assets/broken.png";
          const brokenImmediate = [
            broken.complete,
            broken.currentSrc,
            broken.naturalWidth,
            broken.naturalHeight
          ].join(",");
          let brokenDecode = "";
          try {
            await broken.decode().catch(error => {
              brokenEvents.push("decode");
              throw error;
            });
            brokenDecode = "resolved";
          } catch (error) {
            brokenDecode = error.name + ":" + error.message;
          }
          await new Promise(resolve => setTimeout(resolve, 0));
          const brokenResource = performance.getEntriesByName(
            "https://sandbox.test/assets/broken.png",
            "resource"
          )[0];

          return [
            empty.complete,
            empty.currentSrc,
            empty.naturalWidth,
            empty.naturalHeight,
            emptyDecode,
            immediate,
            image.complete,
            image.currentSrc,
            image.naturalWidth,
            image.naturalHeight,
            image.width,
            image.height,
            imageEvents.join(","),
            resource instanceof PerformanceResourceTiming,
            resource.entryType,
            resource.initiatorType,
            resource.responseStatus,
            resource.contentType,
            resource.encodedBodySize,
            resource.decodedBodySize,
            resource.transferSize,
            resource.responseEnd >= resource.startTime,
            brokenImmediate,
            broken.complete,
            broken.currentSrc,
            broken.naturalWidth,
            broken.naturalHeight,
            brokenDecode,
            brokenEvents.join(","),
            brokenResource.responseStatus,
            brokenResource.contentType,
            brokenResource.encodedBodySize
          ].join("|");
        })()
    "##;
    let expected = format!(
        concat!(
            "true||0|0|EncodingError:The source image cannot be decoded.|",
            "false,,0,0|true|https://sandbox.test/assets/image.svg|7|11|7|11|",
            "decode,load|true|resource|img|200|image/svg+xml|{0}|{0}|{1}|true|",
            "false,,0,0|true|https://sandbox.test/assets/broken.png|0|0|",
            "EncodingError:The source image cannot be decoded.|decode,error|200|image/png|{2}"
        ),
        image_body.len(),
        image_body.len() + 300,
        broken_body.len()
    );

    let mut direct = EdgeRuntime::with_options(options.clone()).expect("direct image runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::with_options(options).expect("traced image runtime");
    traced.enable_proxy_trace().expect("enable image trace");
    assert_eq!(text(&mut traced, source), expected);
    let trace = traced.proxy_trace();
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "call" && entry.api.ends_with(".decode"))
    );
    assert!(trace.iter().any(|entry| {
        entry.operation == "get" && entry.api == "HTMLImageElement.prototype.naturalWidth"
    }));
}

#[test]
fn image_srcset_selection_and_density_correction_follow_edge_dpr_semantics() {
    let one_x =
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="13" height="17"></svg>"#.to_vec();
    let two_x =
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="26" height="34"></svg>"#.to_vec();
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.device_pixel_ratio = 2.0;
    options.fingerprint.screen.viewport_width = 1280.0;
    for (url, body) in [
        ("https://sandbox.test/assets/image-1x.svg", one_x),
        ("https://sandbox.test/assets/image-2x.svg", two_x),
    ] {
        let mut entry = NetworkReplayEntry::get(url, body);
        entry
            .headers
            .push(("content-type".to_owned(), "image/svg+xml".to_owned()));
        options.network_replay.push(entry);
    }
    let source = r##"
        (async () => {
          const image = new Image();
          image.src = "/assets/image-1x.svg";
          image.srcset =
            "/assets/image-1x.svg 1x, /assets/image-2x.svg 2x";
          const immediate = [
            image.complete,
            image.currentSrc,
            image.naturalWidth,
            image.naturalHeight
          ].join(",");
          await image.decode();
          const detached = [image.width, image.height].join(",");
          document.body.appendChild(image);
          const connected = [image.width, image.height].join(",");
          return [
            devicePixelRatio,
            immediate,
            image.complete,
            image.src,
            image.currentSrc,
            image.naturalWidth,
            image.naturalHeight,
            detached,
            connected
          ].join("|");
        })()
    "##;
    let expected = concat!(
        "2|false,,0,0|true|https://sandbox.test/assets/image-1x.svg|",
        "https://sandbox.test/assets/image-2x.svg|13|17|26,34|13,17"
    );
    let mut direct = EdgeRuntime::with_options(options.clone()).expect("direct srcset runtime");
    assert_eq!(text(&mut direct, source), expected);
    let mut traced = EdgeRuntime::with_options(options).expect("traced srcset runtime");
    traced.enable_proxy_trace().expect("enable srcset trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn data_and_blob_images_decode_without_creating_network_resource_entries() {
    let source = r#"
        (async () => {
          const dataUrl =
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' " +
            "width='5' height='9'/%3E";
          const dataImage = new Image();
          const dataEvents = [];
          dataImage.onload = () => dataEvents.push("load");
          dataImage.onerror = () => dataEvents.push("error");
          dataImage.src = dataUrl;
          const dataImmediate = [
            dataImage.complete,
            dataImage.currentSrc,
            dataImage.naturalWidth,
            dataImage.naturalHeight
          ].join(",");
          await dataImage.decode().then(() => dataEvents.push("decode"));

          const blob = new Blob([
            "<svg xmlns='http://www.w3.org/2000/svg' width='8' height='6'/>"
          ], {type: "image/svg+xml"});
          const blobUrl = URL.createObjectURL(blob);
          const blobImage = new Image();
          const blobEvents = [];
          blobImage.onload = () => blobEvents.push("load");
          blobImage.onerror = () => blobEvents.push("error");
          blobImage.src = blobUrl;
          await blobImage.decode().then(() => blobEvents.push("decode"));
          URL.revokeObjectURL(blobUrl);

          const revokedImage = new Image();
          const revokedEvents = [];
          revokedImage.onerror = () => revokedEvents.push("error");
          revokedImage.src = blobUrl;
          let revokedDecode = "";
          try {
            await revokedImage.decode().catch(error => {
              revokedEvents.push("decode");
              throw error;
            });
            revokedDecode = "resolved";
          } catch (error) {
            revokedDecode = error.name;
          }
          await new Promise(resolve => setTimeout(resolve, 0));

          return [
            dataImmediate,
            dataImage.complete,
            dataImage.naturalWidth,
            dataImage.naturalHeight,
            dataEvents.join(","),
            performance.getEntriesByName(dataImage.currentSrc, "resource").length,
            blobImage.complete,
            blobImage.currentSrc === blobUrl,
            blobImage.naturalWidth,
            blobImage.naturalHeight,
            blobEvents.join(","),
            performance.getEntriesByName(blobUrl, "resource").length,
            revokedImage.complete,
            revokedImage.currentSrc === blobUrl,
            revokedImage.naturalWidth,
            revokedImage.naturalHeight,
            revokedDecode,
            revokedEvents.join(",")
          ].join("|");
        })()
    "#;
    let expected = concat!(
        "false,,0,0|true|5|9|decode,load|0|",
        "true|true|8|6|decode,load|0|",
        "true|true|0|0|EncodingError|decode,error"
    );
    let mut direct = EdgeRuntime::new().expect("direct data/blob image runtime");
    assert_eq!(text(&mut direct, source), expected);
    let mut traced = EdgeRuntime::new().expect("traced data/blob image runtime");
    traced.enable_proxy_trace().expect("enable data/blob trace");
    assert_eq!(text(&mut traced, source), expected);
}

#[test]
fn blob_object_urls_use_the_current_https_origin_and_uuid_shape() {
    let mut options = EdgeRuntimeOptions::default();
    options.page = Some(PageInit {
        url: "https://blob-origin.example:8443/path/page.html".to_owned(),
        ..PageInit::default()
    });
    let mut runtime = EdgeRuntime::with_options(options).expect("blob URL runtime");
    let value = text(&mut runtime, "URL.createObjectURL(new Blob(['payload']))");
    let identifier = value
        .strip_prefix("blob:https://blob-origin.example:8443/")
        .expect("Blob URL uses the page origin");
    assert_eq!(identifier.len(), 36, "Blob URL UUID length");
    assert_eq!(identifier.as_bytes()[8], b'-');
    assert_eq!(identifier.as_bytes()[13], b'-');
    assert_eq!(identifier.as_bytes()[18], b'-');
    assert_eq!(identifier.as_bytes()[23], b'-');
    assert_eq!(identifier.as_bytes()[14], b'4', "UUID version");
    assert!(
        matches!(identifier.as_bytes()[19], b'8' | b'9' | b'a' | b'b'),
        "UUID variant"
    );
}

#[test]
fn canvas_paths_rasterize_strokes_curves_transforms_and_rgba_colors() {
    let source = r##"
        (() => {
          const canvas = document.createElement("canvas");
          canvas.width = 20;
          canvas.height = 20;
          const context = canvas.getContext("2d");
          context.strokeStyle = "rgba(255, 0, 0, 0.5)";
          context.lineWidth = 4;
          context.lineCap = "butt";
          context.beginPath();
          context.moveTo(2, 8);
          context.quadraticCurveTo(8, 8, 14, 8);
          context.stroke();
          const straight = Array.from(context.getImageData(8, 8, 1, 1).data);

          context.clearRect(0, 0, 20, 20);
          context.save();
          context.translate(4, 4);
          context.strokeStyle = "rgba(0, 255, 0, 1)";
          context.beginPath();
          context.moveTo(0, 0);
          context.bezierCurveTo(2, 0, 4, 0, 8, 0);
          context.stroke();
          context.restore();
          const transformed = Array.from(context.getImageData(6, 4, 1, 1).data);

          context.clearRect(0, 0, 20, 20);
          context.fillStyle = "#0000ff";
          context.beginPath();
          context.moveTo(2, 2);
          context.lineTo(18, 2);
          context.lineTo(10, 18);
          context.closePath();
          context.fill();
          const filled = Array.from(context.getImageData(10, 8, 1, 1).data);

          context.clearRect(0, 0, 20, 20);
          context.save();
          context.translate(5, 10);
          context.font = "10px Arial";
          context.fillStyle = "rgba(255, 255, 255, 1)";
          context.fillText("G", 0, 0);
          context.restore();
          const transformedText = Array.from(context.getImageData(6, 5, 1, 1).data);
          return [straight, transformed, filled, transformedText]
            .map(value => value.join(","))
            .join("|");
        })()
    "##;
    let mut runtime = EdgeRuntime::new().expect("canvas path raster runtime");
    assert_eq!(
        text(&mut runtime, source),
        "255,0,0,128|0,255,0,255|0,0,255,255|255,255,255,255"
    );
}

#[test]
fn image_bitmap_canvas_sources_crop_resize_draw_and_transfer_match_edge() {
    let source = r##"
        (async () => {
          const canvas = document.createElement("canvas");
          canvas.width = 2;
          canvas.height = 2;
          const context = canvas.getContext("2d");
          context.fillStyle = "#ff0000";
          context.fillRect(0, 0, 1, 2);
          context.fillStyle = "#00ff00";
          context.fillRect(1, 0, 1, 2);

          const bitmap = await createImageBitmap(canvas);
          const copy = document.createElement("canvas");
          copy.width = 2;
          copy.height = 2;
          copy.getContext("2d").drawImage(bitmap, 0, 0);
          const copied = Array.from(
            copy.getContext("2d").getImageData(0, 0, 2, 1).data
          ).join(",");

          const cropped = await createImageBitmap(
            canvas,
            1,
            0,
            1,
            2,
            {resizeWidth: 2, resizeHeight: 1, resizeQuality: "pixelated"}
          );
          const cropCopy = new OffscreenCanvas(2, 1);
          cropCopy.getContext("2d").drawImage(cropped, 0, 0);
          const cropPixels = Array.from(
            cropCopy.getContext("2d").getImageData(0, 0, 2, 1).data
          ).join(",");
          const fiveArgumentCopy = new OffscreenCanvas(4, 1);
          fiveArgumentCopy.getContext("2d").drawImage(canvas, 0, 0, 4, 1);
          const fiveArgumentPixels = Array.from(
            fiveArgumentCopy.getContext("2d").getImageData(0, 0, 4, 1).data
          ).join(",");
          const nineArgumentCopy = new OffscreenCanvas(4, 1);
          nineArgumentCopy
            .getContext("2d")
            .drawImage(canvas, 1, 0, 1, 1, 0, 0, 4, 1);
          const nineArgumentPixels = Array.from(
            nineArgumentCopy.getContext("2d").getImageData(0, 0, 4, 1).data
          ).join(",");
          const alphaCopy = new OffscreenCanvas(1, 1);
          const alphaContext = alphaCopy.getContext("2d");
          alphaContext.globalAlpha = 0.5;
          alphaContext.drawImage(canvas, 0, 0);
          const alphaPixel = Array.from(
            alphaContext.getImageData(0, 0, 1, 1).data
          ).join(",");

          bitmap.close();
          let closedError = "";
          try {
            await createImageBitmap(bitmap);
          } catch (error) {
            closedError = error.name;
          }
          const invalidPromise = createImageBitmap({});
          let invalidError = "";
          try {
            await invalidPromise;
          } catch (error) {
            invalidError = error.name;
          }
          let cropError = "";
          try {
            await createImageBitmap(canvas, 0, 0, 0, 1);
          } catch (error) {
            cropError = error.name;
          }
          let resizeError = "";
          try {
            await createImageBitmap(canvas, {resizeWidth: 0});
          } catch (error) {
            resizeError = error.name;
          }
          const widthResize = await createImageBitmap(
            canvas,
            {resizeWidth: 4}
          );

          const offscreen = new OffscreenCanvas(2, 1);
          const offscreenContext = offscreen.getContext("2d");
          offscreenContext.fillStyle = "#0000ff";
          offscreenContext.fillRect(0, 0, 2, 1);
          const transferred = offscreen.transferToImageBitmap();
          const transferredCopy = document.createElement("canvas");
          transferredCopy.width = 2;
          transferredCopy.height = 1;
          transferredCopy.getContext("2d").drawImage(transferred, 0, 0);
          const transferredPixels = Array.from(
            transferredCopy.getContext("2d").getImageData(0, 0, 1, 1).data
          ).join(",");
          const clearedPixels = Array.from(
            offscreenContext.getImageData(0, 0, 1, 1).data
          ).join(",");

          const image = new Image();
          image.src =
            "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' " +
            "width='7' height='11'/%3E";
          await image.decode();
          const imageBitmap = await createImageBitmap(image);

          const blob = new Blob([
            "<svg xmlns='http://www.w3.org/2000/svg' width='3' height='4'/>"
          ], {type: "image/svg+xml"});
          let blobError = "";
          try {
            await createImageBitmap(blob);
          } catch (error) {
            blobError = error.name;
          }

          return [
            bitmap.width,
            bitmap.height,
            copied,
            cropped.width,
            cropped.height,
            cropPixels,
            fiveArgumentPixels,
            nineArgumentPixels,
            alphaPixel,
            Object.prototype.toString.call(invalidPromise),
            closedError,
            invalidError,
            cropError,
            resizeError,
            widthResize.width,
            widthResize.height,
            transferred.width,
            transferred.height,
            transferredPixels,
            clearedPixels,
            imageBitmap.width,
            imageBitmap.height,
            blobError
          ].join("|");
        })()
    "##;
    let expected = concat!(
        "0|0|255,0,0,255,0,255,0,255|2|1|0,255,0,255,0,255,0,255|",
        "255,0,0,255,255,0,0,255,0,255,0,255,0,255,0,255|",
        "0,255,0,255,0,255,0,255,0,255,0,255,0,255,0,255|255,0,0,128|",
        "[object Promise]|InvalidStateError|TypeError|RangeError|InvalidStateError|",
        "4|4|2|1|0,0,255,255|0,0,0,0|7|11|InvalidStateError"
    );
    let mut direct = EdgeRuntime::new().expect("direct ImageBitmap runtime");
    assert_eq!(text(&mut direct, source), expected);
    let mut traced = EdgeRuntime::new().expect("traced ImageBitmap runtime");
    traced
        .enable_proxy_trace()
        .expect("enable ImageBitmap trace");
    assert_eq!(text(&mut traced, source), expected);
    let trace = traced.proxy_trace();
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api == "window.createImageBitmap" })
    );
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "call" && entry.api.ends_with(".drawImage"))
    );
}

#[test]
fn deterministic_mode_links_clock_timers_events_and_randomness() {
    let mut options = EdgeRuntimeOptions::default();
    options.deterministic.clock_epoch_ms = Some(1_700_000_000_000);
    options.deterministic.clock_step_ms = 2;
    options.deterministic.random_seed = Some(0x1234_5678);
    let mut first = EdgeRuntime::with_options(options.clone()).expect("deterministic runtime");
    let initial = text(
        &mut first,
        r#"
        [
          Date.now(),
          new Date().getTime(),
          performance.timeOrigin,
          performance.now(),
          new Event("edge").timeStamp,
          Function.prototype.toString.call(Date),
          Function.prototype.toString.call(Date.now),
          new Date(0).getTime()
        ].join("|")
        "#,
    );
    assert_eq!(
        initial,
        "1700000000000|1700000000000|1700000000000|0|0|function Date() { [native code] }|function now() { [native code] }|0"
    );
    assert_eq!(
        text(
            &mut first,
            r#"
            const dateDescriptor = Object.getOwnPropertyDescriptor(Date, "now");
            [
              Date(0) === Date(),
              dateDescriptor.writable,
              dateDescriptor.enumerable,
              dateDescriptor.configurable,
              Object.getOwnPropertyDescriptor(globalThis, "Date").enumerable
            ].join("|")
            "#,
        ),
        "true|true|false|true|false"
    );
    let _ = first
        .evaluate(
            r#"
            globalThis.deterministicTimerAnswer = "pending";
            setTimeout(() => {
              const bytes = new Uint8Array(8);
              crypto.getRandomValues(bytes);
              deterministicTimerAnswer = [
                Date.now(),
                performance.now(),
                Math.random(),
                Array.from(bytes).join(",")
              ].join("|");
            }, 10);
            "#,
        )
        .expect("deterministic timer");
    let first_answer = text(&mut first, "deterministicTimerAnswer");
    assert!(first_answer.starts_with("1700000000010|10|"));

    let mut second = EdgeRuntime::with_options(options).expect("second deterministic runtime");
    let _ = second
        .evaluate(
            r#"
            globalThis.deterministicTimerAnswer = "pending";
            setTimeout(() => {
              const bytes = new Uint8Array(8);
              crypto.getRandomValues(bytes);
              deterministicTimerAnswer = [
                Date.now(),
                performance.now(),
                Math.random(),
                Array.from(bytes).join(",")
              ].join("|");
            }, 10);
            "#,
        )
        .expect("second deterministic timer");
    assert_eq!(first_answer, text(&mut second, "deterministicTimerAnswer"));
}

#[test]
fn edge_clock_semantics_link_performance_date_events_timers_and_animation_frames() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let started = std::time::Instant::now();
    let answer = text(
        &mut runtime,
        r#"
        (() => {
          const descriptor = Object.getOwnPropertyDescriptor(
            Performance.prototype,
            "now"
          );
          const start = performance.now();
          const wallStart = Date.now();
          const eventStamp = new Event("clock").timeStamp;
          let precisionDelta = 0;
          for (let index = 0; index < 1000000 && precisionDelta === 0; index++) {
            precisionDelta = performance.now() - start;
          }
          globalThis.edgeClockAnswer = "pending";
          globalThis.edgeRafAnswer = "pending";
          globalThis.edgeStringTimerAnswer = "pending";
          setTimeout("edgeStringTimerAnswer = 'executed'", 0);
          requestAnimationFrame(timestamp => {
            edgeRafAnswer = [
              timestamp - start,
              performance.now() - timestamp,
              document.timeline.currentTime === timestamp
            ].join(",");
          });
          setTimeout(() => {
            const end = performance.now();
            edgeClockAnswer = [
              end - start,
              Date.now() - wallStart,
              Math.abs(performance.timeOrigin + end - Date.now()),
              Math.abs(eventStamp - start),
              precisionDelta,
              Date.now() % 1,
              Function.prototype.toString.call(performance.now),
              performance.now.name,
              performance.now.length,
              descriptor.enumerable,
              descriptor.configurable,
              descriptor.writable
            ].join(",");
          }, 25);
        })()
        "#,
    );
    assert_eq!(answer, "undefined");
    assert!(started.elapsed() >= std::time::Duration::from_millis(20));
    assert_eq!(text(&mut runtime, "edgeStringTimerAnswer"), "executed");

    let values = text(&mut runtime, "edgeClockAnswer")
        .split(',')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 12);
    let timer_delta = values[0].parse::<f64>().expect("timer delta");
    let date_delta = values[1].parse::<f64>().expect("Date delta");
    let epoch_skew = values[2].parse::<f64>().expect("epoch skew");
    let event_skew = values[3].parse::<f64>().expect("event skew");
    let precision_delta = values[4].parse::<f64>().expect("precision delta");
    assert!(timer_delta >= 24.0, "timer delta was {timer_delta}");
    assert!(date_delta >= 20.0, "Date delta was {date_delta}");
    assert!(epoch_skew <= 5.0, "epoch skew was {epoch_skew}");
    assert!(event_skew <= 2.0, "Event skew was {event_skew}");
    assert!(
        (0.099..=5.0).contains(&precision_delta),
        "precision delta was {precision_delta}"
    );
    assert_eq!(
        &values[5..],
        &[
            "0",
            "function now() { [native code] }",
            "now",
            "0",
            "true",
            "true",
            "true"
        ]
    );

    let raf_text = text(&mut runtime, "edgeRafAnswer");
    let raf = raf_text.split(',').collect::<Vec<_>>();
    assert_eq!(raf.len(), 3);
    let raf_delta = raf[0].parse::<f64>().expect("RAF timestamp delta");
    let raf_skew = raf[1].parse::<f64>().expect("RAF/performance skew");
    assert!(raf_delta >= 15.0, "RAF timestamp delta was {raf_delta}");
    assert!(
        raf_skew >= -0.000001,
        "RAF timestamp was unexpectedly in the future by {}ms",
        -raf_skew
    );
    assert_eq!(
        raf[2], "true",
        "document timeline did not use the RAF timestamp"
    );
}

#[test]
fn performance_now_uses_a_realm_relative_monotonic_100_microsecond_grid() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let values = text(
        &mut runtime,
        r#"
        (() => {
          let previous = performance.now();
          let nonMonotonic = false;
          let offGrid = false;
          for (let index = 0; index < 100000; index++) {
            const current = performance.now();
            nonMonotonic ||= current < previous;
            // Edge exposes the 100us grid after subtracting an absolute
            // platform monotonic origin, so normal IEEE-754 cancellation can
            // leave a sub-microsecond residue around the logical grid.
            offGrid ||= Math.abs(current * 10 - Math.round(current * 10)) > 1e-4;
            previous = current;
          }
          const parentNowAtChildCreation = performance.now();
          const frame = document.createElement("iframe");
          document.body.appendChild(frame);
          const childNow = frame.contentWindow.performance.now();
          const childOriginDelta =
            frame.contentWindow.performance.timeOrigin - performance.timeOrigin;
          const parentNowAfterChildCreation = performance.now();
          return [
            nonMonotonic,
            offGrid,
            childNow,
            childOriginDelta,
            parentNowAtChildCreation,
            parentNowAfterChildCreation,
            Math.abs(performance.timeOrigin + performance.now() - Date.now()),
            Math.abs(
              frame.contentWindow.performance.timeOrigin +
              frame.contentWindow.performance.now() -
              frame.contentWindow.Date.now()
            )
          ].join("|");
        })()
        "#,
    )
    .split('|')
    .map(str::to_owned)
    .collect::<Vec<_>>();
    assert_eq!(values.len(), 8);
    assert_eq!(values[0], "false");
    assert_eq!(values[1], "false");
    let child_now = values[2].parse::<f64>().expect("child performance.now");
    let child_origin_delta = values[3].parse::<f64>().expect("child time origin delta");
    let parent_now_before = values[4]
        .parse::<f64>()
        .expect("parent performance.now before");
    let parent_now_after = values[5]
        .parse::<f64>()
        .expect("parent performance.now after");
    let root_epoch_skew = values[6].parse::<f64>().expect("root epoch skew");
    let child_epoch_skew = values[7].parse::<f64>().expect("child epoch skew");
    assert!(child_now >= 0.0);
    assert!(
        child_origin_delta >= parent_now_before - 5.0,
        "child origin delta was {child_origin_delta}; parent before was {parent_now_before}"
    );
    assert!(
        child_origin_delta <= parent_now_after + 5.0,
        "child origin delta was {child_origin_delta}; parent after was {parent_now_after}"
    );
    assert!(
        (child_origin_delta + child_now - parent_now_after).abs() <= 5.0,
        "child timeline did not match parent: origin={child_origin_delta}, child now={child_now}, parent now={parent_now_after}"
    );
    assert!(
        root_epoch_skew <= 5.0,
        "root epoch skew was {root_epoch_skew}"
    );
    assert!(
        child_epoch_skew <= 5.0,
        "child epoch skew was {child_epoch_skew}"
    );
}

#[test]
fn iframe_timers_keep_the_child_window_receiver_and_performance_timeline() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        const clockFrame = document.createElement("iframe");
        clockFrame.srcdoc = "<p>clock</p>";
        document.body.appendChild(clockFrame);
        const child = clockFrame.contentWindow;
        const childStart = child.performance.now();
        globalThis.iframeClockAnswer = "pending";
        child.setTimeout(function () {
          const now = child.performance.now();
          iframeClockAnswer = [
            this === child,
            now - childStart,
            Math.abs(new child.Event("clock").timeStamp - now),
            Math.abs(child.performance.timeOrigin + now - child.Date.now())
          ].join("|");
        }, 10);
        "#,
    );
    let values = text(&mut runtime, "iframeClockAnswer")
        .split('|')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 4);
    assert_eq!(values[0], "true");
    let timer_delta = values[1].parse::<f64>().expect("iframe timer delta");
    let event_skew = values[2].parse::<f64>().expect("iframe Event skew");
    let epoch_skew = values[3].parse::<f64>().expect("iframe epoch skew");
    assert!(timer_delta >= 9.0, "iframe timer delta was {timer_delta}");
    assert!(event_skew <= 2.0, "iframe Event skew was {event_skew}");
    assert!(epoch_skew <= 5.0, "iframe epoch skew was {epoch_skew}");
}

#[test]
fn speech_synthesis_voices_are_profiled_realm_local_and_trace_stable() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.speech.voices = vec![
        SpeechVoiceFingerprint {
            voice_uri: "Microsoft Edge Voice zh-CN XiaoxiaoNeural".to_owned(),
            name: "Microsoft Xiaoxiao Online (Natural) - Chinese (Mainland)".to_owned(),
            lang: "zh-CN".to_owned(),
            local_service: false,
            is_default: true,
        },
        SpeechVoiceFingerprint {
            voice_uri: "Microsoft David - English (United States)".to_owned(),
            name: "Microsoft David - English (United States)".to_owned(),
            lang: "en-US".to_owned(),
            local_service: true,
            is_default: false,
        },
    ];
    let source = r#"
        (() => {
          const first = speechSynthesis.getVoices();
          const second = speechSynthesis.getVoices();
          const descriptor = Object.getOwnPropertyDescriptor(
            SpeechSynthesisVoice.prototype,
            "name"
          );
          const utterance = new SpeechSynthesisUtterance("edge");
          utterance.voice = first[0];
          const frame = document.createElement("iframe");
          document.body.appendChild(frame);
          const frameVoice = frame.contentWindow.speechSynthesis.getVoices()[0];
          return [
            first.length,
            first !== second,
            first[0] === second[0],
            first[0] instanceof SpeechSynthesisVoice,
            Object.getPrototypeOf(first[0]) === SpeechSynthesisVoice.prototype,
            Object.prototype.toString.call(first[0]),
            first[0].voiceURI,
            first[0].name,
            first[0].lang,
            first[0].localService,
            first[0].default,
            utterance.voice === first[0],
            !("value" in descriptor),
            descriptor.enumerable,
            descriptor.configurable,
            descriptor.set === undefined,
            Function.prototype.toString.call(descriptor.get),
            Function.prototype.toString.call(
              SpeechSynthesis.prototype.getVoices
            ),
            frameVoice instanceof frame.contentWindow.SpeechSynthesisVoice,
            frameVoice instanceof SpeechSynthesisVoice,
            frameVoice !== first[0],
            frameVoice.name,
            Object.getOwnPropertyNames(
              SpeechSynthesisVoice.prototype
            ).join(",")
          ].join("|");
        })()
    "#;
    let expected = concat!(
        "2|true|true|true|true|[object SpeechSynthesisVoice]|",
        "Microsoft Edge Voice zh-CN XiaoxiaoNeural|",
        "Microsoft Xiaoxiao Online (Natural) - Chinese (Mainland)|",
        "zh-CN|false|true|true|true|true|true|true|",
        "function get name() { [native code] }|",
        "function getVoices() { [native code] }|",
        "true|false|true|",
        "Microsoft Xiaoxiao Online (Natural) - Chinese (Mainland)|",
        "voiceURI,name,lang,localService,default,constructor"
    );

    let mut direct = EdgeRuntime::with_options(options.clone()).expect("direct Edge runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced = EdgeRuntime::with_options(options).expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, source), expected);
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call" && entry.api.ends_with(".speechSynthesis.getVoices")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get" && entry.api == "SpeechSynthesisVoice.prototype.name"
    }));
}

#[test]
fn rendering_fingerprint_links_canvas_screen_storage_and_audio() {
    let mut baseline_runtime = EdgeRuntime::new().expect("baseline runtime");
    let baseline_canvas = text(
        &mut baseline_runtime,
        "document.createElement('canvas').toDataURL()",
    );
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.width = 1920;
    options.fingerprint.screen.height = 1080;
    options.fingerprint.screen.avail_width = 1900;
    options.fingerprint.screen.avail_height = 1040;
    options.fingerprint.screen.viewport_width = 1536.0;
    options.fingerprint.screen.viewport_height = 864.0;
    options.fingerprint.screen.outer_width = 1600.0;
    options.fingerprint.screen.outer_height = 900.0;
    options.fingerprint.screen.device_pixel_ratio = 1.25;
    options.fingerprint.rendering.canvas.text_width_scale = 1.25;
    options.fingerprint.rendering.canvas.data_url_salt = "profile-A".to_owned();
    options.fingerprint.rendering.audio.sample_rate = 48_000.0;
    options.fingerprint.rendering.audio.channel_noise_amplitude = 0.000_01;
    options.fingerprint.storage.quota_bytes = 2_000_000;
    options.fingerprint.storage.usage_bytes = 125_000;
    options.fingerprint.storage.persisted = true;
    let mut runtime = EdgeRuntime::with_options(options).expect("fingerprinted runtime");
    let answer = text(
        &mut runtime,
        r#"
        const canvas = document.createElement("canvas");
        const context = canvas.getContext("2d");
        const dataURL = canvas.toDataURL();
        globalThis.fingerprintAsyncAnswer = "pending";
        Promise.all([navigator.storage.estimate(), navigator.storage.persisted()]).then(values => {
          fingerprintAsyncAnswer = [
            values[0].quota,
            values[0].usage,
            values[1]
          ].join("|");
        });
        [
          screen.width,
          screen.height,
          screen.availWidth,
          screen.availHeight,
          innerWidth,
          innerHeight,
          outerWidth,
          outerHeight,
          devicePixelRatio,
          context.measureText("abcd").width,
          dataURL.startsWith("data:image/png;base64,"),
          new AudioContext().sampleRate
        ].join("|")
        "#,
    );
    assert_eq!(
        answer,
        "1920|1080|1900|1040|1536|864|1600|900|1.25|28.555221557617188|true|48000"
    );
    let fingerprinted_canvas = text(&mut runtime, "dataURL");
    assert_ne!(fingerprinted_canvas, baseline_canvas);
    assert!(fingerprinted_canvas.len() > baseline_canvas.len());
    assert_eq!(
        text(&mut runtime, "fingerprintAsyncAnswer"),
        "2000000|125000|true"
    );
}

#[test]
fn paint_worklet_executes_in_its_own_realm() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let _ = runtime
        .evaluate(
            r#"
            globalThis.paintWorkletAnswer = "pending";
            CSS.paintWorklet.addModule(
              "data:text/javascript," +
              encodeURIComponent(`
                class Checkerboard {
                  static get inputProperties() { return ["--edge-color"]; }
                  static get inputArguments() { return ["<length>"]; }
                  static get contextOptions() { return {alpha: false}; }
                  paint(context, size, properties, argumentsList) {}
                }
                registerPaint("edge-checkerboard", Checkerboard);
              `)
            ).then(() => paintWorkletAnswer = "loaded");
            "#,
        )
        .expect("PaintWorklet module");
    assert_eq!(text(&mut runtime, "paintWorkletAnswer"), "loaded");
}

#[test]
fn proxy_trace_covers_existing_iframe_and_worklet_realms_without_shape_drift() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let before = text(
        &mut runtime,
        r#"
        globalThis.traceFrame = document.createElement("iframe");
        traceFrame.srcdoc = "<p>ready</p>";
        document.body.appendChild(traceFrame);
        [
          Function.prototype.toString.call(Worklet.prototype.addModule),
          Function.prototype.toString.call(
            Object.getOwnPropertyDescriptor(
              HTMLIFrameElement.prototype,
              "contentWindow"
            ).get
          ),
          Object.getOwnPropertyNames(Worklet.prototype).join(","),
          Object.getPrototypeOf(traceFrame.contentWindow) === Window.prototype
        ].join("|")
        "#,
    );
    runtime.enable_proxy_trace().expect("enable Proxy trace");
    let after = text(
        &mut runtime,
        r#"
        [
          Function.prototype.toString.call(Worklet.prototype.addModule),
          Function.prototype.toString.call(
            Object.getOwnPropertyDescriptor(
              HTMLIFrameElement.prototype,
              "contentWindow"
            ).get
          ),
          Object.getOwnPropertyNames(Worklet.prototype).join(","),
          Object.getPrototypeOf(traceFrame.contentWindow) === Window.prototype
        ].join("|")
        "#,
    );
    assert_eq!(after, before);
    let _ = runtime
        .evaluate(
            r#"
            traceFrame.srcdoc =
              "<script>window.iframeTraceValue = document.createElement('section').nodeName<\/script>";
            CSS.paintWorklet.addModule(
              "data:text/javascript," +
              encodeURIComponent(`
                class TracedPaint {
                  paint(context, size, properties, argumentsList) {}
                }
                registerPaint("traced-paint", TracedPaint);
              `)
            );
            "#,
        )
        .expect("trace iframe and Worklet");
    assert_eq!(
        text(&mut runtime, "traceFrame.contentWindow.iframeTraceValue"),
        "SECTION"
    );
    let trace = runtime.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.api.starts_with("iframe[") && entry.api.contains("document.createElement")
    }));
    assert!(trace.iter().any(|entry| {
        entry.api.starts_with("paintWorklet[") && entry.api.contains("registerPaint")
    }));
    runtime.disable_proxy_trace();
    let count = runtime.proxy_trace().len();
    let _ = runtime
        .evaluate(
            r#"
            traceFrame.srcdoc =
              "<script>window.traceDisabledShape = Function.prototype.toString.call(document.createElement)<\/script>";
            "#,
        )
        .expect("disabled Proxy trace");
    assert_eq!(runtime.proxy_trace().len(), count);
}

#[test]
fn dom_animation_and_visibility_state_survives_proxy_trace() {
    const SCRIPT: &str = r#"
        (() => {
          const host = document.createElement("section");
          const element = document.createElement("div");
          host.appendChild(element);
          document.body.appendChild(host);
          const animation = element.animate(
            [{ opacity: 0 }, { opacity: 1 }],
            { duration: 25 }
          );
          const initial = [
            element.getAnimations().length,
            element.getAnimations()[0] === animation,
            document.getAnimations().includes(animation)
          ].join(",");
          element.style.setProperty("visibility", "hidden");
          const visibility = [
            element.checkVisibility(),
            element.checkVisibility({ visibilityProperty: true })
          ].join(",");
          animation.cancel();
          return [
            initial,
            visibility,
            element.getAnimations().length,
            document.getAnimations().includes(animation),
            Function.prototype.toString.call(
              Element.prototype.getAnimations
            ),
            Function.prototype.toString.call(
              Element.prototype.checkVisibility
            )
          ].join("|");
        })()
    "#;
    let expected = concat!(
        "1,true,true|true,false|0|false|",
        "function getAnimations() { [native code] }|",
        "function checkVisibility() { [native code] }"
    );

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    assert_eq!(text(&mut direct, SCRIPT), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, SCRIPT), expected);
    let trace = traced.proxy_trace();
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".animate") })
    );
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".getAnimations") })
    );
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".checkVisibility") })
    );
}

#[test]
fn navigation_and_all_replayed_resource_initiators_populate_the_edge_timeline() {
    let page_html = "<!doctype html><main id=\"root\"></main>";
    let options = EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://sandbox.test/app/index.html".to_owned(),
            html: page_html.to_owned(),
            referrer: String::new(),
            content_type: "text/html; charset=utf-8".to_owned(),
        }),
        network_replay: vec![
            NetworkReplayEntry {
                url: "https://sandbox.test/assets/fetch.txt".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![(
                    "Content-Type".to_owned(),
                    "text/plain; charset=utf-8".to_owned(),
                )],
                body: b"fetch-body".to_vec(),
            },
            NetworkReplayEntry::get("https://sandbox.test/assets/xhr.txt", b"xhr-body".to_vec()),
            NetworkReplayEntry::get(
                "https://sandbox.test/assets/runtime.js",
                b"globalThis.externalScriptLoaded = 41;".to_vec(),
            ),
            NetworkReplayEntry::get(
                "https://sandbox.test/assets/runtime.css",
                b"main { color: rgb(1, 2, 3); }".to_vec(),
            ),
            NetworkReplayEntry {
                url: "https://sandbox.test/assets/frame.html".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
                body: b"<!doctype html><p id='frame'>frame</p>".to_vec(),
            },
            NetworkReplayEntry::get(
                "https://sandbox.test/assets/worker.js",
                b"postMessage('ready');".to_vec(),
            ),
            NetworkReplayEntry::get("https://sandbox.test/assets/buffer-one", b"one".to_vec()),
            NetworkReplayEntry::get("https://sandbox.test/assets/buffer-two", b"two".to_vec()),
        ],
        ..EdgeRuntimeOptions::default()
    };
    let source = r#"
        (async () => {
          const navigation = performance.getEntriesByType("navigation")[0];
          const initial = [
            navigation instanceof PerformanceNavigationTiming,
            navigation instanceof PerformanceResourceTiming,
            navigation.name,
            navigation.entryType,
            navigation.initiatorType,
            navigation.type,
            navigation.responseStatus,
            navigation.contentType,
            navigation.nextHopProtocol,
            navigation.responseEnd >= navigation.startTime,
            performance.getEntriesByType("paint").length
          ].join(",");

          await fetch("../assets/fetch.txt").then(response => response.text());
          await new Promise(resolve => {
            const xhr = new XMLHttpRequest();
            xhr.open("GET", "../assets/xhr.txt");
            xhr.onloadend = resolve;
            xhr.send();
          });
          await new Promise(resolve => {
            const script = document.createElement("script");
            script.onload = resolve;
            script.src = "../assets/runtime.js";
            document.body.append(script);
          });
          await new Promise(resolve => {
            const link = document.createElement("link");
            link.onload = resolve;
            link.rel = "stylesheet";
            link.href = "../assets/runtime.css";
            document.head.append(link);
          });
          const frameNavigation = await new Promise(resolve => {
            const frame = document.createElement("iframe");
            frame.onload = () => resolve([
              frame.contentWindow.performance.getEntriesByType("navigation")[0].name,
              frame.contentWindow.performance.getEntriesByType("navigation")[0]
                instanceof frame.contentWindow.PerformanceNavigationTiming
            ].join(","));
            frame.src = "../assets/frame.html";
            document.body.append(frame);
          });
          await new Promise(resolve => {
            const worker = new Worker("../assets/worker.js");
            worker.onmessage = () => {
              worker.terminate();
              resolve();
            };
          });
          await new Promise(resolve => setTimeout(resolve, 0));

          const resources = performance.getEntriesByType("resource")
            .map(entry => [
              new URL(entry.name).pathname,
              entry.initiatorType,
              entry.responseStatus,
              entry.nextHopProtocol,
              entry.responseEnd >= entry.startTime
            ].join(","))
            .sort()
            .join(";");

          performance.clearResourceTimings();
          performance.setResourceTimingBufferSize(1);
          let handlerCount = 0;
          let listenerCount = 0;
          const observed = [];
          performance.onresourcetimingbufferfull = () => handlerCount++;
          performance.addEventListener(
            "resourcetimingbufferfull",
            () => listenerCount++
          );
          const observer = new PerformanceObserver(list => {
            observed.push(...list.getEntries().map(entry => entry.name));
          });
          observer.observe({ type: "resource" });
          await fetch("../assets/buffer-one");
          await fetch("../assets/buffer-two");
          await new Promise(resolve => setTimeout(resolve, 0));
          const buffer = [
            performance.getEntriesByType("resource").length,
            handlerCount,
            listenerCount,
            observed.some(name => name.endsWith("/buffer-one")),
            observed.some(name => name.endsWith("/buffer-two"))
          ].join(",");
          return [initial, externalScriptLoaded, frameNavigation, resources, buffer].join("|");
        })()
    "#;
    let expected = concat!(
        "true,true,https://sandbox.test/app/index.html,navigation,navigation,navigate,",
        "200,text/html,h2,true,0|41|https://sandbox.test/assets/frame.html,true|",
        "/assets/fetch.txt,fetch,200,h2,true;",
        "/assets/frame.html,iframe,200,h2,true;",
        "/assets/runtime.css,link,200,h2,true;",
        "/assets/runtime.js,script,200,h2,true;",
        "/assets/worker.js,other,200,h2,true;",
        "/assets/xhr.txt,xmlhttprequest,200,h2,true|",
        "1,1,1,true,true"
    );

    let mut direct =
        EdgeRuntime::with_options(options.clone()).expect("direct performance resource runtime");
    assert_eq!(text(&mut direct, source), expected);

    let mut traced =
        EdgeRuntime::with_options(options).expect("traced performance resource runtime");
    traced
        .enable_proxy_trace()
        .expect("enable performance trace");
    assert_eq!(text(&mut traced, source), expected);
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call" && entry.api.ends_with(".performance.getEntriesByType")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get" && entry.api == "PerformanceResourceTiming.prototype.initiatorType"
    }));
}

#[test]
fn page_load_populates_navigation_visibility_resource_and_paint_entries_in_edge_order() {
    let script_url = "https://timeline.example.test/assets/app.js";
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://timeline.example.test/page".to_owned(),
            html: concat!(
                "<!doctype html><html><head></head><body>",
                "<main>visible content</main>",
                "<script src=\"/assets/app.js\"></script>",
                "</body></html>"
            )
            .to_owned(),
            content_type: "text/html; charset=utf-8".to_owned(),
            ..PageInit::default()
        }),
        network_replay: vec![NetworkReplayEntry {
            url: script_url.to_owned(),
            method: "GET".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: vec![(
                "Content-Type".to_owned(),
                "text/javascript; charset=utf-8".to_owned(),
            )],
            body: b"globalThis.timelineScriptLoaded = true;".to_vec(),
        }],
        ..EdgeRuntimeOptions::default()
    })
    .expect("performance timeline runtime");

    let result = text(
        &mut runtime,
        r#"
        (() => {
          const entries = performance.getEntries();
          const navigation = entries[0];
          const visibility = entries[1];
          const resource = entries[2];
          const paints = entries.slice(3);
          const navigationJSON = navigation.toJSON();
          return [
            timelineScriptLoaded,
            entries.map(entry => entry.entryType).join(","),
            navigation instanceof PerformanceNavigationTiming,
            navigation instanceof PerformanceResourceTiming,
            navigation.duration === navigation.loadEventEnd,
            navigation.responseEnd <= navigation.domInteractive,
            navigation.domInteractive <= navigation.domContentLoadedEventStart,
            navigation.domContentLoadedEventStart <= navigation.domContentLoadedEventEnd,
            navigation.domContentLoadedEventEnd <= navigation.domComplete,
            navigation.domComplete <= navigation.loadEventStart,
            navigation.loadEventStart <= navigation.loadEventEnd,
            navigation.confidence === null,
            navigation.notRestoredReasons === null,
            Object.keys(navigationJSON).join(","),
            visibility instanceof VisibilityStateEntry,
            visibility.name,
            visibility.startTime,
            visibility.duration,
            resource instanceof PerformanceResourceTiming,
            resource.name,
            resource.initiatorType,
            resource.responseStatus,
            resource.encodedBodySize,
            paints.map(entry => entry.name).join(","),
            paints.every(entry =>
              entry instanceof PerformancePaintTiming &&
              entry.duration === 0 &&
              entry.paintTime <= entry.presentationTime &&
              entry.startTime === entry.presentationTime
            )
          ].join("|");
        })()
        "#,
    );
    let expected_navigation_keys = concat!(
        "name,entryType,startTime,duration,initiatorType,deliveryType,",
        "nextHopProtocol,renderBlockingStatus,contentType,contentEncoding,",
        "workerStart,workerRouterEvaluationStart,workerCacheLookupStart,",
        "workerMatchedSourceType,workerFinalSourceType,redirectStart,redirectEnd,",
        "fetchStart,domainLookupStart,domainLookupEnd,connectStart,",
        "secureConnectionStart,connectEnd,requestStart,responseStart,",
        "firstInterimResponseStart,finalResponseHeadersStart,responseEnd,",
        "transferSize,encodedBodySize,decodedBodySize,responseStatus,serverTiming,",
        "unloadEventStart,unloadEventEnd,domInteractive,",
        "domContentLoadedEventStart,domContentLoadedEventEnd,domComplete,",
        "loadEventStart,loadEventEnd,type,redirectCount,activationStart,",
        "criticalCHRestart,notRestoredReasons,confidence"
    );
    assert_eq!(
        result,
        format!(
            concat!(
                "true|navigation,visibility-state,resource,paint,paint|",
                "true|true|true|true|true|true|true|true|true|",
                "true|true|{}|true|visible|0|0|true|{}|script|200|",
                "39|first-paint,first-contentful-paint|true"
            ),
            expected_navigation_keys, script_url
        )
    );
}

#[test]
fn webcodecs_support_state_roundtrip_worker_and_trace_match_edge_shapes() {
    const SOURCE: &str = r#"
        (async () => {
          const constructors = [
            AudioDecoder,
            AudioEncoder,
            VideoDecoder,
            VideoEncoder
          ];
          const staticShape = constructors.map(Constructor => {
            const descriptor = Object.getOwnPropertyDescriptor(
              Constructor,
              "isConfigSupported"
            );
            return [
              Reflect.ownKeys(Constructor).map(String).join(","),
              descriptor.enumerable,
              descriptor.configurable,
              descriptor.writable,
              descriptor.value.name,
              descriptor.value.length,
              Function.prototype.toString.call(descriptor.value)
            ].join(",");
          }).join(";");

          const support = await Promise.all([
            AudioDecoder.isConfigSupported({
              codec: "opus",
              numberOfChannels: 2,
              sampleRate: 48000
            }),
            AudioEncoder.isConfigSupported({
              codec: "opus",
              numberOfChannels: 2,
              sampleRate: 48000,
              bitrate: 128000
            }),
            VideoDecoder.isConfigSupported({
              codec: "vp8",
              codedHeight: 240,
              codedWidth: 320
            }),
            VideoEncoder.isConfigSupported({
              codec: "vp8",
              height: 240,
              width: 320,
              bitrate: 500000,
              framerate: 30
            })
          ]);
          const supportShape = support.map(value => [
            value.supported,
            value.config.codec
          ].join(",")).join(";");

          let missingCodec;
          try {
            await AudioDecoder.isConfigSupported({});
          } catch (error) {
            missingCodec = error.name;
          }

          const stateDecoder = new VideoDecoder({
            output() {},
            error() {}
          });
          let flushError;
          try {
            await stateDecoder.flush();
          } catch (error) {
            flushError = error.name;
          }
          let decodeError;
          try {
            stateDecoder.decode(new EncodedVideoChunk({
              type: "key",
              timestamp: 0,
              data: new Uint8Array()
            }));
          } catch (error) {
            decodeError = error.name;
          }
          stateDecoder.close();
          let configureClosedError;
          try {
            stateDecoder.configure({codec: "vp8"});
          } catch (error) {
            configureClosedError = error.name;
          }

          let encodedAudio;
          let audioMetadata;
          let audioHandlerCount = 0;
          let audioListenerCount = 0;
          const audioEncoder = new AudioEncoder({
            output(chunk, metadata) {
              encodedAudio = chunk;
              audioMetadata = metadata;
            },
            error(error) {
              throw error;
            }
          });
          audioEncoder.ondequeue = () => audioHandlerCount++;
          audioEncoder.addEventListener(
            "dequeue",
            () => audioListenerCount++
          );
          audioEncoder.configure({
            codec: "opus",
            numberOfChannels: 1,
            sampleRate: 48000,
            bitrate: 128000
          });
          const audioInput = new AudioData({
            format: "f32",
            sampleRate: 48000,
            numberOfFrames: 2,
            numberOfChannels: 1,
            timestamp: 5,
            data: new Float32Array([0.25, -0.25])
          });
          audioEncoder.encode(audioInput);
          await audioEncoder.flush();
          let decodedAudio;
          const audioDecoder = new AudioDecoder({
            output(value) {
              decodedAudio = value;
            },
            error(error) {
              throw error;
            }
          });
          audioDecoder.configure(audioMetadata.decoderConfig);
          audioDecoder.decode(encodedAudio);
          await audioDecoder.flush();
          const audioShape = [
            encodedAudio instanceof EncodedAudioChunk,
            Object.prototype.toString.call(encodedAudio),
            encodedAudio.type,
            encodedAudio.timestamp,
            encodedAudio.byteLength,
            audioMetadata.decoderConfig.codec,
            decodedAudio instanceof AudioData,
            decodedAudio.format,
            decodedAudio.numberOfFrames,
            decodedAudio.numberOfChannels,
            decodedAudio.timestamp,
            audioHandlerCount,
            audioListenerCount
          ].join(",");

          let encodedVideo;
          let videoMetadata;
          let decodedVideo;
          const videoEncoder = new VideoEncoder({
            output(chunk, metadata) {
              encodedVideo = chunk;
              videoMetadata = metadata;
            },
            error(error) {
              throw error;
            }
          });
          videoEncoder.configure({
            codec: "vp8",
            width: 2,
            height: 2,
            bitrate: 500000,
            framerate: 30
          });
          const videoInput = new VideoFrame(new Uint8Array(16), {
            format: "RGBA",
            codedWidth: 2,
            codedHeight: 2,
            timestamp: 7,
            duration: 33333
          });
          videoEncoder.encode(videoInput, {keyFrame: true});
          await videoEncoder.flush();
          const videoDecoder = new VideoDecoder({
            output(value) {
              decodedVideo = value;
            },
            error(error) {
              throw error;
            }
          });
          videoDecoder.configure(videoMetadata.decoderConfig);
          videoDecoder.decode(encodedVideo);
          await videoDecoder.flush();
          const videoShape = [
            encodedVideo instanceof EncodedVideoChunk,
            Object.prototype.toString.call(encodedVideo),
            encodedVideo.type,
            encodedVideo.timestamp,
            encodedVideo.duration,
            encodedVideo.byteLength,
            videoMetadata.decoderConfig.codec,
            decodedVideo instanceof VideoFrame,
            decodedVideo.format,
            decodedVideo.codedWidth,
            decodedVideo.codedHeight,
            decodedVideo.timestamp
          ].join(",");

          const workerShape = await new Promise((resolve, reject) => {
            const worker = new Worker(URL.createObjectURL(new Blob([`
              VideoEncoder.isConfigSupported({
                codec: "vp8",
                width: 2,
                height: 2
              }).then(result => postMessage([
                result.supported,
                result.config.codec,
                Function.prototype.toString.call(
                  VideoEncoder.isConfigSupported
                )
              ].join(",")), error => postMessage(error.name));
            `], {type: "text/javascript"})));
            worker.onmessage = event => {
              worker.terminate();
              resolve(event.data);
            };
            worker.onerror = reject;
          });

          return [
            staticShape,
            supportShape,
            missingCodec,
            [flushError, decodeError, configureClosedError].join(","),
            audioShape,
            videoShape,
            workerShape
          ].join("|");
        })()
    "#;

    let mut direct = EdgeRuntime::new().expect("direct WebCodecs runtime");
    let expected = text(&mut direct, SOURCE);
    assert!(expected.contains(
        "length,name,prototype,isConfigSupported,true,true,true,isConfigSupported,1,function isConfigSupported() { [native code] }"
    ), "{expected}");
    assert!(
        expected.contains("true,opus;true,opus;true,vp8;true,vp8|TypeError|"),
        "{expected}"
    );
    assert!(expected.contains(
        "InvalidStateError,InvalidStateError,InvalidStateError|true,[object EncodedAudioChunk],key,5,8,opus,true,f32,2,1,5,1,1|"
    ), "{expected}");
    assert!(expected.contains(
        "true,[object EncodedVideoChunk],key,7,33333,16,vp8,true,RGBA,2,2,7|true,vp8,function isConfigSupported() { [native code] }"
    ), "{expected}");

    let mut traced = EdgeRuntime::new().expect("traced WebCodecs runtime");
    traced.enable_proxy_trace().expect("enable WebCodecs trace");
    assert_eq!(text(&mut traced, SOURCE), expected);
    let trace = traced.proxy_trace();
    assert!(
        trace.iter().any(|entry| {
            entry.operation == "call" && entry.api.ends_with("AudioEncoder.isConfigSupported")
        }),
        "{trace:#?}"
    );
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".encode") })
    );
    assert!(trace.iter().any(|entry| {
        entry.operation == "get" && entry.api == "EncodedVideoChunk.prototype.get byteLength"
    }));
}

#[test]
fn webcodecs_codec_support_is_controlled_by_the_fingerprint_profile() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.media.audio_decoder_codecs = vec!["mp4a.40.2".to_owned()];
    options.fingerprint.media.audio_encoder_codecs = vec!["mp4a.40.2".to_owned()];
    options.fingerprint.media.video_decoder_codecs = vec!["avc1.*".to_owned()];
    options.fingerprint.media.video_encoder_codecs = vec!["avc1.*".to_owned()];
    let mut runtime = EdgeRuntime::with_options(options).expect("profiled WebCodecs runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (async () => {
              const values = await Promise.all([
                AudioDecoder.isConfigSupported({
                  codec: "opus",
                  numberOfChannels: 2,
                  sampleRate: 48000
                }),
                AudioEncoder.isConfigSupported({
                  codec: "mp4a.40.2",
                  numberOfChannels: 2,
                  sampleRate: 48000
                }),
                VideoDecoder.isConfigSupported({
                  codec: "vp8",
                  codedWidth: 2,
                  codedHeight: 2
                }),
                VideoEncoder.isConfigSupported({
                  codec: "avc1.42001e",
                  width: 2,
                  height: 2
                })
              ]);
              return values.map(value => value.supported).join(",");
            })()
            "#
        ),
        "false,true,false,true"
    );
}

#[test]
fn window_clock_callback_identifiers_are_realm_local() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              const child = frame.contentWindow;
              const parentTimer = setTimeout(() => {}, 10000);
              const childTimer = child.setTimeout(() => {}, 10000);
              const parentRaf = requestAnimationFrame(() => {});
              const childRaf = child.requestAnimationFrame(() => {});
              const parentIdle = requestIdleCallback(() => {});
              const childIdle = child.requestIdleCallback(() => {});
              clearTimeout(parentTimer);
              child.clearTimeout(childTimer);
              cancelAnimationFrame(parentRaf);
              child.cancelAnimationFrame(childRaf);
              cancelIdleCallback(parentIdle);
              child.cancelIdleCallback(childIdle);
              return [
                parentTimer, childTimer,
                parentRaf, childRaf,
                parentIdle, childIdle
              ].join(",");
            })()
            "#,
        ),
        "1,1,1,1,1,1"
    );
}

#[test]
fn animation_frame_batch_uses_one_timestamp_and_samples_document_timeline() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.frameClockSamples = [];
        requestAnimationFrame(timestamp => {
          frameClockSamples.push([
            timestamp,
            document.timeline.currentTime,
            performance.now()
          ]);
        });
        requestAnimationFrame(timestamp => {
          frameClockSamples.push([
            timestamp,
            document.timeline.currentTime,
            performance.now()
          ]);
        });
        "#,
    );
    let values = text(
        &mut runtime,
        r#"
        (() => {
          const [first, second] = frameClockSamples;
          return [
            frameClockSamples.length,
            first[0] === second[0],
            first[0] === first[1],
            second[0] === second[1],
            first[2] >= first[0] && first[2] - first[0] < 5,
            second[2] >= second[0] && second[2] - second[0] < 5
          ].join("|");
        })()
        "#,
    );
    assert_eq!(values, "2|true|true|true|true|true");
}

#[test]
fn idle_deadlines_and_timeout_flags_use_the_unified_monotonic_clock() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.idleClockSamples = [];
        requestIdleCallback(deadline => {
          idleClockSamples.push([
            "idle",
            deadline.didTimeout,
            deadline.timeRemaining()
          ]);
        });
        requestIdleCallback(deadline => {
          idleClockSamples.push([
            "timeout",
            deadline.didTimeout,
            deadline.timeRemaining()
          ]);
        }, { timeout: 0 });
        "#,
    );
    let values = text(
        &mut runtime,
        r#"
        (() => {
          const idle = idleClockSamples.find(value => value[0] === "idle");
          const timeout = idleClockSamples.find(value => value[0] === "timeout");
          return [
            idleClockSamples.length,
            idle[1],
            idle[2] > 0 && idle[2] <= 50,
            timeout[1],
            timeout[2]
          ].join("|");
        })()
        "#,
    );
    assert_eq!(values, "2|false|true|true|0");
}

#[test]
fn nested_idle_callback_runs_in_the_next_idle_period() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let values = text(
        &mut runtime,
        r#"
        new Promise(resolve => {
          requestIdleCallback(first => {
            const firstBudget = first.timeRemaining();
            const requestedAt = performance.now();
            requestIdleCallback(second => {
              const secondBudget = second.timeRemaining();
              resolve([
                first.didTimeout,
                firstBudget > 0 && firstBudget <= 17,
                second.didTimeout,
                performance.now() - requestedAt >= 15,
                secondBudget > 0 && secondBudget <= 17
              ].join("|"));
            });
          });
        })
        "#,
    );
    assert_eq!(values, "false|true|false|true|true");
}

#[test]
fn abort_signal_timeout_and_scheduler_delays_are_asynchronous_clock_tasks() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.clockTaskOrder = [];
        const signal = AbortSignal.timeout(5);
        clockTaskOrder.push(`initial:${signal.aborted}`);
        signal.addEventListener("abort", () => {
          clockTaskOrder.push(`abort:${signal.reason.name}`);
        });
        scheduler.postTask(() => {
          clockTaskOrder.push("postTask");
        }, { delay: 10 }).then(() => scheduler.yield()).then(() => {
          clockTaskOrder.push("yield");
        });
        "#,
    );
    assert_eq!(
        text(&mut runtime, "clockTaskOrder.join('|')"),
        "initial:false|abort:TimeoutError|postTask|yield"
    );
}

#[test]
fn scheduler_honors_priority_and_aborted_signals() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.schedulerSemantics = [];
        const controller = new TaskController({ priority: "background" });
        scheduler.postTask(
          () => schedulerSemantics.push("cancelled-callback"),
          { delay: 20, signal: controller.signal }
        ).then(
          () => schedulerSemantics.push("cancelled-fulfilled"),
          error => schedulerSemantics.push(`cancelled-${error.name}`)
        );
        controller.abort();
        scheduler.postTask(
          () => schedulerSemantics.push("background"),
          { priority: "background" }
        );
        scheduler.postTask(
          () => schedulerSemantics.push("user-blocking"),
          { priority: "user-blocking" }
        );
        "#,
    );
    assert_eq!(
        text(&mut runtime, "schedulerSemantics.join('|')"),
        "user-blocking|background|cancelled-AbortError"
    );
}

#[test]
fn timer_task_microtasks_inherit_the_current_timer_nesting_level() {
    let mut options = EdgeRuntimeOptions::default();
    options.deterministic.clock_epoch_ms = Some(1_700_000_000_000);
    options.deterministic.clock_step_ms = 0;
    let mut runtime = EdgeRuntime::with_options(options).expect("deterministic Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.timerNestingValues = [];
        new Promise(resolve => setTimeout(resolve, 0)).then(() => {
          const started = performance.now();
          const next = () => {
            timerNestingValues.push(performance.now() - started);
            if (timerNestingValues.length < 9) setTimeout(next, 0);
          };
          setTimeout(next, 0);
        });
        "#,
    );
    assert_eq!(
        text(&mut runtime, "timerNestingValues.join(',')"),
        "0,0,0,0,0,4,8,12,16"
    );
}

#[test]
fn animation_svg_and_performance_observer_clocks_advance_and_produce_entries() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.advancingClockResults = { animation: [], svg: [], svgAnimation: [], entries: [] };
        const animated = document.createElement("div").animate(
          [{ opacity: 0 }, { opacity: 1 }],
          { duration: 1000 }
        );
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        const svgAnimation = document.createElementNS("http://www.w3.org/2000/svg", "animate");
        svg.appendChild(svgAnimation);
        document.body.appendChild(svg);
        const svgStart = svg.getCurrentTime();
        const svgAnimationStart = svgAnimation.getCurrentTime();
        new PerformanceObserver(list => {
          advancingClockResults.entries.push(...list.getEntries().map(entry => [
            entry.entryType,
            entry.duration,
            Object.prototype.toString.call(entry)
          ]));
        }).observe({ entryTypes: ["longtask", "long-animation-frame"] });
        const topLevelStart = performance.now();
        while (performance.now() - topLevelStart < 55) {}
        requestAnimationFrame(() => {
          advancingClockResults.animation.push(animated.currentTime);
          requestAnimationFrame(() => {
            const frameStart = performance.now();
            while (performance.now() - frameStart < 55) {}
            advancingClockResults.animation.push(animated.currentTime);
            advancingClockResults.svg = [svgStart, svg.getCurrentTime()];
            advancingClockResults.svgAnimation = [svgAnimationStart, svgAnimation.getCurrentTime()];
          });
        });
        "#,
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const types = advancingClockResults.entries.map(entry => entry[0]);
              return [
                advancingClockResults.animation.length === 2,
                advancingClockResults.animation[0] === 0,
                advancingClockResults.animation[1] > advancingClockResults.animation[0],
                advancingClockResults.svg[1] > advancingClockResults.svg[0],
                advancingClockResults.svgAnimation[1] > advancingClockResults.svgAnimation[0],
                types.includes("longtask"),
                types.includes("long-animation-frame"),
                advancingClockResults.entries.every(entry => entry[1] >= 50),
                advancingClockResults.entries.some(entry => entry[2] === "[object PerformanceLongTaskTiming]"),
                advancingClockResults.entries.some(entry => entry[2] === "[object PerformanceLongAnimationFrameTiming]")
              ].join("|");
            })()
            "#,
        ),
        "true|true|true|true|true|true|true|true|true|true"
    );
}

#[test]
fn performance_observer_buffered_delivers_historical_long_tasks_and_loaf_entries() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        const historicalLongTaskStart = performance.now();
        while (performance.now() - historicalLongTaskStart < 55) {}
        requestAnimationFrame(() => {
          const historicalFrameStart = performance.now();
          while (performance.now() - historicalFrameStart < 55) {}
        });
        "#,
    );
    text(
        &mut runtime,
        r#"
        globalThis.historicalPerformanceEntries = [];
        for (const type of ["longtask", "long-animation-frame"]) {
          new PerformanceObserver(list => {
            historicalPerformanceEntries.push(...list.getEntries().map(entry => entry.entryType));
          }).observe({ type, buffered: true });
        }
        "#,
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"[
              historicalPerformanceEntries.filter(type => type === "longtask").length >= 2,
              historicalPerformanceEntries.includes("long-animation-frame"),
              performance.getEntriesByType("longtask").length,
              performance.getEntriesByType("long-animation-frame").length >= 1
            ].join("|")"#,
        ),
        "true|true|0|true"
    );
}

#[test]
fn rendering_updates_produce_element_lcp_and_layout_shift_entries() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.renderingEntryTypes = [];
        for (const type of ["element", "largest-contentful-paint", "layout-shift"]) {
          new PerformanceObserver(list => {
            renderingEntryTypes.push(...list.getEntries().map(entry => entry.entryType));
          }).observe({ type, buffered: true });
        }
        const timed = document.createElement("div");
        timed.id = "timed-content";
        timed.setAttribute("elementtiming", "timed-identifier");
        timed.style.cssText = "width:400px;height:80px";
        timed.textContent = "contentful text";
        document.body.appendChild(timed);
        globalThis.renderingEntryRects = [];
        requestAnimationFrame(() => {
          renderingEntryRects.push(timed.getBoundingClientRect().y);
          requestAnimationFrame(() => {
            timed.setAttribute("style", "width:400px;height:80px;margin-top:40px");
            renderingEntryRects.push(timed.getBoundingClientRect().y);
          });
        });
        "#,
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"[
              renderingEntryTypes.includes("element"),
              renderingEntryTypes.includes("largest-contentful-paint"),
              renderingEntryTypes.includes("layout-shift"),
              renderingEntryRects.join(",")
            ].join("|")"#,
        ),
        "true|true|true|8,48"
    );
}

#[test]
fn image_element_timing_waits_for_load_and_truncates_the_exposed_url() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.imageTimingEntries = [];
        new PerformanceObserver(list => {
          imageTimingEntries.push(...list.getEntries());
        }).observe({type: "element", buffered: true});
        globalThis.timedImage = document.createElement("img");
        timedImage.setAttribute("elementtiming", "loaded-image");
        timedImage.width = 320;
        timedImage.height = 180;
        timedImage.src = "data:image/svg+xml," + encodeURIComponent(
          "<svg xmlns='http://www.w3.org/2000/svg' width='320' height='180'>" +
          "<rect width='320' height='180' fill='navy'/></svg>"
        );
        document.body.appendChild(timedImage);
        requestAnimationFrame(() => requestAnimationFrame(() => {}));
        "#,
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const entry = imageTimingEntries.find(value => value.identifier === "loaded-image");
              return [
                timedImage.complete,
                timedImage.currentSrc.length > 100,
                entry && entry.url.length,
                entry && entry.url === timedImage.currentSrc.slice(0, 100),
                entry && entry.loadTime <= entry.renderTime
              ].join("|");
            })()
            "#,
        ),
        "true|true|100|true|true"
    );
}

#[test]
fn host_click_is_trusted_and_produces_event_timing_without_trusting_script_dispatch() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let setup = text(
        &mut runtime,
        r#"
        (() => {
          const button = document.createElement("button");
          button.id = "host-click-target";
          button.style.cssText =
            "position:fixed;left:100px;top:100px;width:80px;height:30px";
          document.body.appendChild(button);
          globalThis.hostClickEvents = [];
          globalThis.hostClickRetained = null;
          globalThis.hostInteractionId = 0;
          globalThis.hostInteractionIds = [];
          const normalizedInteractionId = entry => {
            if (entry.interactionId === 0) return 0;
            hostInteractionId ||= entry.interactionId;
            hostInteractionIds.push(entry.interactionId);
            return entry.interactionId === hostInteractionId
              ? "interaction"
              : "mismatch";
          };
          for (const type of ["pointerdown", "mousedown", "pointerup", "mouseup", "click"]) {
            button.addEventListener(type, event => {
              hostClickEvents.push([
                type,
                event.isTrusted,
                event.clientX,
                event.clientY,
                event instanceof PointerEvent,
                event instanceof MouseEvent
              ].join(":"));
              if (type === "click" && event.isTrusted) hostClickRetained = event;
            });
          }
          globalThis.hostEventEntries = [];
          globalThis.hostFirstEntries = [];
          const durationBucket = duration =>
            duration > 0 && Number.isInteger(duration / 8) ? "8ms-bucket" : duration;
          new PerformanceObserver(list => {
            hostEventEntries.push(...list.getEntries().map(entry =>
              [entry.name, entry.entryType, durationBucket(entry.duration),
               normalizedInteractionId(entry),
               entry.target === button].join(":")));
          }).observe({type: "event"});
          new PerformanceObserver(list => {
            hostFirstEntries.push(...list.getEntries().map(entry =>
              [entry.name, entry.entryType, durationBucket(entry.duration),
               normalizedInteractionId(entry),
               entry.target === button].join(":")));
          }).observe({type: "first-input", buffered: true});
          return "ready";
        })()
        "#,
    );
    assert_eq!(setup, "ready");
    assert!(
        runtime
            .dispatch_host_click(&crate::HostClickInput::primary(120.0, 115.0))
            .expect("host click")
    );
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const beforeSynthetic = hostClickEvents.join("|");
          document.getElementById("host-click-target").dispatchEvent(hostClickRetained);
          return [
            beforeSynthetic,
            hostClickEvents.at(-1),
            document.activeElement.id,
            performance.eventCounts.get("pointerdown"),
            performance.eventCounts.get("click"),
            performance.interactionCount,
            hostInteractionId >= 107 && hostInteractionId <= 10007,
            hostEventEntries.join("|"),
            hostFirstEntries.join("|")
          ].join("||");
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "pointerdown:true:120:115:true:true|",
            "mousedown:true:120:115:false:true|",
            "pointerup:true:120:115:true:true|",
            "mouseup:true:120:115:false:true|",
            "click:true:120:115:true:true||",
            "click:false:120:115:true:true||",
            "host-click-target||1||1||1||true||",
            "pointerover:event:8ms-bucket:0:true|",
            "pointerenter:event:8ms-bucket:0:false|pointerenter:event:8ms-bucket:0:false|",
            "pointerenter:event:8ms-bucket:0:false|pointerenter:event:8ms-bucket:0:false|",
            "mouseover:event:8ms-bucket:0:true|",
            "pointerdown:event:8ms-bucket:interaction:true|mousedown:event:8ms-bucket:0:true|",
            "pointerup:event:8ms-bucket:interaction:true|mouseup:event:8ms-bucket:0:true|click:event:8ms-bucket:interaction:true||",
            "pointerdown:first-input:8ms-bucket:interaction:true"
        )
    );
    assert!(
        runtime
            .dispatch_host_click(&crate::HostClickInput::primary(120.0, 115.0))
            .expect("second host click")
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const ids = [...new Set(hostInteractionIds)];
              return [
                ids.length,
                ids[0] === hostInteractionId,
                ids[1] - ids[0]
              ].join("|");
            })()
            "#,
        ),
        "2|true|7"
    );
}

#[test]
fn normal_flow_block_rects_stack_and_collapse_adjacent_vertical_margins() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const first = document.createElement("div");
              first.style.cssText = "width:100px;height:20px;margin-bottom:10px";
              const second = document.createElement("div");
              second.style.cssText = "width:100px;height:30px;margin-top:15px";
              document.body.append(first, second);
              const firstRect = first.getBoundingClientRect();
              const secondRect = second.getBoundingClientRect();
              return [
                firstRect.x, firstRect.y, firstRect.width, firstRect.height,
                secondRect.x, secondRect.y, secondRect.width, secondRect.height
              ].join("|");
            })()
            "#,
        ),
        "8|8|100|20|8|43|100|30"
    );
}

#[test]
fn windows_button_intrinsics_and_mixed_inline_block_flow_match_edge_https_evidence() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.viewport_width = 1280.0;
    options.fingerprint.screen.viewport_height = 720.0;
    let mut runtime = EdgeRuntime::with_options(options).expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              document.body.replaceChildren();
              const button = document.createElement("button");
              button.textContent = "Performance event target";
              const block = document.createElement("div");
              block.style.cssText = "font-size:48px;width:700px;height:100px";
              const image = document.createElement("img");
              image.width = 320;
              image.height = 180;
              document.body.append(button, block, image);
              const rect = element => {
                const value = element.getBoundingClientRect();
                return [value.x, value.y, value.width, value.height].join(",");
              };
              const style = getComputedStyle(button);
              return [
                rect(button), rect(block), rect(image),
                style.display, style.font, style.padding, style.border
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "8,10,166.421875,21|8,31,700,100|8,131,320,180|",
            "inline-block|13.3333px Arial|1px 6px|2px outset rgb(0, 0, 0)"
        )
    );
}

#[test]
fn inline_flex_grid_and_positioned_layout_match_edge_https_evidence() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.viewport_width = 1280.0;
    options.fingerprint.screen.viewport_height = 720.0;
    let mut runtime = EdgeRuntime::with_options(options).expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const rect = element => {
                const value = element.getBoundingClientRect();
                return [value.x, value.y, value.width, value.height].join(",");
              };
              const make = (tag, style) => {
                const value = document.createElement(tag);
                value.style.cssText = style;
                return value;
              };
              document.body.replaceChildren();
              document.body.style.cssText = "margin:8px";

              const inline = make("div", "width:140px;height:50px");
              const inlineA = make("span", "display:inline-block;width:40px;height:20px;margin-right:5px");
              const inlineB = make("span", "display:inline-block;width:50px;height:20px;margin-left:3px");
              inline.append(inlineA, inlineB);
              document.body.appendChild(inline);

              const flex = make("div", "display:flex;width:240px;height:80px;gap:10px;justify-content:center;align-items:center;margin-top:10px");
              const flexA = make("div", "width:40px;height:20px");
              const flexB = make("div", "width:60px;height:30px");
              flex.append(flexA, flexB);
              document.body.appendChild(flex);

              const grid = make("div", "display:grid;grid-template-columns:70px 90px;column-gap:8px;row-gap:6px;width:220px;margin-top:10px");
              const gridA = make("div", "height:20px");
              const gridB = make("div", "height:30px");
              const gridC = make("div", "height:15px");
              grid.append(gridA, gridB, gridC);
              document.body.appendChild(grid);

              const relative = make("div", "position:relative;width:200px;height:100px;margin-top:10px;padding:5px;border:2px solid black");
              const absolute = make("div", "position:absolute;left:25%;top:10px;width:50%;height:20px");
              relative.appendChild(absolute);
              document.body.appendChild(relative);

              return [
                rect(document.body),
                rect(inline), rect(inlineA), rect(inlineB),
                rect(flex), rect(flexA), rect(flexB),
                rect(grid), rect(gridA), rect(gridB), rect(gridC),
                rect(relative), rect(absolute)
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "8,8,1264,325|8,8,140,50|8,8,40,20|56,8,50,20|",
            "8,68,240,80|73,98,40,20|123,93,60,30|",
            "8,158,220,51|8,158,70,20|86,158,90,30|8,194,70,15|",
            "8,219,214,114|62.5,231,105,20"
        )
    );
}

#[test]
fn computed_style_includes_edge_user_agent_display_defaults_and_hidden_rule() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const names = ["div", "span", "table", "tbody", "tr", "td", "li", "img", "script"];
              const elements = names.map(name => document.createElement(name));
              elements.forEach(element => document.body.appendChild(element));
              const displays = elements.map(element => getComputedStyle(element).display);
              const detached = document.createElement("div");
              detached.style.display = "block";
              const hidden = document.createElement("div");
              hidden.hidden = true;
              document.body.appendChild(hidden);
              const untilFound = document.createElement("div");
              untilFound.setAttribute("hidden", "until-found");
              document.body.appendChild(untilFound);
              return [
                ...displays,
                getComputedStyle(detached).display,
                getComputedStyle(hidden).display,
                getComputedStyle(untilFound).display,
                getComputedStyle(elements[0]).length,
                getComputedStyle(elements[0]).item(0),
                getComputedStyle(elements[0]).item(473),
                getComputedStyle(elements[0]).cssText
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "block|inline|table|table-row-group|table-row|table-cell|list-item|inline|none|",
            "|none|block|474|accent-color|-webkit-writing-mode|"
        )
    );
}

#[test]
fn connected_div_exposes_all_474_edge_computed_values_without_empty_slots() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.screen.viewport_width = 1280.0;
    options.fingerprint.screen.viewport_height = 720.0;
    let mut runtime = EdgeRuntime::with_options(options).expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const element = document.createElement("div");
              document.body.appendChild(element);
              const style = getComputedStyle(element);
              const empty = Array.from(style).filter(name => style.getPropertyValue(name) === "");
              const keys = Reflect.ownKeys(style);
              const zero = Object.getOwnPropertyDescriptor(style, "0");
              return [
                style.length,
                empty.length,
                style.accentColor,
                style.scrollbarColor,
                style.stroke,
                style.width,
                style.height,
                style.inlineSize,
                style.blockSize,
                style.transformOrigin,
                style.perspectiveOrigin,
                keys.length,
                keys.slice(0, 5).join(","),
                keys.slice(-5).join(","),
                zero.value,
                zero.writable,
                zero.enumerable,
                zero.configurable
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "474|0|auto|auto|none|1264px|0px|1264px|0px|632px 0px|632px 0px|",
            "1218|0,1,2,3,4|writingMode,x,y,zIndex,zoom|",
            "accent-color|false|true|true"
        )
    );
}

#[test]
fn computed_style_inherits_edge_text_list_svg_and_interaction_properties() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const parent = document.createElement("div");
              parent.style.cssText = [
                "color:rgb(4,5,6)", "direction:rtl", "font-family:Arial",
                "font-size:20px", "font-weight:700", "line-height:30px",
                "text-align:right", "visibility:hidden", "cursor:pointer",
                "white-space:pre", "letter-spacing:2px", "word-spacing:3px",
                "text-transform:uppercase", "fill:rgb(7,8,9)",
                "stroke:rgb(10,11,12)", "list-style-position:inside",
                "list-style-type:square", "tab-size:7"
              ].join(";");
              const child = document.createElement("div");
              parent.appendChild(child);
              document.body.appendChild(parent);
              const style = getComputedStyle(child);
              return [
                style.color, style.direction, style.fontFamily, style.fontSize,
                style.fontWeight, style.lineHeight, style.textAlign,
                style.visibility, style.cursor, style.whiteSpace,
                style.letterSpacing, style.wordSpacing, style.textTransform,
                style.fill, style.stroke, style.listStylePosition,
                style.listStyleType, style.tabSize
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "rgb(4, 5, 6)|rtl|Arial|20px|700|30px|right|hidden|pointer|pre|",
            "2px|3px|uppercase|rgb(7, 8, 9)|rgb(10, 11, 12)|inside|square|7"
        )
    );
}

#[test]
fn computed_currentcolor_dependents_follow_the_inherited_edge_color() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const parent = document.createElement("div");
              parent.style.color = "rgb(31,32,33)";
              const child = document.createElement("div");
              parent.appendChild(child);
              document.body.appendChild(parent);
              const style = getComputedStyle(child);
              return [
                style.borderTopColor, style.borderRightColor,
                style.borderBottomColor, style.borderLeftColor,
                style.borderBlockStartColor, style.borderBlockEndColor,
                style.borderInlineStartColor, style.borderInlineEndColor,
                style.caretColor, style.columnRuleColor, style.outlineColor,
                style.textDecorationColor, style.textEmphasisColor,
                style.webkitTextFillColor, style.webkitTextStrokeColor
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "rgb(31, 32, 33)|rgb(31, 32, 33)|rgb(31, 32, 33)|",
            "rgb(31, 32, 33)|rgb(31, 32, 33)|rgb(31, 32, 33)|",
            "rgb(31, 32, 33)|rgb(31, 32, 33)|rgb(31, 32, 33)|",
            "rgb(31, 32, 33)|rgb(31, 32, 33)|rgb(31, 32, 33)|",
            "rgb(31, 32, 33)|rgb(31, 32, 33)|rgb(31, 32, 33)"
        )
    );
}

#[test]
fn computed_style_inherits_extended_edge_table_typography_and_webkit_properties() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const parent = document.createElement("div");
              parent.style.cssText = [
                "border-collapse:collapse", "border-spacing:5px 6px",
                "caption-side:bottom", "color-scheme:dark", "empty-cells:hide",
                'font-feature-settings:"kern" 0', "font-kerning:none",
                "font-optical-sizing:none", "font-variant-caps:small-caps",
                'font-variation-settings:"wght" 500', "hyphens:none",
                "image-rendering:pixelated", "line-break:strict", "orphans:3",
                "overflow-wrap:anywhere", "pointer-events:none",
                'quotes:"<" ">"', "ruby-position:under",
                "text-decoration-skip-ink:none",
                "text-emphasis-color:rgb(13,14,15)",
                "text-emphasis-position:under", "text-indent:9px",
                "text-rendering:optimizelegibility",
                "text-shadow:rgb(16,17,18) 1px 2px 3px",
                "text-size-adjust:80%", "widows:4", "word-break:break-all",
                "writing-mode:vertical-rl",
                "-webkit-text-fill-color:rgb(19,20,21)",
                "-webkit-text-stroke-color:rgb(22,23,24)",
                "-webkit-text-stroke-width:2px"
              ].join(";");
              const child = document.createElement("div");
              parent.appendChild(child);
              document.body.appendChild(parent);
              const style = getComputedStyle(child);
              const names = [
                "border-collapse", "border-spacing", "caption-side", "color-scheme",
                "empty-cells", "font-feature-settings", "font-kerning",
                "font-optical-sizing", "font-variant-caps", "font-variation-settings",
                "hyphens", "image-rendering", "line-break", "orphans",
                "overflow-wrap", "pointer-events", "quotes", "ruby-position",
                "text-decoration-skip-ink", "text-emphasis-color",
                "text-emphasis-position", "text-indent", "text-rendering",
                "text-shadow", "text-size-adjust", "widows", "word-break",
                "writing-mode", "-webkit-text-fill-color",
                "-webkit-text-stroke-color", "-webkit-text-stroke-width"
              ];
              return names.map(name => style.getPropertyValue(name)).join("|");
            })()
            "#,
        ),
        concat!(
            "collapse|5px 6px|bottom|dark|hide|\"kern\" 0|none|none|small-caps|",
            "\"wght\" 500|none|pixelated|strict|3|anywhere|none|\"<\" \">\"|",
            "under|none|rgb(13, 14, 15)|under|9px|optimizelegibility|",
            "rgb(16, 17, 18) 1px 2px 3px|80%|4|break-all|vertical-rl|",
            "rgb(19, 20, 21)|rgb(22, 23, 24)|2px"
        )
    );
}

#[test]
fn computed_style_resolves_edge_core_initial_values_and_shorthands() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const element = document.createElement("div");
              element.style.cssText = "width:140px;height:50px";
              document.body.appendChild(element);
              const value = getComputedStyle(element);
              return [
                value.font, value.fontFamily, value.fontSize, value.fontStyle,
                value.fontWeight, value.lineHeight, value.color, value.visibility,
                value.position, value.boxSizing, value.width, value.height,
                value.padding, value.borderTopWidth,
                value.getPropertyValue("font"), value.getPropertyValue("padding")
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "16px \"Times New Roman\"|\"Times New Roman\"|16px|normal|400|normal|",
            "rgb(0, 0, 0)|visible|static|content-box|140px|50px|0px|0px|",
            "16px \"Times New Roman\"|0px"
        )
    );
}

#[test]
fn default_times_new_roman_canvas_width_matches_edge_150_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let value = text(
        &mut runtime,
        r#"
        (() => {
          const context = document.createElement("canvas").getContext("2d");
          context.font = '48px "Times New Roman"';
          return context.measureText(
            "A large contentful text element used by the browser API audit"
          ).width;
        })()
        "#,
    )
    .parse::<f64>()
    .expect("Canvas width");
    assert!((value - 1190.7890625).abs() < 0.000_001, "width={value}");
}

#[test]
fn cross_origin_isolation_selects_edge_high_resolution_clock_precision() {
    let options = EdgeRuntimeOptions {
        cross_origin_isolated: true,
        ..Default::default()
    };
    let mut runtime = EdgeRuntime::with_options(options).expect("isolated Edge runtime");
    let values = text(
        &mut runtime,
        r#"
        (() => {
          let previous = performance.now();
          let minimum = Infinity;
          for (let index = 0; index < 500000; index++) {
            const current = performance.now();
            const delta = current - previous;
            if (delta > 0 && delta < minimum) minimum = delta;
            previous = current;
          }
          return [crossOriginIsolated, minimum].join("|");
        })()
        "#,
    );
    let mut values = values.split('|');
    assert_eq!(values.next(), Some("true"));
    let minimum = values
        .next()
        .expect("isolated clock minimum")
        .parse::<f64>()
        .expect("numeric isolated clock minimum");
    assert!(
        (0.004..0.1).contains(&minimum),
        "isolated clock resolution was {minimum}ms"
    );
}

#[test]
fn deterministic_temporal_now_shares_the_date_epoch_without_shape_drift() {
    let mut options = EdgeRuntimeOptions::default();
    options.deterministic.clock_epoch_ms = Some(1_700_000_000_000);
    options.deterministic.clock_step_ms = 0;
    let mut runtime = EdgeRuntime::with_options(options).expect("deterministic Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const instant = Temporal.Now.instant();
              return [
                instant.epochNanoseconds.toString(),
                Date.now(),
                Temporal.Now.timeZoneId(),
                Object.getOwnPropertyNames(Temporal.Now).join(","),
                Temporal.Now.instant.name,
                Temporal.Now.instant.length,
                Function.prototype.toString.call(Temporal.Now.instant),
                Temporal.Now.plainDateISO().constructor.name,
                Temporal.Now.plainTimeISO().constructor.name,
                Temporal.Now.plainDateTimeISO().constructor.name,
                Temporal.Now.zonedDateTimeISO().constructor.name
              ].join("|");
            })()
            "#,
        ),
        "1700000000000000000|1700000000000|Asia/Shanghai|instant,timeZoneId,plainDateTimeISO,zonedDateTimeISO,plainDateISO,plainTimeISO|instant|0|function instant() { [native code] }|PlainDate|PlainTime|PlainDateTime|ZonedDateTime"
    );
}

#[test]
fn deterministic_same_deadline_timers_keep_cross_realm_registration_order() {
    let mut options = EdgeRuntimeOptions::default();
    options.deterministic.clock_epoch_ms = Some(1_700_000_000_000);
    options.deterministic.clock_step_ms = 0;
    let mut runtime = EdgeRuntime::with_options(options).expect("deterministic Edge runtime");
    text(
        &mut runtime,
        r#"
        globalThis.sameDeadlineOrder = [];
        const frame = document.createElement("iframe");
        document.body.appendChild(frame);
        frame.contentWindow.setTimeout(() => sameDeadlineOrder.push("child"), 0);
        setTimeout(() => sameDeadlineOrder.push("parent"), 0);
        "#,
    );
    assert_eq!(
        text(&mut runtime, "sameDeadlineOrder.join(',')"),
        "child,parent"
    );
}

#[test]
fn computed_overflow_shorthand_and_inherited_writing_mode_drive_used_axes() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const parent = document.createElement("div");
              parent.style.cssText =
                "width:120px;height:80px;overflow:auto;writing-mode:vertical-rl";
              const child = document.createElement("div");
              child.style.cssText = "width:20px;height:10px";
              parent.appendChild(child);
              document.body.appendChild(parent);
              const parentStyle = getComputedStyle(parent);
              const childStyle = getComputedStyle(child);
              const probe = document.createElement("div");
              probe.style.cssText =
                "font:italic small-caps 700 16px/20px Arial;overflow-x:hidden;overflow:auto;text-size-adjust:80%";
              document.body.appendChild(probe);
              const probeStyle = getComputedStyle(probe);
              const firstOverflow = `${probeStyle.overflowX},${probeStyle.overflowY}`;
              const fontValues = [
                probe.style.fontFamily,
                probeStyle.fontFamily,
                probeStyle.fontSize,
                probeStyle.lineHeight,
                probeStyle.webkitTextSizeAdjust
              ].join(",");
              probe.style.cssText = "overflow:auto;overflow-x:hidden";
              const secondProbeStyle = getComputedStyle(probe);
              return [
                parentStyle.overflowX,
                parentStyle.overflowY,
                childStyle.writingMode,
                childStyle.inlineSize,
                childStyle.blockSize,
                firstOverflow,
                fontValues,
                `${secondProbeStyle.overflowX},${secondProbeStyle.overflowY}`
              ].join("|");
            })()
            "#,
        ),
        "auto|auto|vertical-rl|10px|20px|auto,auto|Arial,Arial,16px,20px,80%|hidden,auto"
    );
}

#[test]
fn edge_static_webidl_factories_and_descriptors_match_their_platform_shapes() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const point = DOMPoint.fromPoint({x: 2, y: 3, z: 4, w: 5});
              const rect = DOMRect.fromRect({x: 1, y: 2, width: -3, height: -4});
              const quad = DOMQuad.fromRect({x: 1, y: 2, width: 3, height: 4});
              const matrix = DOMMatrixReadOnly.fromFloat32Array(
                new Float32Array([1, 2, 3, 4, 5, 6])
              );
              const safe = Document.parseHTML(
                '<script>bad()</script><p onclick="x()">t</p>'
              );
              const unsafe = Document.parseHTMLUnsafe(
                '<script>bad()</script><p onclick="x()">t</p>'
              );
              const sources = PressureObserver.knownSources;
              const fullscreen = Object.getOwnPropertyDescriptor(
                Document.prototype,
                "fullscreen"
              );
              const before = document.fullscreen;
              document.fullscreen = true;
              return [
                point.constructor.name,
                [point.x, point.y, point.z, point.w].join(","),
                [rect.top, rect.right, rect.bottom, rect.left].join(","),
                [quad.p1.x, quad.p2.x, quad.p3.y, quad.p4.y].join(","),
                Array.from(matrix.toFloat64Array()).join(","),
                [safe.URL, safe.body.innerHTML, safe.scripts.length].join(","),
                [unsafe.URL, unsafe.body.innerHTML, unsafe.scripts.length].join(","),
                [
                  HTMLScriptElement.supports("classic"),
                  HTMLScriptElement.supports("module"),
                  HTMLScriptElement.supports("text/javascript")
                ].join(","),
                [
                  sources.join(","),
                  sources === PressureObserver.knownSources,
                  Object.isFrozen(sources)
                ].join(","),
                [
                  fullscreen.set.name,
                  fullscreen.set.length,
                  Function.prototype.toString.call(fullscreen.set),
                  before === document.fullscreen,
                  !Object.hasOwn(document, "fullscreen")
                ].join(",")
              ].join("|");
            })()
            "#,
        ),
        "DOMPoint|2,3,4,5|-2,1,2,-2|1,4,6,6|1,2,0,0,3,4,0,0,0,0,1,0,5,6,0,1|about:blank,<p>t</p>,0|about:blank,<p onclick=\"x()\">t</p>,1|true,true,false|cpu,true,true|set fullscreen,1,function set fullscreen() { [native code] },true,true"
    );
}

#[test]
fn dom_matrix_typed_array_factories_match_edge_type_and_length_conversion() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const capture = callback => {
                try { callback(); return "no error"; }
                catch (error) { return `${error.name}: ${error.message}`; }
              };
              const mutable = DOMMatrix.fromFloat32Array(
                new Float32Array([1, 2, 3, 4, 5, 6])
              );
              return [
                Array.from(mutable.toFloat64Array()).join(","),
                mutable.constructor.name,
                capture(() => DOMMatrix.fromFloat32Array([1, 2, 3, 4, 5, 6])),
                capture(() => DOMMatrix.fromFloat64Array()),
                capture(() => DOMMatrix.fromFloat64Array(new Float64Array(5))),
                capture(() => DOMMatrixReadOnly.fromFloat32Array()),
                capture(() => DOMMatrixReadOnly.fromFloat32Array(new Float32Array(5))),
                capture(() => DOMMatrixReadOnly.fromFloat64Array(new Float64Array(5)))
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "1,2,0,0,3,4,0,0,0,0,1,0,5,6,0,1|DOMMatrix|",
            "TypeError: Failed to execute 'fromFloat32Array' on 'DOMMatrix': parameter 1 is not of type 'Float32Array'.|",
            "TypeError: Failed to execute 'fromFloat64Array' on 'DOMMatrix': 1 argument required, but only 0 present.|",
            "TypeError: Failed to execute 'fromFloat64Array' on 'DOMMatrix': The sequence must contain 6 elements for a 2D matrix or 16 elements for a 3D matrix.|",
            "TypeError: Failed to execute 'fromFloat32Array' on 'DOMMatrixReadOnly': 1 argument required, but only 0 present.|",
            "TypeError: Failed to execute 'fromFloat32Array' on 'DOMMatrixReadOnly': The sequence must contain 6 elements for a 2D matrix or 16 elements a for 3D matrix.|",
            "TypeError: Failed to execute 'fromFloat64Array' on 'DOMMatrixReadOnly': The sequence must contain 6 elements for a 2D matrix or 16 elements for a 3D matrix."
        )
    );
}

#[test]
fn navigator_legacy_arrays_and_private_font_set_prototype_match_edge_exotics() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const descriptor = (object, key) => {
                const value = Object.getOwnPropertyDescriptor(object, key);
                return [value.writable, value.enumerable, value.configurable].join(",");
              };
              const plugins = navigator.plugins;
              const mimeTypes = navigator.mimeTypes;
              const pluginName = plugins[0].name;
              const mimeName = mimeTypes[0].type;
              const beforePlugin = plugins[0];
              const beforeMime = mimeTypes[0];
              let pluginWrite;
              let mimeWrite;
              try { Function("p", '"use strict"; p[0] = 1')(plugins); }
              catch (error) { pluginWrite = error.name; }
              try { Function("m", '"use strict"; m[0] = 1')(mimeTypes); }
              catch (error) { mimeWrite = error.name; }
              const fontPrototype = Object.getPrototypeOf(document.fonts);
              return [
                descriptor(plugins, "0"),
                descriptor(plugins, pluginName),
                descriptor(mimeTypes, "0"),
                descriptor(mimeTypes, mimeName),
                Object.keys(plugins).join(","),
                Object.keys(mimeTypes).join(","),
                delete plugins[0],
                delete plugins[pluginName],
                delete mimeTypes[0],
                delete mimeTypes[mimeName],
                plugins[0] === beforePlugin,
                mimeTypes[0] === beforeMime,
                pluginWrite,
                mimeWrite,
                Reflect.ownKeys(fontPrototype).map(String).join(","),
                document.fonts.constructor.name,
                !Object.hasOwn(fontPrototype, "constructor")
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "false,true,true|false,false,true|false,true,true|false,false,true|",
            "0,1,2,3,4|0,1|false|false|false|false|true|true|TypeError|TypeError|",
            "onloading,onloadingdone,onloadingerror,ready,status,size,check,load,add,clear,delete,entries,forEach,has,keys,values,Symbol(Symbol.toStringTag),Symbol(Symbol.iterator)|",
            "EventTarget|true"
        )
    );
}

#[test]
fn dom_collection_exotics_and_css_named_properties_match_edge_instances() {
    let mut options = EdgeRuntimeOptions::default();
    options.page = Some(PageInit {
        html: concat!(
            "<!doctype html><html><head><style>.x{color:red}</style></head><body>",
            "<form id=f><input id=i name=n class=\"a b\"><select id=s>",
            "<option id=o>A</option></select></form>",
            "<div id=d style=\"width:10px\"></div></body></html>"
        )
        .to_owned(),
        ..PageInit::default()
    });
    let mut runtime = EdgeRuntime::with_options(options).expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const descriptor = (object, key) => {
                const value = Object.getOwnPropertyDescriptor(object, key);
                return value
                  ? [value.writable, value.enumerable, value.configurable].join(",")
                  : "missing";
              };
              const input = document.getElementById("i");
              const select = document.getElementById("s");
              const style = document.getElementById("d").style;
              const rules = document.styleSheets[0].cssRules;
              const types = new DataTransfer().types;
              const option = new Option("X", "x");
              select.options[3] = option;
              return [
                descriptor(input.attributes, "id"),
                descriptor(input.attributes, "0"),
                delete input.attributes.id,
                Reflect.ownKeys(input.classList).join(","),
                descriptor(input.classList, "0"),
                delete input.classList[0],
                descriptor(rules, "0"),
                delete rules[0],
                descriptor(document.body.children, "f"),
                descriptor(document.getElementById("f").elements, "i"),
                descriptor(select.options, "0"),
                select.options[3] === option,
                select.options.length,
                select.textContent,
                Object.isFrozen(types),
                Object.getOwnPropertyDescriptor(types, "length").writable,
                Reflect.ownKeys(style)
                  .filter(key => !Object.getOwnPropertyDescriptor(style, key))
                  .join(","),
                "epubCaptionSide" in style
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "false,false,true|false,true,true|false|0,1|false,true,true|false|",
            "false,true,true|false|false,false,true|false,true,true|true,true,true|",
            "true|4|AX|true|false|",
            "epubCaptionSide,epubTextCombine,epubTextEmphasis,epubTextEmphasisColor,",
            "epubTextEmphasisStyle,epubTextOrientation,epubTextTransform,epubWordBreak,",
            "epubWritingMode|false"
        )
    );
}

#[test]
fn webidl_array_like_iterators_remain_generic_like_edge() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const receiver = {0: "x", 1: "y", length: 2};
              const entries = [[0, "x"], [1, "y"]];
              const values = ["x", "y"];
              const keys = [0, 1];
              const targets = [
                [NodeList.prototype.entries, entries],
                [NodeList.prototype.keys, keys],
                [NodeList.prototype.values, values],
                [NodeList.prototype[Symbol.iterator], values],
                [DOMTokenList.prototype.entries, entries],
                [DOMTokenList.prototype.keys, keys],
                [DOMTokenList.prototype.values, values],
                [DOMTokenList.prototype[Symbol.iterator], values],
                [HTMLCollection.prototype[Symbol.iterator], values],
                [NamedNodeMap.prototype[Symbol.iterator], values],
                [FileList.prototype[Symbol.iterator], values],
                [MimeTypeArray.prototype[Symbol.iterator], values],
                [PluginArray.prototype[Symbol.iterator], values],
                [CSSRuleList.prototype[Symbol.iterator], values],
                [StyleSheetList.prototype[Symbol.iterator], values]
              ];
              return targets.every(([method, expected]) =>
                JSON.stringify(Array.from(method.call(receiver))) ===
                  JSON.stringify(expected)
              );
            })()
            "#,
        ),
        "true"
    );
}

#[test]
fn core_dom_methods_reject_invalid_receivers_like_edge() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const illegal = callback => {
                try { callback(); return false; }
                catch (error) {
                  return error.name === "TypeError" &&
                    error.message === "Illegal invocation";
                }
              };
              return [
                () => EventTarget.prototype.addEventListener.call({}, "x", null),
                () => Node.prototype.isEqualNode.call({}, null),
                () => HTMLElement.prototype.focus.call({}),
                () => HTMLElement.prototype.blur.call({}),
                () => HTMLSelectElement.prototype.remove.call({}, 0),
                () => DOMTokenList.prototype.supports.call({}, "x"),
                () => DataTransferItemList.prototype.clear.call({}),
                () => DataTransferItemList.prototype.remove.call({}, 0)
              ].every(illegal);
            })()
            "#,
        ),
        "true"
    );
}

#[test]
fn rendering_context_methods_reject_invalid_receivers_like_edge() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (async () => {
              const illegal = callback => {
                try { callback(); return false; }
                catch (error) {
                  return error.name === "TypeError" &&
                    error.message === "Illegal invocation";
                }
              };
              const synchronous = [
                () => CanvasRenderingContext2D.prototype.createLinearGradient
                  .call({}, 0, 0, 1, 1),
                () => OffscreenCanvasRenderingContext2D.prototype.setLineDash
                  .call({}, []),
                () => WebGLRenderingContext.prototype.deleteBuffer.call({}, null),
                () => WebGLRenderingContext.prototype.getProgramParameter
                  .call({}, null, 0),
                () => WebGL2RenderingContext.prototype.deleteQuery.call({}, null),
                () => WebGL2RenderingContext.prototype.getSyncParameter
                  .call({}, null, 0)
              ].every(illegal);
              const webgl1 = WebGLRenderingContext.prototype.makeXRCompatible.call({});
              const webgl2 = WebGL2RenderingContext.prototype.makeXRCompatible.call({});
              const rejection = async (promise, interfaceName) => {
                try { await promise; return false; }
                catch (error) {
                  return error.name === "TypeError" && error.message ===
                    `Failed to execute 'makeXRCompatible' on '${interfaceName}': Illegal invocation`;
                }
              };
              return synchronous &&
                webgl1 instanceof Promise &&
                webgl2 instanceof Promise &&
                await rejection(webgl1, "WebGLRenderingContext") &&
                await rejection(webgl2, "WebGL2RenderingContext");
            })()
            "#,
        ),
        "true"
    );
}

#[test]
fn webidl_receiver_order_and_promise_rejections_match_edge() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (async () => {
              const illegal = callback => {
                try { callback(); return false; }
                catch (error) {
                  return error.name === "TypeError" &&
                    error.message === "Illegal invocation";
                }
              };
              const synchronous = [
                () => Object.getOwnPropertyDescriptor(
                  DataTransfer.prototype, "dropEffect").set.call({}, null),
                () => MediaQueryList.prototype.addListener.call({}, null),
                () => Selection.prototype.removeRange.call({}, null),
                () => Sanitizer.prototype.allowElement.call({}, null),
                () => DOMMatrix.prototype.rotateAxisAngleSelf
                  .call({}, null, null, null, null),
                () => MediaStream.prototype.addTrack.call({}, null),
                () => SVGSVGElement.prototype.checkEnclosure.call({}, null, null),
                () => WebSocket.prototype.close.call({}, null)
              ].every(illegal);
              const rejected = async (promise, message) => {
                if (!(promise instanceof Promise)) return false;
                try { await promise; return false; }
                catch (error) {
                  return error.name === "TypeError" && error.message === message;
                }
              };
              const element = document.createElement("div");
              document.body.appendChild(element);
              const validScrollPromises = [
                element.scroll(),
                element.scrollBy(),
                element.scrollIntoView(),
                element.scrollTo()
              ].every(value => value instanceof Promise);
              const ready = Object.getOwnPropertyDescriptor(
                ServiceWorkerContainer.prototype, "ready").get.call({});
              return synchronous &&
                validScrollPromises &&
                await rejected(
                  Blob.prototype.text.call({}),
                  "Failed to execute 'text' on 'Blob': Illegal invocation") &&
                await rejected(
                  Request.prototype.text.call({}),
                  "Failed to execute 'text' on 'Request': Illegal invocation") &&
                await rejected(
                  Response.prototype.json.call({}),
                  "Failed to execute 'json' on 'Response': Illegal invocation") &&
                await rejected(
                  Navigator.prototype.getBattery.call({}),
                  "Failed to execute 'getBattery' on 'Navigator': Illegal invocation") &&
                await rejected(
                  SubtleCrypto.prototype.digest.call({}),
                  "Failed to execute 'digest' on 'SubtleCrypto': Illegal invocation") &&
                await rejected(
                  USBDevice.prototype.open.call({}),
                  "Failed to execute 'open' on 'USBDevice': Illegal invocation") &&
                await rejected(
                  CredentialsContainer.prototype.get.call({}),
                  "Failed to execute 'get' on 'CredentialsContainer': Illegal invocation") &&
                await rejected(
                  Element.prototype.scroll.call({}),
                  "Failed to execute 'scroll' on 'Element': Illegal invocation") &&
                await rejected(
                  ready,
                  "Failed to read the 'ready' property from 'ServiceWorkerContainer': Illegal invocation") &&
                await rejected(
                  RTCPeerConnection.prototype.getStats.call({}),
                  "Failed to execute 'getStats' on 'RTCPeerConnection': Illegal invocation") &&
                await rejected(
                  SerialPort.prototype.forget.call({}),
                  "Failed to execute 'forget' on 'SerialPort': Illegal invocation");
            })()
            "#,
        ),
        "true"
    );
}
