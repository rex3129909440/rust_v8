use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

#[test]
fn structured_clone_preserves_graph_types_and_transfers_buffers_and_ports() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const cycle = { value: 7 };
              cycle.self = cycle;
              const key = { key: 1 };
              const date = new Date(1700000000000);
              const regexp = /edge/gi;
              const buffer = new Uint8Array([4, 5, 6]).buffer;
              const channel = new MessageChannel();
              const cloned = structuredClone(
                {
                  cycle,
                  map: new Map([[key, new Set([2, 3])]]),
                  date,
                  regexp,
                  buffer,
                  port: channel.port1
                },
                { transfer: [buffer, channel.port1] }
              );
              return [
                cloned.cycle === cloned.cycle.self,
                cloned.map instanceof Map,
                [...cloned.map.values()][0] instanceof Set,
                cloned.date instanceof Date,
                cloned.date.getTime(),
                cloned.regexp instanceof RegExp,
                cloned.regexp.source,
                cloned.regexp.flags,
                buffer.byteLength,
                cloned.buffer.byteLength,
                new Uint8Array(cloned.buffer)[1],
                cloned.port instanceof MessagePort
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "true|true|true|true|1700000000000|true|edge|gi|",
            "0|3|5|true"
        )
    );

    runtime
        .evaluate(
            r#"
            globalThis.portAnswer = "pending";
            const carrier = new MessageChannel();
            const moved = new MessageChannel();
            const payload = {};
            payload.self = payload;
            payload.port = moved.port1;
            carrier.port2.onmessage = event => {
              portAnswer = [
                event.data === event.data.self,
                event.data.port === event.ports[0],
                event.ports[0] instanceof MessagePort
              ].join("|");
            };
            carrier.port1.postMessage(payload, { transfer: [moved.port1] });
            "#,
        )
        .expect("queue port message");
    assert_eq!(text(&mut runtime, "portAnswer"), "true|true|true");
}

#[test]
fn window_post_message_targets_iframe_realm_and_reports_source_and_origin() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    runtime
        .evaluate(
            r#"
            globalThis.windowMessageAnswer = "pending";
            globalThis.messageFrame = document.createElement("iframe");
            messageFrame.srcdoc = `
              <script>
                onmessage = event => {
                  window.childMessageReceived = "yes";
                  parent.postMessage({
                    value: event.data.map.get("edge"),
                    sourceMatches: event.source === parent,
                    receivedOrigin: event.origin,
                    bufferValue: new Uint8Array(event.data.buffer)[0]
                  }, "*");
                };
              <\/script>
            `;
            document.body.appendChild(messageFrame);
            onmessage = event => {
              windowMessageAnswer = [
                event.data.value,
                event.data.sourceMatches,
                event.data.receivedOrigin,
                event.data.bufferValue,
                event.source === messageFrame.contentWindow,
                event.origin
              ].join("|");
            };
            const outgoingBuffer = new Uint8Array([23]).buffer;
            messageFrame.contentWindow.postMessage(
              {
                map: new Map([["edge", 41]]),
                buffer: outgoingBuffer
              },
              {
                targetOrigin: "https://sandbox.test",
                transfer: [outgoingBuffer]
              }
            );
            globalThis.detachedWindowMessageBufferLength = outgoingBuffer.byteLength;
            "#,
        )
        .expect("queue Window messages");
    assert_eq!(
        text(
            &mut runtime,
            "[
              windowMessageAnswer,
              detachedWindowMessageBufferLength,
              typeof messageFrame.contentWindow.onmessage,
              messageFrame.contentWindow.childMessageReceived
            ].join('~')",
        ),
        concat!(
            "41|true|https://sandbox.test|23|true|https://sandbox.test",
            "~0~function~yes"
        )
    );
}

#[test]
fn broadcast_channel_is_async_cloned_and_excludes_sender() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    runtime
        .evaluate(
            r#"
            globalThis.broadcastAnswer = [];
            const sender = new BroadcastChannel("edge-channel");
            const receiver = new BroadcastChannel("edge-channel");
            const unrelated = new BroadcastChannel("other-channel");
            sender.onmessage = () => broadcastAnswer.push("sender");
            unrelated.onmessage = () => broadcastAnswer.push("unrelated");
            receiver.onmessage = event => {
              broadcastAnswer.push([
                event.data === event.data.self,
                event.data.map instanceof Map,
                event.data.map.get("answer"),
                event.source === null,
                event.ports.length,
                event.origin
              ].join("|"));
            };
            const payload = { map: new Map([["answer", 42]]) };
            payload.self = payload;
            sender.postMessage(payload);
            payload.map.set("answer", 0);
            "#,
        )
        .expect("queue broadcast");
    assert_eq!(
        text(&mut runtime, "broadcastAnswer.join('~')"),
        "true|true|42|true|0|https://sandbox.test"
    );
}

#[test]
fn invalid_structured_clone_inputs_raise_data_clone_error() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const names = [];
              try { structuredClone(() => {}); } catch (error) { names.push(error.name); }
              const buffer = new ArrayBuffer(2);
              try {
                structuredClone(buffer, { transfer: [buffer, buffer] });
              } catch (error) {
                names.push(error.name);
              }
              const channel = new MessageChannel();
              try {
                structuredClone({ port: channel.port1 });
              } catch (error) {
                names.push(error.name);
              }
              try {
                structuredClone(navigator.plugins);
              } catch (error) {
                names.push(error.name + ":" + error.code);
              }
              try {
                postMessage(navigator.plugins, "*");
              } catch (error) {
                names.push(error.name + ":" + error.code);
              }
              try {
                structuredClone(document.createElement("div"));
              } catch (error) {
                names.push(error.name + ":" + error.code);
              }
              try {
                structuredClone(new URL("https://example.test/"));
              } catch (error) {
                names.push(error.name + ":" + error.code);
              }
              const fake = Object.create(PluginArray.prototype);
              fake.answer = 42;
              const clonedFake = structuredClone(fake);
              names.push(
                Object.getPrototypeOf(clonedFake) === Object.prototype &&
                clonedFake.answer === 42 ? "fake-ok" : "fake-bad"
              );
              return names.join("|");
            })()
            "#,
        ),
        concat!(
            "DataCloneError|DataCloneError|DataCloneError|",
            "DataCloneError:25|DataCloneError:25|",
            "DataCloneError:25|DataCloneError:25|fake-ok"
        )
    );
}
