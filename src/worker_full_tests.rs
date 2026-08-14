use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => value,
        value => value.to_string(),
    }
}

#[test]
fn dedicated_worker_executes_in_an_isolated_worker_realm() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let scheduled = text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            postMessage([
              self === globalThis,
              typeof window,
              typeof document,
              self instanceof DedicatedWorkerGlobalScope,
              Object.getPrototypeOf(DedicatedWorkerGlobalScope.prototype) ===
                WorkerGlobalScope.prototype,
              Object.getPrototypeOf(WorkerGlobalScope.prototype) ===
                EventTarget.prototype,
              Object.prototype.toString.call(self),
              Object.prototype.toString.call(location),
              Object.prototype.toString.call(navigator),
              location.href.startsWith("data:"),
              navigator.userAgent.includes("Chrome/150") &&
                !navigator.userAgent.includes("Edg/") &&
                !navigator.userAgent.includes("HeadlessChrome/"),
              navigator.languages.join(","),
              typeof importScripts,
              typeof Worker,
              typeof SharedWorker,
              typeof MessageChannel
            ]);
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source),
            { name: "realm-name" }
          );
          globalThis.workerRealmAnswer = "pending";
          worker.onerror = event =>
            globalThis.workerRealmAnswer = "ERR:" + event.message;
          worker.onmessage = event =>
            globalThis.workerRealmAnswer = event.data.join("|");
          return "scheduled";
        })()
        "#,
    );
    assert_eq!(scheduled, "scheduled");
    let result = text(&mut runtime, "workerRealmAnswer");
    assert_eq!(
        result,
        "true|undefined|undefined|true|true|true|[object DedicatedWorkerGlobalScope]|[object WorkerLocation]|[object WorkerNavigator]|true|true|zh-CN,en,en-GB,en-US|function|function|undefined|function"
    );
}

#[test]
fn worker_global_algorithms_create_values_in_the_worker_realm_and_trace_calls() {
    let setup = r#"
        (() => {
          const source = `
            (async () => {
              const cloned = structuredClone({
                list: [1, 2],
                map: new Map([["edge", 150]])
              });
              const bitmapPromise = createImageBitmap(
                new ImageData(
                  new Uint8ClampedArray([10, 20, 30, 255]),
                  1,
                  1
                )
              );
              const bitmap = await bitmapPromise;
              const fetchPromise = fetch(
                "data:text/plain;base64,ZWRnZQ=="
              );
              const response = await fetchPromise;
              const body = await response.text();
              const fileSystemResult = await new Promise(resolve => {
                webkitRequestFileSystem(
                  TEMPORARY,
                  1024,
                  fileSystem => {
                    const rootUrl = fileSystem.root.toURL();
                    webkitResolveLocalFileSystemURL(
                      rootUrl,
                      entry => resolve([
                        Object.prototype.toString.call(fileSystem),
                        Object.prototype.toString.call(fileSystem.root),
                        entry === fileSystem.root,
                        fileSystem.root.filesystem === fileSystem
                      ].join(":")),
                      error => resolve("resolve-error:" + error.name)
                    );
                  },
                  error => resolve("request-error:" + error.name)
                );
              });
              const reportedError = new Promise(resolve => {
                addEventListener(
                  "error",
                  event => resolve(
                    event.error instanceof Error &&
                    event.message === "Uncaught Error: worker-local"
                  ),
                  { once: true }
                );
              });
              reportError(new Error("worker-local"));
              postMessage([
                origin,
                atob("ZWRnZQ=="),
                btoa("edge"),
                Object.getPrototypeOf(cloned) === Object.prototype,
                cloned.list instanceof Array,
                cloned.map instanceof Map,
                cloned.map.get("edge"),
                bitmapPromise instanceof Promise,
                bitmap instanceof ImageBitmap,
                bitmap.width,
                bitmap.height,
                fetchPromise instanceof Promise,
                response instanceof Response,
                body,
                fileSystemResult,
                await reportedError,
                Function.prototype.toString.call(fetch),
                Function.prototype.toString.call(structuredClone),
                Function.prototype.toString.call(createImageBitmap),
                Function.prototype.toString.call(
                  webkitRequestFileSystem
                )
              ].join("|"));
            })().catch(error => postMessage(
              "ERR:" + error.name + ":" + error.message
            ));
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerGlobalAlgorithmAnswer = "pending";
          worker.onerror = event =>
            globalThis.workerGlobalAlgorithmAnswer =
              "WORKER-ERR:" + event.message;
          worker.onmessage = event => {
            globalThis.workerGlobalAlgorithmAnswer = event.data;
            worker.terminate();
          };
          return "scheduled";
        })()
    "#;
    let expected = concat!(
        "https://sandbox.test|edge|ZWRnZQ==|true|true|true|150|",
        "true|true|1|1|true|true|edge|",
        "[object DOMFileSystem]:[object DirectoryEntry]:true:true|true|",
        "function fetch() { [native code] }|",
        "function structuredClone() { [native code] }|",
        "function createImageBitmap() { [native code] }|",
        "function webkitRequestFileSystem() { [native code] }"
    );

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    text(&mut direct, setup);
    assert_eq!(text(&mut direct, "workerGlobalAlgorithmAnswer"), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    text(&mut traced, setup);
    let traced_result = text(&mut traced, "workerGlobalAlgorithmAnswer");
    let trace = traced.proxy_trace();
    assert_eq!(traced_result, expected, "{trace:#?}");
    for suffix in [
        ".atob",
        ".btoa",
        ".structuredClone",
        ".createImageBitmap",
        ".fetch",
        ".webkitRequestFileSystem",
        ".webkitResolveLocalFileSystemURL",
        ".reportError",
    ] {
        assert!(
            trace.iter().any(|entry| {
                entry.operation == "call"
                    && entry.api.starts_with("worker[")
                    && entry.api.ends_with(suffix)
            }),
            "missing Worker trace for {suffix}: {trace:#?}"
        );
    }
}

#[test]
fn worker_encoding_stream_xhr_and_performance_use_local_realm_implementations() {
    let setup = r#"
        (() => {
          const source = `
            const constructors = [
              TextEncoder,
              TextDecoder,
              TextEncoderStream,
              TextDecoderStream,
              ReadableStream,
              ReadableStreamDefaultReader,
              ReadableStreamDefaultController,
              ReadableStreamBYOBRequest,
              ReadableStreamBYOBReader,
              ReadableByteStreamController,
              WritableStream,
              WritableStreamDefaultWriter,
              WritableStreamDefaultController,
              TransformStream,
              TransformStreamDefaultController,
              CompressionStream,
              DecompressionStream,
              ByteLengthQueuingStrategy,
              CountQueuingStrategy,
              XMLHttpRequest,
              XMLHttpRequestEventTarget,
              XMLHttpRequestUpload,
              PerformanceEntry,
              PerformanceMark,
              PerformanceMeasure,
              PerformanceServerTiming,
              PerformanceResourceTiming,
              PerformanceObserverEntryList,
              PerformanceObserver
            ];
            const localFunctionChains = constructors.every(Constructor => {
              let cursor = Constructor;
              while (cursor !== null) {
                if (cursor === Function.prototype) return true;
                cursor = Object.getPrototypeOf(cursor);
              }
              return false;
            });
            const encoded = new TextEncoder().encode("edge");
            const decoded = new TextDecoder().decode(encoded);
            const readable = new ReadableStream();
            const writable = new WritableStream();
            const transform = new TransformStream();
            const compressed = new CompressionStream("gzip");
            const request = new XMLHttpRequest();
            request.open("GET", "https://sandbox.test/worker-xhr");
            const mark = new PerformanceMark("worker-mark", {
              startTime: 2,
              detail: "worker-detail"
            });
            postMessage([
              localFunctionChains,
              decoded,
              readable instanceof ReadableStream,
              writable instanceof WritableStream,
              transform.readable instanceof ReadableStream,
              transform.writable instanceof WritableStream,
              compressed.readable instanceof ReadableStream,
              compressed.writable instanceof WritableStream,
              request instanceof XMLHttpRequest,
              request instanceof XMLHttpRequestEventTarget,
              request.readyState,
              mark instanceof PerformanceMark,
              mark instanceof PerformanceEntry,
              mark.name,
              mark.entryType,
              mark.startTime,
              mark.detail,
              Object.getPrototypeOf(PerformanceMark.prototype) ===
                PerformanceEntry.prototype,
              Function.prototype.toString.call(
                TextEncoder.prototype.encode
              )
            ].join("|"));
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerLocalApiAnswer = "pending";
          worker.onerror = event =>
            globalThis.workerLocalApiAnswer = "ERR:" + event.message;
          worker.onmessage = event => {
            globalThis.workerLocalApiAnswer = event.data;
            worker.terminate();
          };
        })()
    "#;
    let expected = concat!(
        "true|edge|true|true|true|true|true|true|true|true|1|",
        "true|true|worker-mark|mark|2|worker-detail|true|",
        "function encode() { [native code] }"
    );

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    text(&mut direct, setup);
    assert_eq!(text(&mut direct, "workerLocalApiAnswer"), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    text(&mut traced, setup);
    assert_eq!(text(&mut traced, "workerLocalApiAnswer"), expected);
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("worker[")
            && entry.api.ends_with(".TextEncoder().encode")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("worker[")
            && entry.api.ends_with(".XMLHttpRequest().open")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.contains("worker[")
            && entry.api.ends_with(".PerformanceMark().name")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.contains("worker[")
            && entry.api.ends_with(".CompressionStream().readable")
    }));
}

#[test]
fn worker_navigator_services_and_fonts_are_worker_realm_objects() {
    let setup = r#"
        (() => {
          const source = `
            const face = new FontFace("WorkerFace", "url(worker.woff2)");
            fonts.add(face);
            const fontPrototype = Object.getPrototypeOf(fonts);
            const fontConstructor = fontPrototype.constructor;
            postMessage([
              navigator.connection instanceof NetworkInformation,
              navigator.gpu instanceof GPU,
              navigator.hid instanceof HID,
              navigator.locks instanceof LockManager,
              navigator.mediaCapabilities instanceof MediaCapabilities,
              navigator.permissions instanceof Permissions,
              navigator.serial instanceof Serial,
              navigator.storageBuckets instanceof StorageBucketManager,
              navigator.storage instanceof StorageManager,
              navigator.usb instanceof USB,
              navigator.userAgentData instanceof NavigatorUAData,
              navigator.connection === navigator.connection,
              navigator.storage === navigator.storage,
              fonts === fonts,
              fonts.has(face),
              fonts.size,
              fontConstructor.name,
              typeof FontFaceSet,
              !Object.hasOwn(fontPrototype, "constructor"),
              Object.getPrototypeOf(fontPrototype) === EventTarget.prototype,
              Object.getPrototypeOf(fonts) === fontPrototype,
              fonts instanceof EventTarget,
              Function.prototype.toString.call(fonts.add)
            ].join("|"));
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerNavigatorRealmAnswer = "pending";
          worker.onerror = event =>
            globalThis.workerNavigatorRealmAnswer = "ERR:" + event.message;
          worker.onmessage = event => {
            globalThis.workerNavigatorRealmAnswer = event.data;
            worker.terminate();
          };
        })()
    "#;
    let expected = concat!(
        "true|true|true|true|true|true|true|true|true|true|true|",
        "true|true|true|true|1|EventTarget|function|true|true|true|true|",
        "function add() { [native code] }"
    );

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    text(&mut direct, setup);
    assert_eq!(text(&mut direct, "workerNavigatorRealmAnswer"), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    text(&mut traced, setup);
    let traced_result = text(&mut traced, "workerNavigatorRealmAnswer");
    let trace = traced.proxy_trace();
    assert_eq!(traced_result, expected, "{trace:#?}");
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.starts_with("worker[")
            && entry.api.ends_with(".navigator.storage")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("worker[")
            && entry.api.ends_with(".fonts.add")
    }));
}

#[test]
fn worker_rendering_factories_return_local_realm_resource_types() {
    let setup = r#"
        (() => {
          const source = `
            const canvas = new OffscreenCanvas(4, 4);
            const gl = canvas.getContext("webgl2");
            const canvas2d = new OffscreenCanvas(4, 4);
            const context2d = canvas2d.getContext("2d");
            const buffer = gl.createBuffer();
            const texture = gl.createTexture();
            const program = gl.createProgram();
            const shader = gl.createShader(35633);
            const framebuffer = gl.createFramebuffer();
            const renderbuffer = gl.createRenderbuffer();
            const query = gl.createQuery();
            const sampler = gl.createSampler();
            const feedback = gl.createTransformFeedback();
            const vertexArray = gl.createVertexArray();
            const gradient = context2d.createLinearGradient(0, 0, 1, 1);
            const pattern = context2d.createPattern(canvas2d, "repeat");
            const matrix = new DOMMatrix();
            const point = new DOMPoint(1, 2, 3, 4);
            const rect = new DOMRect(1, 2, 3, 4);
            const imageData = new ImageData(2, 2);
            const path = new Path2D();
            const chunk = new EncodedVideoChunk({
              type: "key",
              timestamp: 1,
              data: new Uint8Array([1, 2, 3])
            });
            postMessage([
              buffer instanceof WebGLBuffer,
              texture instanceof WebGLTexture,
              program instanceof WebGLProgram,
              shader instanceof WebGLShader,
              framebuffer instanceof WebGLFramebuffer,
              renderbuffer instanceof WebGLRenderbuffer,
              query instanceof WebGLQuery,
              sampler instanceof WebGLSampler,
              feedback instanceof WebGLTransformFeedback,
              vertexArray instanceof WebGLVertexArrayObject,
              gradient instanceof CanvasGradient,
              pattern instanceof CanvasPattern,
              matrix instanceof DOMMatrix,
              matrix instanceof DOMMatrixReadOnly,
              point instanceof DOMPoint,
              point instanceof DOMPointReadOnly,
              rect instanceof DOMRect,
              rect instanceof DOMRectReadOnly,
              imageData instanceof ImageData,
              path instanceof Path2D,
              chunk instanceof EncodedVideoChunk,
              chunk.byteLength,
              Object.getPrototypeOf(WebGLBuffer) === WebGLObject,
              Object.getPrototypeOf(DOMMatrix) === DOMMatrixReadOnly,
              Function.prototype.toString.call(
                WebGL2RenderingContext.prototype.createBuffer
              )
            ].join("|"));
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerRenderingAnswer = "pending";
          worker.onerror = event =>
            globalThis.workerRenderingAnswer = "ERR:" + event.message;
          worker.onmessage = event => {
            globalThis.workerRenderingAnswer = event.data;
            worker.terminate();
          };
        })()
    "#;
    let expected = concat!(
        "true|true|true|true|true|true|true|true|true|true|",
        "true|true|true|true|true|true|true|true|true|true|",
        "true|3|true|true|function createBuffer() { [native code] }"
    );

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    text(&mut direct, setup);
    assert_eq!(text(&mut direct, "workerRenderingAnswer"), expected);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    text(&mut traced, setup);
    assert_eq!(text(&mut traced, "workerRenderingAnswer"), expected);
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("worker[")
            && entry.api.ends_with(".createBuffer")
            && entry.result.contains("WebGLBuffer")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "construct"
            && entry.api.starts_with("worker[")
            && entry.api.ends_with(".DOMMatrix")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "construct"
            && entry.api.starts_with("worker[")
            && entry.api.ends_with(".EncodedVideoChunk")
    }));
}

#[test]
fn dedicated_worker_messages_use_structured_clone_and_transfer() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            onmessage = event => {
              event.data.self = event.data;
              postMessage({
                nested: event.data.nested,
                cycle: event.data.self === event.data,
                byte: event.data.bytes[1]
              });
            };
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          const buffer = new ArrayBuffer(3);
          new Uint8Array(buffer).set([4, 7, 9]);
          globalThis.workerCloneAnswer = "pending";
          worker.onmessage = event => {
            globalThis.workerCloneAnswer = [
              event.data.nested.value,
              event.data.cycle,
              event.data.byte,
              buffer.byteLength
            ].join("|");
          };
          worker.postMessage(
            { nested: { value: 42 }, bytes: new Uint8Array(buffer) },
            [buffer]
          );
          return "scheduled";
        })()
        "#,
    );
    let result = text(&mut runtime, "workerCloneAnswer");
    assert_eq!(result, "42|true|7|0");
}

#[test]
fn worker_import_scripts_timers_errors_and_termination_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const imported =
            "data:text/javascript," +
            encodeURIComponent("self.importedValue = 9;");
          const source =
            "importScripts('" + imported + "');" +
            "onmessage=event=>{" +
            "setTimeout(()=>postMessage(importedValue+event.data),0)}";
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerImportAnswer = "pending";
          worker.onmessage = event =>
            globalThis.workerImportAnswer = String(event.data);
          worker.postMessage(3);
          return "scheduled";
        })()
        "#,
    );
    let result = text(&mut runtime, "workerImportAnswer");
    assert_eq!(result, "12");

    let terminated = text(
        &mut runtime,
        r#"
        (() => {
          const worker = new Worker(
            "data:text/javascript," +
              encodeURIComponent("onmessage=e=>postMessage(e.data)")
          );
          let called = false;
          worker.onmessage = () => called = true;
          worker.terminate();
          worker.postMessage("ignored");
          return called;
        })()
        "#,
    );
    assert_eq!(terminated, "false");

    text(
        &mut runtime,
        r#"
        (() => {
          const worker = new Worker(
            "data:text/javascript," +
              encodeURIComponent("throw new Error('worker boom')")
          );
          globalThis.workerErrorAnswer = "pending";
          worker.onerror = event =>
            globalThis.workerErrorAnswer = [
              event.message,
              event.filename.startsWith("data:text/javascript,"),
              event.lineno > 0,
              event.colno > 0,
              event.error instanceof Error
            ].join("|");
          return "scheduled";
        })()
        "#,
    );
    let error = text(&mut runtime, "workerErrorAnswer");
    assert!(error.contains("worker boom"), "{error}");
    assert!(error.ends_with("|true|true|true|true"), "{error}");
}

#[test]
fn blob_module_and_shared_workers_execute_real_scripts() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const url = URL.createObjectURL(
            new Blob(["postMessage(21 * 2)"], { type: "text/javascript" })
          );
          const worker = new Worker(url);
          globalThis.blobWorkerAnswer = "pending";
          worker.onmessage = event =>
            globalThis.blobWorkerAnswer = String(event.data);
          URL.revokeObjectURL(url);
          return "scheduled";
        })()
        "#,
    );
    let blob_worker = text(&mut runtime, "blobWorkerAnswer");
    assert_eq!(blob_worker, "42");

    text(
        &mut runtime,
        r#"
        (() => {
          const dependency =
            "data:text/javascript," +
            encodeURIComponent("export const value = 40");
          const source =
            "import { value } from '" + dependency + "';" +
            "postMessage(value + 2);";
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source),
            { type: "module" }
          );
          globalThis.moduleWorkerAnswer = "pending";
          worker.onmessage = event =>
            globalThis.moduleWorkerAnswer = String(event.data);
          return "scheduled";
        })()
        "#,
    );
    let module_worker = text(&mut runtime, "moduleWorkerAnswer");
    assert_eq!(module_worker, "42");

    text(
        &mut runtime,
        r#"
        (() => {
          const dependency =
            "data:text/javascript," +
            encodeURIComponent("export const value = 41");
          const source =
            "import('" + dependency + "')" +
            ".then(module => postMessage(module.value + 1));";
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source),
            { type: "module" }
          );
          globalThis.dynamicModuleWorkerAnswer = "pending";
          worker.onmessage = event =>
            globalThis.dynamicModuleWorkerAnswer = String(event.data);
          return "scheduled";
        })()
        "#,
    );
    let dynamic_module_worker = text(&mut runtime, "dynamicModuleWorkerAnswer");
    assert_eq!(dynamic_module_worker, "42");

    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            let connections = 0;
            onconnect = event => {
              const port = event.ports[0];
              connections += 1;
              port.onmessage = message =>
                port.postMessage(connections + ":" + message.data);
            };
          `;
          const url = "data:text/javascript," + encodeURIComponent(source);
          const first = new SharedWorker(url, { name: "shared-edge" });
          const second = new SharedWorker(url, { name: "shared-edge" });
          globalThis.sharedLeft = "pending";
          globalThis.sharedRight = "pending";
          first.port.onmessage = event =>
            globalThis.sharedLeft = event.data;
          second.port.onmessage = event =>
            globalThis.sharedRight = event.data;
          first.port.postMessage("a");
          second.port.postMessage("b");
          globalThis.sharedPortShape = first.port instanceof MessagePort;
          return "scheduled";
        })()
        "#,
    );
    let shared = text(
        &mut runtime,
        "[sharedLeft, sharedRight, sharedPortShape].join('|')",
    );
    assert_eq!(shared, "2:a|2:b|true");
}

#[test]
fn proxy_trace_records_worker_realm_apis_without_changing_shapes() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let baseline = text(
        &mut runtime,
        r#"
        [
          Function.prototype.toString.call(Worker),
          Function.prototype.toString.call(Worker.prototype.postMessage),
          Object.getOwnPropertyNames(Worker.prototype).join(","),
          Object.getOwnPropertyNames(window).length
        ].join("|")
        "#,
    );
    runtime.enable_proxy_trace().expect("enable trace");
    text(
        &mut runtime,
        r#"
        (() => {
          const worker = new Worker(
            "data:text/javascript," +
              encodeURIComponent(
                "const agent=navigator.userAgent;" +
                "postMessage([" +
                "location.href," +
                "agent," +
                "Function.prototype.toString.call(" +
                "MessagePort.prototype.postMessage)," +
                "Object.getOwnPropertyNames(globalThis).length" +
                "].join('|'))"
              )
          );
          globalThis.workerTraceAnswer = "pending";
          worker.onerror = event =>
            globalThis.workerTraceAnswer = "ERR:" + event.message;
          worker.onmessage = event =>
            globalThis.workerTraceAnswer = event.data;
          return "scheduled";
        })()
        "#,
    );
    let answer = text(&mut runtime, "workerTraceAnswer");
    assert!(answer.contains("Chrome/150"), "{answer}");
    assert!(!answer.contains("Edg/"), "{answer}");
    assert!(!answer.contains("HeadlessChrome/"), "{answer}");
    assert!(answer.contains("[native code]"), "{answer}");
    assert!(answer.ends_with("|334"), "{answer}");
    text(
        &mut runtime,
        r#"
        (() => {
          const worker = new Worker(
            "data:text/javascript," +
              encodeURIComponent(
                "postMessage(location.href + '|' + navigator.userAgent)"
              ),
            { type: "module" }
          );
          globalThis.moduleWorkerTraceAnswer = "pending";
          worker.onmessage = event =>
            globalThis.moduleWorkerTraceAnswer = event.data;
          return "scheduled";
        })()
        "#,
    );
    let module_answer = text(&mut runtime, "moduleWorkerTraceAnswer");
    assert!(module_answer.contains("Chrome/150"), "{module_answer}");
    assert!(!module_answer.contains("Edg/"), "{module_answer}");
    assert!(
        !module_answer.contains("HeadlessChrome/"),
        "{module_answer}"
    );
    text(
        &mut runtime,
        r#"
        (() => {
          const dependency =
            "data:text/javascript," +
            encodeURIComponent(
              "export const agent = navigator.userAgent"
            );
          const source =
            "import('" + dependency + "')" +
            ".then(module => postMessage(" +
            "location.href + '|' + module.agent))";
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source),
            { type: "module" }
          );
          globalThis.dynamicModuleWorkerTraceAnswer = "pending";
          worker.onmessage = event =>
            globalThis.dynamicModuleWorkerTraceAnswer = event.data;
          return "scheduled";
        })()
        "#,
    );
    let dynamic_module_answer = text(&mut runtime, "dynamicModuleWorkerTraceAnswer");
    assert!(
        dynamic_module_answer.contains("Chrome/150"),
        "{dynamic_module_answer}"
    );
    assert!(!dynamic_module_answer.contains("Edg/"));
    assert!(!dynamic_module_answer.contains("HeadlessChrome/"));
    text(
        &mut runtime,
        r#"
        (() => {
          const worker = new Worker(
            "data:text/javascript," +
              encodeURIComponent(
                "const channel=new MessageChannel();" +
                "channel.port1.onmessage=event=>postMessage(event.data);" +
                "channel.port2.postMessage('message-port-trace')"
              )
          );
          globalThis.messagePortTraceAnswer = "pending";
          worker.onmessage = event =>
            globalThis.messagePortTraceAnswer = event.data;
          return "scheduled";
        })()
        "#,
    );
    assert_eq!(
        text(&mut runtime, "messagePortTraceAnswer"),
        "message-port-trace"
    );
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            skipWaiting();
            onmessage = event => {
              clients.matchAll().then(values =>
                values[0].postMessage(registration.scope)
              );
            };
          `;
          globalThis.serviceWorkerProxyTraceAnswer = "pending";
          navigator.serviceWorker.onmessage = event =>
            globalThis.serviceWorkerProxyTraceAnswer = event.data;
          navigator.serviceWorker.register(
            "data:text/javascript," + encodeURIComponent(source)
          ).then(registration => registration.active.postMessage("trace"));
          return "scheduled";
        })()
        "#,
    );
    let service_answer = text(&mut runtime, "serviceWorkerProxyTraceAnswer");
    assert_eq!(service_answer, "https://sandbox.test/");
    let trace = runtime.proxy_trace();
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "get" && entry.api.ends_with(".navigator") })
    );
    assert!(
        trace.iter().any(|entry| {
            entry.operation == "get" && entry.api.ends_with(".navigator.userAgent")
        })
    );
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with(".postMessage") })
    );
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.starts_with("worker[")
            && entry.api.ends_with(".location")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.starts_with("worker[2]")
            && entry.api.ends_with(".navigator")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.starts_with("worker[2]")
            && entry.api.ends_with(".navigator.userAgent")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.starts_with("worker[2]")
            && entry.api.ends_with(".postMessage")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.starts_with("worker[3]")
            && entry.api.ends_with(".navigator.userAgent")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.starts_with("worker[3]")
            && entry.api.ends_with(".postMessage")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.starts_with("serviceWorker[")
            && entry.api.ends_with(".clients.matchAll")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.starts_with("serviceWorker[")
            && entry.api.ends_with(".skipWaiting")
    }));
    let message_trace = trace
        .iter()
        .filter(|entry| {
            entry.api.contains("MessageChannel")
                || entry.api.contains("port1")
                || entry.api.contains("port2")
                || (entry.operation == "call" && entry.api.ends_with(".postMessage"))
        })
        .collect::<Vec<_>>();
    assert!(
        trace.iter().any(|entry| {
            entry.operation == "get"
                && entry.api.contains("worker[")
                && entry.api.ends_with(".port1")
        }),
        "{message_trace:?}"
    );
    assert!(
        trace.iter().any(|entry| {
            entry.operation == "call"
                && entry.api.ends_with(".port2.postMessage")
                && entry.receiver.contains("worker[")
                && entry.receiver.ends_with(".port2")
        }),
        "{message_trace:?}"
    );
    runtime.disable_proxy_trace();
    runtime.clear_proxy_trace();
    text(
        &mut runtime,
        "delete globalThis.workerTraceAnswer; delete globalThis.moduleWorkerTraceAnswer; delete globalThis.dynamicModuleWorkerTraceAnswer; delete globalThis.messagePortTraceAnswer; delete globalThis.serviceWorkerProxyTraceAnswer",
    );
    let after = text(
        &mut runtime,
        r#"
        [
          Function.prototype.toString.call(Worker),
          Function.prototype.toString.call(Worker.prototype.postMessage),
          Object.getOwnPropertyNames(Worker.prototype).join(","),
          Object.getOwnPropertyNames(window).length
        ].join("|")
        "#,
    );
    assert_eq!(after, baseline);
    assert!(runtime.proxy_trace().is_empty());
}

#[test]
fn service_worker_lifecycle_clients_and_messages_are_functional() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let initial = text(&mut runtime, "navigator.serviceWorker.controller === null");
    assert_eq!(initial, "true");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            let installed = false;
            let activated = false;
            let installExtendable = false;
            let activateExtendable = false;
            oninstall = event => {
              installed = true;
              installExtendable =
                event instanceof ExtendableEvent &&
                typeof event.waitUntil === "function";
              event.waitUntil(Promise.resolve("installed"));
            };
            onactivate = event => {
              activated = true;
              activateExtendable =
                event instanceof ExtendableEvent &&
                typeof event.waitUntil === "function";
              event.waitUntil(Promise.resolve("activated"));
            };
            onfetch = event => {
              event.respondWith(Promise.resolve(new Response([
                event instanceof FetchEvent,
                event instanceof ExtendableEvent,
                event.request instanceof Request,
                event.request.url,
                event.preloadResponse instanceof Promise,
                event.handled instanceof Promise,
                event.clientId,
                event.resultingClientId,
                event.replacesClientId
              ].join("|"))));
            };
            onmessage = event => {
              try {
                const registrationConstructor =
                  Object.getPrototypeOf(registration).constructor;
                const serviceWorkerConstructor =
                  Object.getPrototypeOf(serviceWorker).constructor;
                const navigationPreloadConstructor =
                  Object.getPrototypeOf(
                    registration.navigationPreload
                  ).constructor;
                event.source.postMessage({
                value: event.data.value + 1,
                installed,
                activated,
                installExtendable,
                activateExtendable,
                serviceShape: self instanceof ServiceWorkerGlobalScope,
                clientShape: event.source instanceof WindowClient,
                clientType: event.source.type,
                scope: registration.scope,
                hasClients: typeof clients.matchAll === "function",
                registrationShape:
                  registration instanceof registrationConstructor,
                serviceWorkerShape:
                  serviceWorker instanceof serviceWorkerConstructor,
                activeIdentity: registration.active === serviceWorker,
                localActiveState: registration.active.state,
                registrationPrototype:
                  Object.getPrototypeOf(registration) ===
                    registrationConstructor.prototype,
                serviceWorkerPrototype:
                  Object.getPrototypeOf(serviceWorker) ===
                    serviceWorkerConstructor.prototype,
                navigationPreloadShape:
                  registration.navigationPreload instanceof
                    navigationPreloadConstructor,
                localNativeShape:
                  Function.prototype.toString.call(
                      serviceWorkerConstructor.prototype.postMessage
                    )
                });
              } catch (error) {
                event.source.postMessage({
                  failure: error.name + ":" + error.message
                });
              }
            };
          `;
          const url = "data:text/javascript," + encodeURIComponent(source);
          globalThis.serviceWorkerAnswer = "pending";
          globalThis.serviceWorkerFetchAnswer = "pending";
          globalThis.serviceWorkerControllerChanges = 0;
          navigator.serviceWorker.oncontrollerchange = () =>
            serviceWorkerControllerChanges += 1;
          navigator.serviceWorker.onmessage = event => {
            const value = event.data;
            if (value.failure) {
              serviceWorkerAnswer = "ERR:" + value.failure;
              return;
            }
            serviceWorkerAnswer = [
              value.value,
              value.installed,
              value.activated,
              value.installExtendable,
              value.activateExtendable,
              value.serviceShape,
              value.clientShape,
              value.clientType,
              value.scope,
              value.hasClients,
              value.registrationShape,
              value.serviceWorkerShape,
              value.activeIdentity,
              value.localActiveState,
              value.registrationPrototype,
              value.serviceWorkerPrototype,
              value.navigationPreloadShape,
              value.localNativeShape
            ].join("|");
          };
          navigator.serviceWorker.register(url, {
            scope: "https://sandbox.test/app/",
            updateViaCache: "none"
          }).then(registration => {
            globalThis.serviceRegistrationShape = [
              registration.active instanceof ServiceWorker,
              registration.installing === null,
              registration.waiting === null,
              registration.active.state,
              registration.scope,
              registration.updateViaCache,
              registration.navigationPreload instanceof NavigationPreloadManager,
              registration.backgroundFetch instanceof BackgroundFetchManager,
              registration.periodicSync instanceof PeriodicSyncManager,
              registration.sync instanceof SyncManager,
              registration.cookies instanceof CookieStoreManager,
              registration.pushManager instanceof PushManager,
              registration.paymentManager instanceof PaymentManager,
              navigator.serviceWorker.controller === registration.active
            ].join("|");
            registration.active.postMessage({ value: 41 });
            fetch("https://sandbox.test/app/intercepted")
              .then(response => response.text())
              .then(value => serviceWorkerFetchAnswer = value);
          });
          return "scheduled";
        })()
        "#,
    );
    let answer = text(&mut runtime, "serviceWorkerAnswer");
    assert_eq!(
        answer,
        concat!(
            "42|true|true|true|true|true|true|window|https://sandbox.test/app/|true|",
            "true|true|true|activated|true|true|true|",
            "function postMessage() { [native code] }"
        )
    );
    let registration = text(&mut runtime, "serviceRegistrationShape");
    assert_eq!(
        registration,
        "true|true|true|activated|https://sandbox.test/app/|none|true|true|true|true|true|true|true|true"
    );
    let controller_changes = text(&mut runtime, "serviceWorkerControllerChanges");
    assert_eq!(controller_changes, "1");
    let fetch_answer = text(&mut runtime, "serviceWorkerFetchAnswer");
    assert_eq!(
        fetch_answer,
        "true|true|true|https://sandbox.test/app/intercepted|true|true|||"
    );
}

#[test]
fn dedicated_worker_prototype_surface_matches_edge_evidence() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            postMessage([
              Object.getOwnPropertyNames(DedicatedWorkerGlobalScope.prototype).join(","),
              Object.getOwnPropertyNames(WorkerGlobalScope.prototype).join(","),
              Object.getOwnPropertyNames(WorkerNavigator.prototype).join(","),
              Object.getOwnPropertyNames(globalThis).length,
              typeof SharedWorker,
              typeof EventSource,
              typeof WebTransport,
              typeof FileSystemObserver,
              typeof GPUDevice,
              typeof HID,
              typeof USB
            ].join("|"));
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerSurfaceEvidence = "pending";
          worker.onmessage = event => workerSurfaceEvidence = event.data;
          return "scheduled";
        })()
        "#,
    );
    let surface = text(&mut runtime, "workerSurfaceEvidence");
    assert_eq!(
        surface,
        concat!(
            "TEMPORARY,PERSISTENT,constructor|",
            "self,location,onerror,onlanguagechange,navigator,onrejectionhandled,",
            "onunhandledrejection,origin,performance,trustedTypes,crypto,indexedDB,",
            "fonts,createImageBitmap,fetch,importScripts,constructor,isSecureContext,",
            "crossOriginIsolated,scheduler,caches,atob,btoa,queueMicrotask,reportError,",
            "structuredClone,clearInterval,clearTimeout,setInterval,setTimeout|",
            "hardwareConcurrency,appCodeName,appName,appVersion,platform,product,",
            "userAgent,language,languages,onLine,connection,constructor,hid,",
            "mediaCapabilities,permissions,serial,usb,deviceMemory,userAgentData,",
            "locks,storage,gpu,storageBuckets|",
            "334|undefined|function|function|function|function|function|function"
        )
    );
}

#[test]
fn dedicated_worker_surface_switches_with_chromium_140_through_151() {
    let expected = [
        (140, 328),
        (141, 332),
        (142, 332),
        (143, 332),
        (144, 333),
        (145, 334),
        (146, 334),
        (147, 334),
        (148, 334),
        (149, 334),
        (150, 334),
        (151, 335),
    ];
    for (major, count) in expected {
        let mut fingerprint = crate::EdgeFingerprint::default();
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
        let mut runtime = EdgeRuntime::with_fingerprint(fingerprint)
            .unwrap_or_else(|error| panic!("Chromium {major} runtime: {error}"));
        text(
            &mut runtime,
            r#"
            (() => {
              const source = `postMessage([
                Object.getOwnPropertyNames(globalThis).length,
                Object.getOwnPropertyNames(WorkerNavigator.prototype).join(","),
                (reflectKeys => {
                  const keyName = key => typeof key === "symbol" ?
                    "@@" + String(key.description || "") : key;
                  const ownKeys = value => reflectKeys ?
                    Reflect.ownKeys(value).map(keyName) :
                    Object.getOwnPropertyNames(value);
                  const records = [];
                  const names = Object.getOwnPropertyNames(globalThis).sort();
                  for (const owner of names) {
                    const descriptor = Object.getOwnPropertyDescriptor(globalThis, owner);
                    if (!descriptor || !("value" in descriptor)) continue;
                    const value = descriptor.value;
                    if (typeof value === "function") {
                      if (value.prototype) records.push(
                        "constructorPrototypes:" + owner + ":" +
                        ownKeys(value.prototype).join("\\u001e")
                      );
                      records.push(
                        "constructorStatics:" + owner + ":" +
                        ownKeys(value).join("\\u001e")
                      );
                    } else if (value && typeof value === "object" && value !== globalThis) {
                      const objectNames = Object.getOwnPropertyNames(value);
                      if (objectNames.length) records.push(
                        "globalObjects:" + owner + ":" + ownKeys(value).join("\\u001e")
                      );
                    }
                  }
                  records.sort();
                  let hash = 2166136261;
                  const input = records.join("\\u001f");
                  for (let index = 0; index < input.length; index += 1) {
                    hash = Math.imul(hash ^ input.charCodeAt(index), 16777619);
                  }
                  return String(hash >>> 0);
                })(false),
                (reflectKeys => {
                  const keyName = key => typeof key === "symbol" ?
                    "@@" + String(key.description || "") : key;
                  const ownKeys = value => reflectKeys ?
                    Reflect.ownKeys(value).map(keyName) :
                    Object.getOwnPropertyNames(value);
                  const records = [];
                  for (const owner of Object.getOwnPropertyNames(globalThis).sort()) {
                    const descriptor = Object.getOwnPropertyDescriptor(globalThis, owner);
                    if (!descriptor || !("value" in descriptor)) continue;
                    const value = descriptor.value;
                    if (typeof value === "function") {
                      if (value.prototype) records.push(
                        "constructorPrototypes:" + owner + ":" + ownKeys(value.prototype).join("\\u001e")
                      );
                      records.push(
                        "constructorStatics:" + owner + ":" + ownKeys(value).join("\\u001e")
                      );
                    } else if (value && typeof value === "object" && value !== globalThis) {
                      if (Object.getOwnPropertyNames(value).length) records.push(
                        "globalObjects:" + owner + ":" + ownKeys(value).join("\\u001e")
                      );
                    }
                  }
                  records.sort();
                  let hash = 2166136261;
                  const input = records.join("\\u001f");
                  for (let index = 0; index < input.length; index += 1) {
                    hash = Math.imul(hash ^ input.charCodeAt(index), 16777619);
                  }
                  return String(hash >>> 0);
                })(true)
              ].join("|"))`;
              const worker = new Worker("data:text/javascript," + encodeURIComponent(source));
              globalThis.versionWorkerSurface = "pending";
              worker.onmessage = event => versionWorkerSurface = event.data;
              return "scheduled";
            })()
            "#,
        );
        let observed = text(&mut runtime, "versionWorkerSurface");
        let navigator = crate::browser_surface_data::worker_navigator_names(major).join(",");
        let surface_hash =
            crate::browser_surface_data::expected_worker_versioned_surface_hash(major);
        let surface_keys_hash =
            crate::browser_surface_data::expected_worker_versioned_surface_keys_hash(major);
        assert_eq!(
            observed,
            format!("{count}|{navigator}|{surface_hash}|{surface_keys_hash}"),
            "Chromium {major}"
        );
    }
}

#[test]
#[ignore = "developer diagnostic for generated worker surface allowlists"]
fn diagnose_chromium_140_worker_surface_owners() {
    let mut fingerprint = crate::EdgeFingerprint::default();
    fingerprint.navigator.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
(KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36"
        .to_owned();
    fingerprint.navigator.app_version = fingerprint
        .navigator
        .user_agent
        .strip_prefix("Mozilla/")
        .unwrap()
        .to_owned();
    let mut runtime = EdgeRuntime::with_fingerprint(fingerprint).unwrap();
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
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
            postMessage(JSON.stringify(records));
          `;
          const worker = new Worker("data:text/javascript," + encodeURIComponent(source));
          globalThis.workerOwnerDiagnostic = "pending";
          worker.onmessage = event => workerOwnerDiagnostic = event.data;
        })()
        "#,
    );
    let observed = text(&mut runtime, "workerOwnerDiagnostic");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("build/chromium-version-surfaces/sandbox-140-worker-owner-diagnostic.json");
    std::fs::write(&path, observed).unwrap();
    panic!("wrote {}", path.display());
}

#[test]
fn android_dedicated_worker_surface_switches_with_chromium_140_through_151() {
    let expected_counts = [321, 325, 325, 325, 326, 329, 329, 329, 329, 329, 327, 328];
    for (offset, count) in expected_counts.into_iter().enumerate() {
        let major = 140 + offset as u16;
        let mut fingerprint = crate::EdgeFingerprint::default();
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
        let mut runtime = EdgeRuntime::with_fingerprint(fingerprint)
            .unwrap_or_else(|error| panic!("Android Chromium {major} runtime: {error}"));
        text(
            &mut runtime,
            r#"
            (() => {
              const source = `postMessage([Object.getOwnPropertyNames(globalThis).length,
                Object.getOwnPropertyNames(WorkerNavigator.prototype).join(","),
                (reflectKeys => {
                  const keyName = key => typeof key === "symbol" ?
                    "@@" + String(key.description || "") : key;
                  const ownKeys = value => (reflectKeys ? Reflect.ownKeys(value) :
                    Object.getOwnPropertyNames(value)).map(keyName);
                  const records = [];
                  for (const owner of Object.getOwnPropertyNames(globalThis).sort()) {
                    const descriptor = Object.getOwnPropertyDescriptor(globalThis, owner);
                    if (!descriptor || !("value" in descriptor)) continue;
                    const value = descriptor.value;
                    if (typeof value === "function") {
                      if (value.prototype) records.push("constructorPrototypes:" + owner + ":" +
                        ownKeys(value.prototype).join("\\u001e"));
                      records.push("constructorStatics:" + owner + ":" + ownKeys(value).join("\\u001e"));
                    } else if (value && typeof value === "object" && value !== globalThis &&
                               Object.getOwnPropertyNames(value).length) {
                      records.push("globalObjects:" + owner + ":" + ownKeys(value).join("\\u001e"));
                    }
                  }
                  records.sort();
                  let hash = 2166136261;
                  const input = records.join("\\u001f");
                  for (let index = 0; index < input.length; index += 1) {
                    hash = Math.imul(hash ^ input.charCodeAt(index), 16777619);
                  }
                  return String(hash >>> 0);
                })(false),
                (reflectKeys => {
                  const keyName = key => typeof key === "symbol" ?
                    "@@" + String(key.description || "") : key;
                  const ownKeys = value => (reflectKeys ? Reflect.ownKeys(value) :
                    Object.getOwnPropertyNames(value)).map(keyName);
                  const records = [];
                  for (const owner of Object.getOwnPropertyNames(globalThis).sort()) {
                    const descriptor = Object.getOwnPropertyDescriptor(globalThis, owner);
                    if (!descriptor || !("value" in descriptor)) continue;
                    const value = descriptor.value;
                    if (typeof value === "function") {
                      if (value.prototype) records.push("constructorPrototypes:" + owner + ":" +
                        ownKeys(value.prototype).join("\\u001e"));
                      records.push("constructorStatics:" + owner + ":" + ownKeys(value).join("\\u001e"));
                    } else if (value && typeof value === "object" && value !== globalThis &&
                               Object.getOwnPropertyNames(value).length) {
                      records.push("globalObjects:" + owner + ":" + ownKeys(value).join("\\u001e"));
                    }
                  }
                  records.sort();
                  let hash = 2166136261;
                  const input = records.join("\\u001f");
                  for (let index = 0; index < input.length; index += 1) {
                    hash = Math.imul(hash ^ input.charCodeAt(index), 16777619);
                  }
                  return String(hash >>> 0);
                })(true)].join("|"))`;
              const worker = new Worker("data:text/javascript," + encodeURIComponent(source));
              globalThis.androidWorkerSurface = "pending";
              worker.onmessage = event => androidWorkerSurface = event.data;
              return "scheduled";
            })()
            "#,
        );
        let observed = text(&mut runtime, "androidWorkerSurface");
        let navigator =
            crate::browser_android_surface_data::worker_navigator_names(major).join(",");
        let surface =
            crate::browser_android_surface_data::expected_worker_versioned_surface_hash(major);
        let keys =
            crate::browser_android_surface_data::expected_worker_versioned_surface_keys_hash(major);
        assert_eq!(
            observed,
            format!("{count}|{navigator}|{surface}|{keys}"),
            "Android Chromium {major} worker HTTPS surface",
        );
    }
}

#[test]
fn dedicated_worker_sync_reader_and_rtc_transform_are_functional() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            const reader = new FileReaderSync();
            const blob = new Blob([
              new Uint8Array([65, 66, 67])
            ], { type: "text/plain" });
            postMessage([
              reader.readAsText(blob),
              reader.readAsBinaryString(blob),
              reader.readAsArrayBuffer(blob).byteLength,
              reader.readAsDataURL(blob),
              typeof FileSystemSyncAccessHandle,
              Object.getOwnPropertyNames(
                FileSystemSyncAccessHandle.prototype
              ).join(",")
            ].join("|"));
            (async () => {
              const root = await navigator.storage.getDirectory();
              const file = await root.getFileHandle("worker.bin", {
                create: true
              });
              const access = await file.createSyncAccessHandle();
              const written = access.write(
                new Uint8Array([70, 83, 65, 80, 73])
              );
              access.flush();
              const output = new Uint8Array(5);
              const read = access.read(output);
              const size = access.getSize();
              access.close();
              postMessage([
                "FS",
                file instanceof FileSystemFileHandle,
                typeof file.createSyncAccessHandle,
                access instanceof FileSystemSyncAccessHandle,
                written,
                read,
                size,
                String.fromCharCode(...output)
              ].join("|"));
            })().catch(error => postMessage(
              "FSERR|" + error.name + "|" + error.message
            ));
            onrtctransform = event => {
              postMessage([
                event instanceof RTCTransformEvent,
                event.transformer instanceof RTCRtpScriptTransformer,
                event.transformer.options.value,
                event.transformer.readable instanceof ReadableStream,
                event.transformer.writable instanceof WritableStream,
                typeof event.transformer.generateKeyFrame === "undefined",
                event.transformer.sendKeyFrameRequest() instanceof Promise
              ].join("|"));
            };
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.syncReaderAnswer = "pending";
          globalThis.syncAccessAnswer = "pending";
          globalThis.rtcTransformAnswer = "pending";
          worker.onmessage = event => {
            if (String(event.data).startsWith("ABC|")) {
              syncReaderAnswer = event.data;
            } else if (String(event.data).startsWith("FS")) {
              syncAccessAnswer = event.data;
            } else {
              rtcTransformAnswer = event.data;
            }
          };
          new RTCRtpScriptTransform(worker, { value: 42 });
          return "scheduled";
        })()
        "#,
    );
    let reader = text(&mut runtime, "syncReaderAnswer");
    assert_eq!(
        reader,
        "ABC|ABC|3|data:text/plain;base64,QUJD|function|close,flush,getSize,read,truncate,write,mode,constructor"
    );
    let access = text(&mut runtime, "syncAccessAnswer");
    assert_eq!(access, "FS|true|function|true|5|5|5|FSAPI");
    let transform = text(&mut runtime, "rtcTransformAnswer");
    assert_eq!(transform, "true|true|42|true|true|true|true");
}
