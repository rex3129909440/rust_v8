use crate::{
    EdgeRuntime, EdgeRuntimeOptions, Evaluation, IframeHook, NetworkReplayEntry,
    NetworkRequestSource,
};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => value,
        value => value.to_string(),
    }
}

#[test]
fn private_v8_hook_binding_repeats_for_navigation_and_nested_iframes() {
    let hook = IframeHook::new(
        "xhr-open",
        r#"
        top.__hookBindingWasInWindow ||= "__edgev8" in window;
        const originalOpen = XMLHttpRequest.prototype.open;
        XMLHttpRequest.prototype.open = __edgev8.proxy(
          function open(method, url) {
            top.__hookUrls.push(String(url));
            return Reflect.apply(originalOpen, this, arguments);
          },
          "open"
        );
        "#,
    );
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        iframe_hooks: vec![hook],
        ..Default::default()
    })
    .expect("runtime with iframe hook");

    let result = text(
        &mut runtime,
        r#"
        (() => {
          window.__hookUrls = [];
          window.__hookBindingWasInWindow = false;
          const outer = document.createElement("iframe");
          outer.srcdoc = `<script>
            const first = new XMLHttpRequest();
            first.open("GET", "/first");
          <\/script>`;
          document.body.appendChild(outer);
          const inner = outer.contentDocument.createElement("iframe");
          inner.srcdoc = `<script>
            const nested = new XMLHttpRequest();
            nested.open("GET", "/nested");
          <\/script>`;
          outer.contentDocument.body.appendChild(inner);
          outer.srcdoc = `<script>
            const second = new XMLHttpRequest();
            second.open("GET", "/second");
            top.__pageBindingType = typeof window.__edgev8;
          <\/script>`;
          return [
            __hookUrls.join(","),
            __hookBindingWasInWindow,
            __pageBindingType,
            "__edgev8" in outer.contentWindow,
            Reflect.ownKeys(outer.contentWindow).includes("__edgev8"),
            Function.prototype.toString.call(
              outer.contentWindow.XMLHttpRequest.prototype.open
            )
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "/first,/nested,/second|false|undefined|false|false|function open() { [native code] }"
    );
    assert!(runtime.native_trace().is_empty());
}

#[test]
fn hook_protection_can_register_an_already_assigned_prototype_method() {
    let hook = IframeHook::new(
        "protect-existing",
        r#"
        const originalSend = XMLHttpRequest.prototype.send;
        XMLHttpRequest.prototype.send = function send() {
          return Reflect.apply(originalSend, this, arguments);
        };
        const replacement = XMLHttpRequest.prototype.send;
        const returned = __edgev8.protectPrototypeFunction(
          XMLHttpRequest.prototype,
          "send"
        );
        parent.__protectorReturnedSameFunction = returned === replacement;
        "#,
    );
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        iframe_hooks: vec![hook],
        ..Default::default()
    })
    .expect("runtime with iframe hook");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              return [
                __protectorReturnedSameFunction,
                Function.prototype.toString.call(
                  frame.contentWindow.XMLHttpRequest.prototype.send
                )
              ].join("|");
            })()
            "#,
        ),
        "true|function send() { [native code] }"
    );
}

#[test]
fn iframe_xhr_hook_runs_before_tl_send_without_trace() {
    let hook = IframeHook::new(
        "xhr-tl",
        r#"
        const originalOpen = XMLHttpRequest.prototype.open;
        const originalSend = XMLHttpRequest.prototype.send;
        const metadata = new WeakMap();
        XMLHttpRequest.prototype.open = __edgev8.proxy(
          function open(method, url) {
            metadata.set(this, [String(method), String(url)]);
            return Reflect.apply(originalOpen, this, arguments);
          },
          "open"
        );
        XMLHttpRequest.prototype.send = __edgev8.proxy(
          function send() {
            const body = arguments[0];
            const request = metadata.get(this);
            top.__tlHook = [request[0], request[1], String(body)].join("~");
            return Reflect.apply(originalSend, this, arguments);
          },
          "send"
        );
        "#,
    );
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        iframe_hooks: vec![hook],
        network_replay: vec![NetworkReplayEntry {
            url: "https://sandbox.test/tl".to_owned(),
            method: "POST".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: vec![("content-type".to_owned(), "text/plain".to_owned())],
            body: b"ok".to_vec(),
        }],
        ..Default::default()
    })
    .expect("runtime with iframe XHR hook");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              frame.srcdoc = `<script>
                const request = new XMLHttpRequest();
                request.open("POST", "/tl");
                request.send("payload");
              <\/script>`;
              document.body.appendChild(frame);
              return [
                __tlHook,
                frame.contentWindow.Function.prototype.toString.call(
                  frame.contentWindow.XMLHttpRequest.prototype.send
                ),
                "__edgev8" in frame.contentWindow
              ].join("|");
            })()
            "#,
        ),
        "POST~/tl~payload|function send() { [native code] }|false"
    );
    let requests = runtime.network_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].source, NetworkRequestSource::XmlHttpRequest);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].url, "https://sandbox.test/tl");
    assert_eq!(requests[0].body, b"payload");
    assert!(runtime.native_trace().is_empty());
}

#[test]
fn iframe_text_encoder_hook_exports_console_arguments_and_bytes_without_trace() {
    let hook = IframeHook::new(
        "text-encoder-stdout",
        r#"
        const originalEncode = TextEncoder.prototype.encode;
        TextEncoder.prototype.encode = __edgev8.proxy(
          function encode() {
            const output = Reflect.apply(originalEncode, this, arguments);
            console.log(
              "TextEncoder.prototype.encode",
              arguments,
              output,
              { input: arguments[0], byteLength: output.byteLength }
            );
            return output;
          },
          "encode"
        );
        "#,
    );
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        iframe_hooks: vec![hook],
        ..Default::default()
    })
    .expect("runtime with TextEncoder hook");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              frame.srcdoc = `<script>
                new TextEncoder().encode("中文-test");
              <\/script>`;
              document.body.appendChild(frame);
              return [
                Function.prototype.toString.call(
                  frame.contentWindow.TextEncoder.prototype.encode
                ),
                frame.contentWindow.console.log.name,
                frame.contentWindow.console.log.length
              ].join("|");
            })()
            "#,
        ),
        "function encode() { [native code] }|log|0"
    );
    assert!(runtime.native_trace().is_empty());

    let stdout = runtime.stdout();
    assert_eq!(stdout.len(), 1);
    assert_eq!(stdout[0].level, crate::ConsoleLevel::Log);
    assert_eq!(stdout[0].arguments.len(), 4);
    assert_eq!(
        stdout[0].arguments[0],
        crate::ConsoleValue::String {
            value: "TextEncoder.prototype.encode".to_owned(),
            truncated: false,
        }
    );
    assert_eq!(
        stdout[0].arguments[1],
        crate::ConsoleValue::Sequence {
            type_name: "Arguments".to_owned(),
            values: vec![crate::ConsoleValue::String {
                value: "中文-test".to_owned(),
                truncated: false,
            }],
            truncated: false,
        }
    );
    assert_eq!(
        stdout[0].arguments[2],
        crate::ConsoleValue::Bytes {
            type_name: "Uint8Array".to_owned(),
            value: "中文-test".as_bytes().to_vec(),
            truncated: false,
        }
    );
    assert_eq!(
        stdout[0].arguments[3],
        crate::ConsoleValue::Object {
            type_name: "Object".to_owned(),
            entries: vec![
                (
                    "input".to_owned(),
                    crate::ConsoleValue::String {
                        value: "中文-test".to_owned(),
                        truncated: false,
                    },
                ),
                ("byteLength".to_owned(), crate::ConsoleValue::Number(11.0)),
            ],
            truncated: false,
        }
    );

    runtime.clear_stdout();
    assert!(runtime.stdout().is_empty());
}

#[test]
fn stdout_preserves_native_error_lazy_stack_and_message() {
    let mut runtime = EdgeRuntime::new().expect("runtime");
    runtime
        .evaluate(r#"console.log(new TypeError("iterator failure"))"#)
        .expect("console evaluation");
    let stdout = runtime.stdout();
    assert_eq!(stdout.len(), 1);
    let crate::ConsoleValue::Object {
        type_name, entries, ..
    } = &stdout[0].arguments[0]
    else {
        panic!("native error was not captured as an object")
    };
    assert_eq!(type_name, "TypeError");
    let value = |name: &str| {
        entries
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    };
    assert!(
        matches!(
            value("message"),
            Some(crate::ConsoleValue::String { value, .. }) if value == "iterator failure"
        ),
        "captured error entries: {entries:#?}"
    );
    assert!(
        matches!(
            value("stack"),
            Some(crate::ConsoleValue::Other { type_name, display })
                if type_name == "Accessor" && display == "[accessor]"
        ),
        "captured error entries: {entries:#?}"
    );

    runtime
        .evaluate(r#"console.log(Symbol("edge"))"#)
        .expect("symbol console evaluation");
    let stdout = runtime.stdout();
    assert_eq!(stdout.len(), 2);
    assert_eq!(
        stdout[1].arguments[0],
        crate::ConsoleValue::Other {
            type_name: "Symbol".to_owned(),
            display: "Symbol(edge)".to_owned(),
        }
    );
}

#[test]
fn stdout_console_error_formatting_matches_edge_lazy_stack_behavior() {
    let mut runtime = EdgeRuntime::new().expect("runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const methods = [
                "debug", "info", "log", "warn", "error",
                "dir", "dirxml", "table", "trace"
              ];
              const hits = { name: 0, message: 0, stack: 0, extra: 0 };
              for (const method of methods) {
                const error = new Error(method);
                Object.defineProperties(error, {
                  name: {
                    configurable: true,
                    get() { hits.name += 1; return "ObservedError"; }
                  },
                  message: {
                    configurable: true,
                    get() { hits.message += 1; return method; }
                  },
                  stack: {
                    configurable: true,
                    get() { hits.stack += 1; return "user stack"; }
                  },
                  extra: {
                    configurable: true,
                    get() { hits.extra += 1; return "user value"; }
                  }
                });
                console[method](error);
              }

              const originalPrepare = Object.getOwnPropertyDescriptor(
                Error,
                "prepareStackTrace"
              );
              let prepareHits = 0;
              Error.prepareStackTrace = () => {
                prepareHits += 1;
                return "prepared by user";
              };
              for (const method of methods) {
                console[method](new Error(method));
              }
              if (originalPrepare) {
                Object.defineProperty(Error, "prepareStackTrace", originalPrepare);
              } else {
                delete Error.prepareStackTrace;
              }

              const nested = { name: 0, message: 0, stack: 0 };
              const nestedError = new Error("nested");
              Object.defineProperties(nestedError, {
                name: {
                  configurable: true,
                  get() { nested.name += 1; return "NestedError"; }
                },
                message: {
                  configurable: true,
                  get() { nested.message += 1; return "nested"; }
                },
                stack: {
                  configurable: true,
                  get() { nested.stack += 1; return "nested stack"; }
                }
              });
              console.log({ nested: nestedError });
              const objectHits = [nested.name, nested.message, nested.stack].join(",");
              console.log([nestedError]);
              const arrayHits = [nested.name, nested.message, nested.stack].join(",");

              return [
                hits.name,
                hits.message,
                hits.stack,
                hits.extra,
                prepareHits,
                objectHits,
                arrayHits
              ].join("|");
            })()
            "#,
        ),
        "9|9|0|0|0|0,0,0|1,1,0"
    );
}

#[test]
fn remaining_console_error_and_task_semantics_match_edge_without_devtools() {
    let mut runtime = EdgeRuntime::new().expect("runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const output = [];
              for (const method of [
                "count", "countReset", "time", "timeLog",
                "timeEnd", "timeStamp", "profile", "profileEnd"
              ]) {
                let calls = 0;
                console[method]({
                  toString() { calls += 1; return method + "-label"; }
                });
                let thrown;
                try {
                  console[method]({
                    toString() { throw new RangeError(method + " conversion"); }
                  });
                } catch (error) {
                  thrown = error.name + ":" + error.message;
                }
                let symbol;
                try {
                  console[method](Symbol(method));
                } catch (error) {
                  symbol = error.name + ":" + error.message;
                }
                output.push([method, calls, thrown, symbol].join("~"));
              }

              const errorHits = { group: 0, collapsed: 0, assertion: 0 };
              const throwingError = key => {
                const error = new Error(key);
                Object.defineProperty(error, "name", {
                  get() {
                    errorHits[key] += 1;
                    throw new RangeError(key + " name");
                  }
                });
                return error;
              };
              console.group(throwingError("group"));
              console.groupEnd();
              console.groupCollapsed(throwingError("collapsed"));
              console.groupEnd();
              const assertion = throwingError("assertion");
              console.assert(false, assertion);
              console.assert(true, assertion);
              output.push("errorHits~" + [
                errorHits.group,
                errorHits.collapsed,
                errorHits.assertion
              ].join(","));

              const invalidTasks = [undefined, null, "", 1, {}, Symbol("task")]
                .map(value => {
                  try {
                    console.createTask(value);
                    return "missing";
                  } catch (error) {
                    return error.name + ":" + error.message;
                  }
                });
              try {
                console.createTask();
                invalidTasks.unshift("missing");
              } catch (error) {
                invalidTasks.unshift(error.name + ":" + error.message);
              }
              output.push("invalidTasks~" + invalidTasks.join(","));

              const task = console.createTask("task name");
              const prototype = Object.getOwnPropertyDescriptor(
                task.run,
                "prototype"
              );
              output.push("taskShape~" + [
                Reflect.ownKeys(task).map(String).join(","),
                Object.getPrototypeOf(task) === Object.prototype,
                task.run.name,
                task.run.length,
                Object.hasOwn(task.run, "prototype"),
                prototype.writable,
                prototype.enumerable,
                prototype.configurable,
                Function.prototype.toString.call(task.run)
              ].join(","));

              const taskCall = callback => {
                try {
                  return callback();
                } catch (error) {
                  return error.name + ":" + error.message;
                }
              };
              const detached = task.run;
              output.push("taskCalls~" + [
                taskCall(() => task.run()),
                taskCall(() => task.run(1)),
                taskCall(() => detached(() => 1)),
                taskCall(() => new task.run()),
                taskCall(() => new task.run(() => ({ constructed: true }))),
                task.run(function callback() {
                  "use strict";
                  return this === undefined ? "strict-undefined" : "other";
                })
              ].join(","));
              return output.join("|");
            })()
            "#,
        ),
        concat!(
            "count~1~RangeError:count conversion~TypeError:Cannot convert a Symbol value to a string|",
            "countReset~1~RangeError:countReset conversion~TypeError:Cannot convert a Symbol value to a string|",
            "time~1~RangeError:time conversion~TypeError:Cannot convert a Symbol value to a string|",
            "timeLog~1~RangeError:timeLog conversion~TypeError:Cannot convert a Symbol value to a string|",
            "timeEnd~1~RangeError:timeEnd conversion~TypeError:Cannot convert a Symbol value to a string|",
            "timeStamp~1~RangeError:timeStamp conversion~TypeError:Cannot convert a Symbol value to a string|",
            "profile~1~RangeError:profile conversion~TypeError:Cannot convert a Symbol value to a string|",
            "profileEnd~1~RangeError:profileEnd conversion~TypeError:Cannot convert a Symbol value to a string|",
            "errorHits~1,1,1|",
            "invalidTasks~Error:First argument must be a non-empty string.,",
            "Error:First argument must be a non-empty string.,",
            "Error:First argument must be a non-empty string.,",
            "Error:First argument must be a non-empty string.,",
            "Error:First argument must be a non-empty string.,",
            "Error:First argument must be a non-empty string.,",
            "Error:First argument must be a non-empty string.|",
            "taskShape~run,true,run,0,true,true,false,false,function run() { [native code] }|",
            "taskCalls~Error:First argument must be a function.,",
            "Error:First argument must be a function.,",
            "Error:'run' called with illegal receiver.,",
            "Error:First argument must be a function.,",
            "Error:'run' called with illegal receiver.,strict-undefined"
        )
    );
}
