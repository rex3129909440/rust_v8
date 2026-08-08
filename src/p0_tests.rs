use crate::{
    EdgeRuntime, EdgeRuntimeOptions, Evaluation, NetworkReplayEntry, PageInit,
    SpeechVoiceFingerprint,
};

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
    let source = r#"
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
    "#;
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
              Math.abs(timestamp - performance.now())
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

    let raf = text(&mut runtime, "edgeRafAnswer")
        .split(',')
        .map(|value| value.parse::<f64>().expect("RAF value"))
        .collect::<Vec<_>>();
    assert_eq!(raf.len(), 2);
    assert!(raf[0] >= 15.0, "RAF timestamp delta was {}", raf[0]);
    assert!(raf[1] <= 0.3, "RAF/performance skew was {}", raf[1]);
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
