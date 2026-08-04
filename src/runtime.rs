use std::sync::Once;

static V8_INIT: Once = Once::new();

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Evaluation {
    Undefined,
    Null,
    Boolean(bool),
    Number(String),
    String(String),
    Other(String),
}

impl std::fmt::Display for Evaluation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => formatter.write_str("undefined"),
            Self::Null => formatter.write_str("null"),
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::Number(value) | Self::String(value) | Self::Other(value) => {
                formatter.write_str(value)
            }
        }
    }
}

pub struct EdgeRuntime {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
    limits: crate::SandboxLimits,
    locale: String,
    time_zone: String,
    #[allow(dead_code)]
    late_intrinsics: crate::intrinsics::LateIntrinsics,
}

impl EdgeRuntime {
    pub fn new() -> Result<Self, String> {
        Self::with_options(crate::EdgeRuntimeOptions::default())
    }

    pub fn with_fingerprint(
        fingerprint: crate::fingerprint::EdgeFingerprint,
    ) -> Result<Self, String> {
        Self::new_with_fingerprint(fingerprint)
    }

    pub fn new_with_fingerprint(
        fingerprint: crate::fingerprint::EdgeFingerprint,
    ) -> Result<Self, String> {
        Self::with_options(crate::EdgeRuntimeOptions {
            fingerprint,
            ..Default::default()
        })
    }

    pub fn with_options(options: crate::EdgeRuntimeOptions) -> Result<Self, String> {
        options.validate()?;
        initialize_v8();

        let locale = options.fingerprint.locale.locale.clone();
        let time_zone = options.fingerprint.locale.time_zone.clone();
        let _locale_guard = crate::locale_runtime::lock_process_defaults();
        crate::locale_runtime::configure_process_defaults(&locale, &time_zone)?;
        let create_params = options
            .limits
            .max_heap_bytes
            .map_or_else(v8::CreateParams::default, |maximum| {
                v8::CreateParams::default().heap_limits(0, maximum)
            });
        let mut isolate = v8::Isolate::new(create_params);
        let mut deterministic = options.deterministic;
        options.fingerprint.timing.apply(&mut deterministic);
        crate::page_init::prepare(&mut isolate, options.page)?;
        crate::fingerprint::prepare(&mut isolate, options.fingerprint);
        crate::determinism::prepare(&mut isolate, deterministic);
        crate::locale_runtime::prepare(&mut isolate);
        crate::network_replay::prepare(&mut isolate, options.network_replay);
        crate::network_capture::prepare(&mut isolate);
        crate::console_capture::prepare(&mut isolate);
        crate::trace::prepare(&mut isolate);
        crate::iframe_hook::prepare(&mut isolate, options.iframe_hooks)?;
        crate::web::prepare(&mut isolate);
        let (context, late_intrinsics) = {
            v8::scope!(let scope, &mut isolate);
            let global_template = crate::web::root_window_proxy::global_template(scope);
            let context = v8::Context::new(
                scope,
                v8::ContextOptions {
                    global_template: Some(global_template),
                    ..Default::default()
                },
            );
            let context_scope = &mut v8::ContextScope::new(scope, context);
            let late_intrinsics =
                crate::intrinsics::LateIntrinsics::detach(context_scope, context)?;
            crate::web::install_prefix(context_scope)?;
            let temporal = v8::Local::new(context_scope, &late_intrinsics.temporal);
            crate::web::temporal_global::install(context_scope, temporal)?;
            let suppressed_error = v8::Local::new(context_scope, &late_intrinsics.suppressed_error);
            crate::web::suppressed_error_global::install(context_scope, suppressed_error)?;
            let disposable_stack = v8::Local::new(context_scope, &late_intrinsics.disposable_stack);
            crate::web::disposable_stack_global::install(context_scope, disposable_stack)?;
            let async_disposable_stack =
                v8::Local::new(context_scope, &late_intrinsics.async_disposable_stack);
            crate::web::async_disposable_stack_global::install(
                context_scope,
                async_disposable_stack,
            )?;
            let float16_array = v8::Local::new(context_scope, &late_intrinsics.float16_array);
            crate::web::float16_array_global::install(context_scope, float16_array)?;
            crate::web::install_after_late_intrinsics(context_scope)?;
            let web_assembly = v8::Local::new(context_scope, &late_intrinsics.web_assembly);
            crate::web::web_assembly_global::install(context_scope, web_assembly)?;
            crate::web::install_after_webassembly(context_scope)?;
            crate::locale_runtime::install(context_scope)?;
            crate::determinism::install(context_scope)?;
            crate::iframe_hook::install_for_root(context_scope)?;
            (v8::Global::new(context_scope, context), late_intrinsics)
        };
        {
            // Re-enter the finished root realm from a fresh handle scope.
            // Some API installers create nested realms; capturing here avoids
            // observing one of those temporary globals as the Window surface.
            v8::scope!(let scope, &mut isolate);
            let root_context = v8::Local::new(scope, &context);
            let root_scope = &mut v8::ContextScope::new(scope, root_context);
            let root_window = root_context.global(root_scope);
            crate::web::html_i_frame_element::capture_window_surface(root_scope, root_window);
        }

        Ok(Self {
            isolate,
            context,
            limits: options.limits,
            locale,
            time_zone,
            late_intrinsics,
        })
    }

    pub fn evaluate(&mut self, source: &str) -> Result<Evaluation, String> {
        self.evaluate_internal(source, None)
    }

    pub fn evaluate_with_source_url(
        &mut self,
        source: &str,
        source_url: &str,
    ) -> Result<Evaluation, String> {
        if source_url.is_empty() {
            return Err("JavaScript source URL cannot be empty".to_owned());
        }
        if source_url.len() > 64 * 1024 || source_url.contains('\0') {
            return Err("JavaScript source URL is invalid or too large".to_owned());
        }
        self.evaluate_internal(source, Some(source_url))
    }

    fn evaluate_internal(
        &mut self,
        source: &str,
        source_url: Option<&str>,
    ) -> Result<Evaluation, String> {
        if self
            .limits
            .max_source_bytes
            .is_some_and(|maximum| source.len() > maximum)
        {
            return Err("JavaScript source exceeded max_source_bytes".to_owned());
        }
        let _locale_guard = crate::locale_runtime::lock_process_defaults();
        crate::locale_runtime::configure_process_defaults(&self.locale, &self.time_zone)?;
        let watchdog = EvaluationWatchdog::start(&self.isolate, self.limits.timeout);
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        v8::tc_scope!(let try_catch, scope);
        let source = v8::String::new(try_catch, source)
            .ok_or_else(|| "source exceeds V8 limits".to_owned())?;
        let traced = crate::trace::is_enabled(try_catch);
        if traced {
            crate::trace::start_recording(try_catch)?;
        }
        let value = if let Some(source_url) = source_url {
            let resource_name = v8::String::new(try_catch, source_url)
                .ok_or_else(|| "JavaScript source URL exceeds V8 limits".to_owned())?;
            let origin = v8::ScriptOrigin::new(
                try_catch,
                resource_name.into(),
                0,
                0,
                false,
                -1,
                None,
                false,
                false,
                false,
                None,
            );
            v8::Script::compile(try_catch, source, Some(&origin))
                .and_then(|script| script.run(try_catch))
        } else {
            v8::Script::compile(try_catch, source, None).and_then(|script| script.run(try_catch))
        };
        let Some(value) = value else {
            if traced {
                crate::trace::stop_recording(try_catch);
            }
            let message = if watchdog.timed_out() {
                "JavaScript execution exceeded the configured timeout".to_owned()
            } else {
                caught_exception_text(try_catch, "JavaScript compilation or execution failed")
            };
            let _ = try_catch.cancel_terminate_execution();
            return Err(message);
        };
        try_catch.perform_microtask_checkpoint();
        for _ in 0..crate::determinism::max_task_turns(try_catch) {
            if !crate::web::run_pending_tasks(try_catch) {
                break;
            }
            crate::determinism::advance_task_turn(try_catch);
            try_catch.perform_microtask_checkpoint();
        }
        if watchdog.timed_out() {
            if traced {
                crate::trace::stop_recording(try_catch);
            }
            let _ = try_catch.cancel_terminate_execution();
            return Err("JavaScript execution exceeded the configured timeout".to_owned());
        }
        let value = if let Ok(promise) = v8::Local::<v8::Promise>::try_from(value) {
            match promise.state() {
                v8::PromiseState::Fulfilled => promise.result(try_catch),
                v8::PromiseState::Rejected => {
                    promise.mark_as_handled();
                    let rejected_value = promise.result(try_catch);
                    let rejection = rejection_text(try_catch, rejected_value);
                    if traced {
                        crate::trace::stop_recording(try_catch);
                    }
                    return Err(format!("top-level Promise rejected: {rejection}"));
                }
                v8::PromiseState::Pending => {
                    if traced {
                        crate::trace::stop_recording(try_catch);
                    }
                    return Err(
                        "top-level Promise remained pending after the configured task turns"
                            .to_owned(),
                    );
                }
            }
        } else {
            value
        };
        let value = if traced {
            crate::trace::stop_recording(try_catch);
            value
        } else {
            value
        };
        let evaluation = value_to_evaluation(try_catch, value);
        if self
            .limits
            .max_output_bytes
            .is_some_and(|maximum| evaluation.to_string().len() > maximum)
        {
            return Err("JavaScript output exceeded max_output_bytes".to_owned());
        }
        Ok(evaluation)
    }

    pub fn enable_native_trace(&mut self) -> Result<(), String> {
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        crate::trace::enable(scope)?;
        crate::web::enable_native_trace_for_existing_realms(scope)
    }

    pub fn enable_proxy_trace(&mut self) -> Result<(), String> {
        self.enable_native_trace()
    }

    pub fn disable_native_trace(&mut self) {
        crate::web::disable_native_trace_for_existing_realms(&mut self.isolate);
        crate::trace::disable(&mut self.isolate);
    }

    pub fn disable_proxy_trace(&mut self) {
        self.disable_native_trace();
    }

    pub fn clear_native_trace(&mut self) {
        crate::trace::clear(&mut self.isolate);
    }

    pub fn set_native_trace_exclusions(&mut self, exclusions: &[String]) -> Result<(), String> {
        crate::trace::set_excluded_apis(&mut self.isolate, exclusions)
    }

    pub fn clear_proxy_trace(&mut self) {
        self.clear_native_trace();
    }

    pub fn native_trace(&self) -> Vec<crate::trace::TraceEntry> {
        crate::trace::entries(&self.isolate)
    }

    pub fn native_trace_matching(&self, needle: &str) -> Vec<crate::trace::TraceEntry> {
        crate::trace::matching_entries(&self.isolate, needle)
    }

    pub fn proxy_trace(&self) -> Vec<crate::trace::TraceEntry> {
        self.native_trace()
    }

    pub fn proxy_trace_matching(&self, needle: &str) -> Vec<crate::trace::TraceEntry> {
        self.native_trace_matching(needle)
    }

    pub fn network_requests(&self) -> Vec<crate::CapturedNetworkRequest> {
        crate::network_capture::entries(&self.isolate)
    }

    pub fn clear_network_requests(&mut self) {
        crate::network_capture::clear(&mut self.isolate);
    }

    pub fn stdout(&self) -> Vec<crate::CapturedConsoleOutput> {
        crate::console_capture::entries(&self.isolate)
    }

    pub fn clear_stdout(&mut self) {
        crate::console_capture::clear(&mut self.isolate);
    }

    #[cfg(test)]
    pub(crate) fn evaluate_without_native_trace_entries(
        &mut self,
        source: &str,
    ) -> Result<Evaluation, String> {
        let _locale_guard = crate::locale_runtime::lock_process_defaults();
        crate::locale_runtime::configure_process_defaults(&self.locale, &self.time_zone)?;
        v8::scope!(let scope, &mut self.isolate);
        let context = v8::Local::new(scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        v8::tc_scope!(let try_catch, scope);
        let source = v8::String::new(try_catch, source)
            .ok_or_else(|| "source exceeds V8 limits".to_owned())?;
        if !crate::trace::is_enabled(try_catch) {
            return Err("native trace is not enabled".to_owned());
        }
        crate::trace::stop_recording(try_catch);
        let Some(value) =
            v8::Script::compile(try_catch, source, None).and_then(|script| script.run(try_catch))
        else {
            return Err(caught_exception_text(
                try_catch,
                "JavaScript compilation or execution failed",
            ));
        };
        Ok(value_to_evaluation(try_catch, value))
    }
}

struct EvaluationWatchdog {
    cancel: Option<std::sync::mpsc::Sender<()>>,
    timed_out: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl EvaluationWatchdog {
    fn start(isolate: &v8::OwnedIsolate, timeout: Option<std::time::Duration>) -> Self {
        let timed_out = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let Some(timeout) = timeout else {
            return Self {
                cancel: None,
                timed_out,
                thread: None,
            };
        };
        let handle = isolate.thread_safe_handle();
        let (sender, receiver) = std::sync::mpsc::channel();
        let signal = timed_out.clone();
        let thread = std::thread::spawn(move || {
            if receiver.recv_timeout(timeout).is_err() {
                signal.store(true, std::sync::atomic::Ordering::Release);
                let _ = handle.terminate_execution();
            }
        });
        Self {
            cancel: Some(sender),
            timed_out,
            thread: Some(thread),
        }
    }

    fn timed_out(&self) -> bool {
        self.timed_out.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl Drop for EvaluationWatchdog {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn initialize_v8() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn rejection_text(scope: &mut v8::PinScope<'_, '_>, rejection: v8::Local<'_, v8::Value>) -> String {
    let display = rejection
        .to_string(scope)
        .map(|value| value.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "Promise rejected".to_owned());
    let stack = v8::Local::<v8::Object>::try_from(rejection)
        .ok()
        .and_then(|object| {
            let key = v8::String::new(scope, "stack")?;
            object.get(scope, key.into())
        })
        .and_then(|value| value.to_string(scope))
        .map(|value| value.to_rust_string_lossy(scope))
        .filter(|value| !value.is_empty());
    match stack {
        Some(stack) if stack == display || stack.starts_with(&format!("{display}\n")) => stack,
        Some(stack) => format!("{display}\n{stack}"),
        None => display,
    }
}

fn caught_exception_text(
    scope: &v8::PinnedRef<'_, v8::TryCatch<'_, '_, v8::HandleScope<'_, v8::Context>>>,
    fallback: &str,
) -> String {
    let Some(exception) = scope.exception() else {
        return fallback.to_owned();
    };
    let exception_text = exception
        .to_string(scope)
        .map(|text| text.to_rust_string_lossy(scope))
        .unwrap_or_else(|| fallback.to_owned());
    let stack = scope
        .stack_trace()
        .and_then(|value| value.to_string(scope))
        .map(|text| text.to_rust_string_lossy(scope))
        .filter(|text| !text.is_empty());
    match stack {
        Some(stack)
            if stack == exception_text || stack.starts_with(&format!("{exception_text}\n")) =>
        {
            stack
        }
        Some(stack) => format!("{exception_text}\n{stack}"),
        None => exception_text,
    }
}

fn value_to_evaluation(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Evaluation {
    if value.is_undefined() {
        return Evaluation::Undefined;
    }
    if value.is_null() {
        return Evaluation::Null;
    }
    if value.is_boolean() {
        return Evaluation::Boolean(value.boolean_value(scope));
    }
    let text = value
        .to_string(scope)
        .map(|text| text.to_rust_string_lossy(scope))
        .unwrap_or_default();
    if value.is_number() {
        Evaluation::Number(text)
    } else if value.is_string() || value.is_string_object() {
        Evaluation::String(text)
    } else {
        Evaluation::Other(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{EdgeRuntime, Evaluation};

    fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
        match runtime.evaluate(source).expect("JavaScript evaluation") {
            Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => {
                value
            }
            value => value.to_string(),
        }
    }

    fn native_text_without_trace_entries(runtime: &mut EdgeRuntime, source: &str) -> String {
        match runtime
            .evaluate_without_native_trace_entries(source)
            .expect("native-trace JavaScript evaluation")
        {
            Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => {
                value
            }
            value => value.to_string(),
        }
    }

    #[test]
    fn explicit_source_url_names_error_stacks_and_uncaught_diagnostics() {
        let mut runtime = EdgeRuntime::new().expect("runtime");
        let source_url = "https://exxxxx.com/a.js";
        let stack = runtime
            .evaluate_with_source_url(
                r#"
                function inner() { return new Error("probe").stack; }
                inner();
                "#,
                source_url,
            )
            .expect("named stack")
            .to_string();
        assert!(
            stack.contains(source_url),
            "stack did not contain URL: {stack}"
        );
        assert!(
            stack.contains("inner"),
            "stack did not contain function: {stack}"
        );

        let error = runtime
            .evaluate_with_source_url(
                r#"
                function explode() { throw new Error("boom"); }
                explode();
                "#,
                source_url,
            )
            .expect_err("uncaught exception");
        assert!(error.contains("Error: boom"), "missing exception: {error}");
        assert!(error.contains(source_url), "missing source URL: {error}");
        assert!(error.contains("explode"), "missing stack frame: {error}");

        let rejection = runtime
            .evaluate_with_source_url("Promise.reject(new Error('async boom'))", source_url)
            .expect_err("rejected promise");
        assert!(
            rejection.contains(source_url),
            "missing rejection URL: {rejection}"
        );
    }

    #[test]
    fn edge_https_window_surface_and_secure_apis() {
        let mut runtime = EdgeRuntime::new().expect("Edge runtime");
        let surface = text(
            &mut runtime,
            r#"
            (() => {
              const hash = text => {
                let value = 2166136261;
                for (let index = 0; index < text.length; index += 1) {
                  value = Math.imul(value ^ text.charCodeAt(index), 16777619);
                }
                return (value >>> 0).toString(16).padStart(8, "0");
              };
              const names = Object.getOwnPropertyNames(window);
              const descriptors = names.map(name => {
                const descriptor = Object.getOwnPropertyDescriptor(window, name);
                return name + ":" + ("value" in descriptor ? "d" : "a") + ":" +
                  Number(descriptor.enumerable) + Number(descriptor.configurable) +
                  Number(Boolean(descriptor.writable)) + ":" +
                  Number(Boolean(descriptor.get)) + Number(Boolean(descriptor.set));
              });
              return [
                isSecureContext,
                location.href,
                origin,
                names.length,
                hash(names.join("\u001f")),
                hash(descriptors.join("\u001f")),
                Object.getOwnPropertyNames(Navigator.prototype).length,
                hash(Object.getOwnPropertyNames(Navigator.prototype).join("\u001f")),
                Object.getOwnPropertyNames(console).length,
                hash(Object.getOwnPropertyNames(console).join("\u001f")),
                Object.prototype.toString.call(console)
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            surface,
            "true|https://sandbox.test/|https://sandbox.test|1232|60594b80|946e759f|83|fbf27bf3|25|9f9f9a5f|[object console]"
        );

        let complete_surface = text(
            &mut runtime,
            r#"
            (() => {
              const hash = text => {
                let value = 2166136261;
                for (let index = 0; index < text.length; index += 1) {
                  value = Math.imul(value ^ text.charCodeAt(index), 16777619);
                }
                return (value >>> 0).toString(16).padStart(8, "0");
              };
              const keyName = key =>
                typeof key === "symbol" ? "@@" + (key.description || "") : key;
              const functionValue = value => {
                if (typeof value === "function") {
                  return value.name + "/" + value.length + "/" +
                    Function.prototype.toString.call(value);
                }
                if (value === null) {
                  return "null";
                }
                const kind = typeof value;
                return kind + (kind !== "object" && kind !== "function"
                  ? "/" + String(value)
                  : "");
              };
              const descriptor = (object, key) => {
                const value = Object.getOwnPropertyDescriptor(object, key);
                return keyName(key) + ":" + ("value" in value ? "d" : "a") + ":" +
                  Number(value.enumerable) + Number(value.configurable) +
                  Number(Boolean(value.writable)) + ":" +
                  functionValue(value.value) + ":" +
                  functionValue(value.get) + ":" +
                  functionValue(value.set);
              };
              const constructors = [];
              const functions = [];
              const objects = [];
              const primitives = [];
              for (const name of Object.getOwnPropertyNames(globalThis)) {
                let value;
                try {
                  value = globalThis[name];
                } catch (_) {
                  continue;
                }
                if (typeof value === "function") {
                  functions.push(name + "\t" + functionValue(value));
                  if (value.prototype && typeof value.prototype === "object") {
                    const prototype = value.prototype;
                    const parent = Object.getPrototypeOf(prototype);
                    const parentName =
                      parent && parent.constructor && parent.constructor.name || "";
                    constructors.push(
                      name + "\t" + value.name + "/" + value.length + "\t" +
                      parentName + "\t" +
                      Reflect.ownKeys(prototype)
                        .map(key => descriptor(prototype, key))
                        .join("\u001e")
                    );
                  }
                } else if (value !== null && typeof value === "object") {
                  const parent = Object.getPrototypeOf(value);
                  const parentName =
                    parent && parent.constructor && parent.constructor.name || "";
                  objects.push(
                    name + "\t" + Object.prototype.toString.call(value) + "\t" +
                    parentName + "\t" +
                    Reflect.ownKeys(value)
                      .map(key => descriptor(value, key))
                      .join("\u001e")
                  );
                } else {
                  primitives.push(name + "\t" + functionValue(value));
                }
              }
              return [
                constructors.length,
                hash(constructors.join("\n")),
                functions.length,
                hash(functions.join("\n")),
                objects.length,
                hash(objects.join("\n")),
                primitives.length,
                hash(primitives.join("\n"))
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            complete_surface,
            "961|e65af842|1024|a8b3e550|53|59942bf5|155|907ccdc7"
        );

        let relationships = text(
            &mut runtime,
            r#"
            (() => {
              const windowProperties = Object.getPrototypeOf(Window.prototype);
              return [
                Object.getPrototypeOf(window) === Window.prototype,
                windowProperties[Symbol.toStringTag],
                Object.getPrototypeOf(windowProperties) === EventTarget.prototype,
                Object.getPrototypeOf(EventTarget.prototype) === Object.prototype,
                Window.prototype.constructor === Window,
                Object.getPrototypeOf(DOMException) === Function.prototype,
                Object.getPrototypeOf(DOMException.prototype) === Error.prototype,
                DOMException.prototype.constructor === DOMException
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            relationships,
            "true|WindowProperties|true|true|true|true|true|true"
        );

        let document_factories = text(
            &mut runtime,
            r#"
            (() => {
              const listing = document.createElement("listing");
              const circle = document.createElementNS(
                "http://www.w3.org/2000/svg",
                "circle"
              );
              const unknownSvg = document.createElementNS(
                "http://www.w3.org/2000/svg",
                "edge-unknown"
              );
              const math = document.createElementNS(
                "http://www.w3.org/1998/Math/MathML",
                "mfrac"
              );
              const text = document.createTextNode("text");
              const comment = document.createComment("comment");
              const attribute = document.createAttributeNS("urn:edge", "e:data");
              const instruction =
                new DOMParser().parseFromString("<root/>", "application/xml")
                  .createProcessingInstruction("edge", "data");
              let cdata;
              try {
                document.createCDATASection("data");
              } catch (error) {
                cdata = error.name + "/" + error.code;
              }
              return [
                listing.constructor.name,
                listing.localName,
                listing.ownerDocument === document,
                circle.constructor.name,
                circle instanceof SVGCircleElement,
                circle.ownerDocument === document,
                unknownSvg.constructor.name,
                unknownSvg instanceof SVGElement,
                math.constructor.name,
                math instanceof MathMLElement,
                text.constructor.name,
                text.ownerDocument === document,
                comment.constructor.name,
                comment instanceof CharacterData,
                comment.ownerDocument === document,
                attribute.constructor.name,
                attribute.localName,
                attribute.prefix,
                attribute.ownerDocument === document,
                instruction.constructor.name,
                instruction.ownerDocument.contentType,
                cdata
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            document_factories,
            "HTMLPreElement|listing|true|SVGCircleElement|true|true|SVGElement|true|MathMLElement|true|Text|true|Comment|true|true|Attr|data|e|true|ProcessingInstruction|application/xml|NotSupportedError/9"
        );

        let legacy_event_factories = text(
            &mut runtime,
            r#"
            (() => {
              const supported = [
                document.createEvent("BeforeUnloadEvent").constructor.name,
                document.createEvent("CompositionEvent").constructor.name,
                document.createEvent("CustomEvent").constructor.name,
                document.createEvent("DeviceMotionEvent").constructor.name,
                document.createEvent("DeviceOrientationEvent").constructor.name,
                document.createEvent("DragEvent").constructor.name,
                document.createEvent("Events").constructor.name,
                document.createEvent("FocusEvent").constructor.name,
                document.createEvent("HashChangeEvent").constructor.name,
                document.createEvent("KeyboardEvent").constructor.name,
                document.createEvent("MessageEvent").constructor.name,
                document.createEvent("MouseEvents").constructor.name,
                document.createEvent("StorageEvent").constructor.name,
                document.createEvent("TextEvent").constructor.name,
                document.createEvent("UIEvents").constructor.name
              ].join(",");
              let unsupported;
              try {
                document.createEvent("Bogus");
              } catch (error) {
                unsupported = error.name + "/" + error.code;
              }
              return supported + "|" + unsupported;
            })()
            "#,
        );
        assert_eq!(
            legacy_event_factories,
            "BeforeUnloadEvent,CompositionEvent,CustomEvent,DeviceMotionEvent,DeviceOrientationEvent,DragEvent,Event,FocusEvent,HashChangeEvent,KeyboardEvent,MessageEvent,MouseEvent,StorageEvent,TextEvent,UIEvent|NotSupportedError/9"
        );

        let xr_layers = text(
            &mut runtime,
            r#"
            (() => {
              const binding = new XRWebGLBinding({}, {});
              const layer = binding.createQuadLayer({width: 2, height: 3, opacity: 0.5});
              layer.width = 4;
              layer.opacity = 0.75;
              const subImage = binding.getSubImage(layer, {});
              const depth = binding.getDepthInformation({});
              const beforeDestroy = [
                layer instanceof XRQuadLayer,
                layer instanceof XRCompositionLayer,
                layer.width,
                layer.height,
                layer.opacity,
                layer.needsRedraw,
                subImage instanceof XRWebGLSubImage,
                subImage.viewport instanceof XRViewport,
                depth instanceof XRWebGLDepthInformation
              ].join(",");
              layer.destroy();
              return beforeDestroy + "," + layer.needsRedraw;
            })()
            "#,
        );
        assert_eq!(xr_layers, "true,true,4,3,0.75,true,true,true,true,false");

        assert_eq!(
            text(
                &mut runtime,
                r#"
                var secureSession;
                navigator.xr.requestSession("inline").then(session => secureSession = session);
                "queued"
                "#,
            ),
            "queued"
        );
        assert_eq!(
            text(
                &mut runtime,
                "secureSession instanceof XRSession && secureSession.environmentBlendMode === 'opaque'",
            ),
            "true"
        );

        assert_eq!(
            text(
                &mut runtime,
                r#"
                localStorage.clear();
                sessionStorage.clear();
                localStorage.setItem("alpha", "1");
                sessionStorage.setItem("beta", "2");
                var edgeAudioContext = new AudioContext();
                var edgeSinkResolved = false;
                var edgeTopics;
                var edgePrivateToken;
                var edgeRedemptionRecord;
                edgeAudioContext.setSinkId({type: "none"}).then(
                  () => edgeSinkResolved = true
                );
                document.browsingTopics().then(value => edgeTopics = value);
                document.hasPrivateToken("https://issuer.test").then(
                  value => edgePrivateToken = value
                );
                document.hasRedemptionRecord("https://issuer.test").then(
                  value => edgeRedemptionRecord = value
                );
                "queued"
                "#,
            ),
            "queued"
        );
        assert_eq!(
            text(
                &mut runtime,
                r#"
                (() => {
                  let rawUpdates = 0;
                  const element = document.createElement("div");
                  element.onpointerrawupdate = () => rawUpdates += 1;
                  element.dispatchEvent(new Event("pointerrawupdate"));
                  const link = document.createElement("link");
                  link.relList = "preload stylesheet preload";
                  const uuid = crypto.randomUUID();
                  const memory = Object.getOwnPropertyDescriptor(console, "memory");
                  return [
                    localStorage instanceof Storage,
                    sessionStorage instanceof Storage,
                    localStorage !== sessionStorage,
                    localStorage.length,
                    localStorage.key(0),
                    localStorage.getItem("alpha"),
                    sessionStorage.getItem("beta"),
                    crypto.subtle instanceof SubtleCrypto,
                    uuid.length,
                    uuid[14],
                    ["8", "9", "a", "b"].includes(uuid[19]),
                    edgeAudioContext.audioWorklet instanceof AudioWorklet,
                    edgeAudioContext.sinkId instanceof AudioSinkInfo,
                    edgeAudioContext.sinkId.type,
                    edgeSinkResolved,
                    Array.isArray(edgeTopics),
                    edgePrivateToken,
                    edgeRedemptionRecord,
                    rawUpdates,
                    link.relList.value,
                    document.location === location,
                    CSS.paintWorklet instanceof Worklet,
                    typeof WebAssembly.compileStreaming,
                    typeof WebAssembly.instantiateStreaming,
                    memory.get.name,
                    memory.get.length,
                    memory.set.name,
                    memory.set.length,
                    Object.keys(Element.prototype[Symbol.unscopables]).join(",")
                  ].join("|");
                })()
                "#,
            ),
            "true|true|true|1|alpha|1|2|true|36|4|true|true|true|none|true|true|false|false|1|preload stylesheet|true|true|function|function||0||0|after,append,before,prepend,remove,replaceChildren,replaceWith,slot"
        );

        assert_eq!(
            text(
                &mut runtime,
                r#"
                var edgeCompiledModule;
                var edgeInstantiatedModule;
                var edgeInstantiatedInstance;
                var edgeCompileError;
                var edgeInstantiateError;
                var edgeResponseBytes;
                var edgeInvalidMimeError;
                var edgeConsumedResponseError;
                const edgeWasmBytes =
                  new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]);
                new Response(edgeWasmBytes).arrayBuffer().then(
                  buffer => edgeResponseBytes = Array.from(new Uint8Array(buffer)).join(",")
                );
                WebAssembly.compileStreaming(
                  new Response(edgeWasmBytes, {
                    headers: new Headers([["Content-Type", "application/wasm"]])
                  })
                ).then(
                  module => edgeCompiledModule = module,
                  error => edgeCompileError = error
                );
                WebAssembly.instantiateStreaming(
                  new Response(edgeWasmBytes, {
                    headers: new Headers([["Content-Type", "application/wasm"]])
                  })
                ).then(result => {
                  edgeInstantiatedModule = result.module;
                  edgeInstantiatedInstance = result.instance;
                }, error => edgeInstantiateError = error);
                WebAssembly.compileStreaming(new Response(edgeWasmBytes)).then(
                  () => edgeInvalidMimeError = "resolved",
                  error => edgeInvalidMimeError = error.name
                );
                const edgeConsumedResponse = new Response(edgeWasmBytes, {
                  headers: new Headers([["Content-Type", "application/wasm"]])
                });
                edgeConsumedResponse.arrayBuffer().then(
                  () => WebAssembly.compileStreaming(edgeConsumedResponse)
                ).then(
                  () => edgeConsumedResponseError = "resolved",
                  error => edgeConsumedResponseError = error.name
                );
                "queued"
                "#,
            ),
            "queued"
        );
        assert_eq!(
            text(
                &mut runtime,
                r#"
                [
                  edgeCompiledModule instanceof WebAssembly.Module,
                  edgeInstantiatedModule instanceof WebAssembly.Module,
                  edgeInstantiatedInstance instanceof WebAssembly.Instance,
                  edgeResponseBytes,
                  String(edgeCompileError),
                  String(edgeInstantiateError),
                  edgeInvalidMimeError,
                  edgeConsumedResponseError
                ].join("|")
                "#,
            ),
            "true|true|true|0,97,115,109,1,0,0,0|undefined|undefined|TypeError|TypeError"
        );
    }

    #[test]
    fn native_trace_records_api_interactions_without_polluting_window() {
        let mut runtime = EdgeRuntime::new().expect("Edge runtime");
        assert_eq!(crate::trace::proxy_count(&runtime.isolate), 0);
        assert_eq!(
            text(
                &mut runtime,
                "Object.getOwnPropertyNames(window).length.toString()",
            ),
            "1232"
        );
        assert_eq!(
            text(&mut runtime, "document.createElement('span').tagName",),
            "SPAN"
        );
        assert!(runtime.proxy_trace().is_empty());
        assert_eq!(crate::trace::proxy_count(&runtime.isolate), 0);
        runtime.enable_proxy_trace().expect("enable native trace");
        assert_eq!(crate::trace::proxy_count(&runtime.isolate), 0);
        assert!(crate::trace::native_callback_count(&runtime.isolate) > 1000);
        let native_shape = text(
            &mut runtime,
            r#"
            (() => {
              const native = (fn, name) =>
                Function.prototype.toString.call(fn) ===
                  `function ${name}() { [native code] }` &&
                fn.toString() === `function ${name}() { [native code] }` &&
                fn.toString.toString() ===
                  "function toString() { [native code] }" &&
                String(fn) === `function ${name}() { [native code] }` &&
                Object.prototype.toString.call(fn) === "[object Function]" &&
                Object.getPrototypeOf(fn) === Function.prototype;
              const dataDescriptor = (owner, key, fn) => {
                const descriptor =
                  Object.getOwnPropertyDescriptor(owner, key);
                return "value" in descriptor &&
                  descriptor.value === fn &&
                  descriptor.enumerable ===
                    (owner !== window) &&
                  descriptor.configurable === true &&
                  descriptor.writable === true &&
                  descriptor.value.toString() === fn.toString();
              };
              const userAgent =
                Object.getOwnPropertyDescriptor(
                  Navigator.prototype,
                  "userAgent"
                );
              const href =
                Object.getOwnPropertyDescriptor(URL.prototype, "href");
              return [
                native(URL, "URL"),
                native(Document.prototype.createElement, "createElement"),
                native(Element.prototype.getAttribute, "getAttribute"),
                native(Element.prototype.setAttribute, "setAttribute"),
                native(Function.prototype.toString, "toString"),
                native(Object.getPrototypeOf, "getPrototypeOf"),
                native(Reflect.ownKeys, "ownKeys"),
                Object.getOwnPropertyNames(URL).join(",") ===
                  "length,name,prototype,canParse,parse,createObjectURL,revokeObjectURL",
                dataDescriptor(window, "URL", URL),
                dataDescriptor(
                  Document.prototype,
                  "createElement",
                  Document.prototype.createElement
                ),
                dataDescriptor(
                  Element.prototype,
                  "getAttribute",
                  Element.prototype.getAttribute
                ),
                !("value" in userAgent),
                userAgent.enumerable,
                userAgent.configurable,
                native(userAgent.get, "get userAgent"),
                userAgent.set === undefined,
                !("value" in href),
                href.enumerable,
                href.configurable,
                native(href.get, "get href"),
                native(href.set, "set href"),
                URL.prototype.constructor === URL
              ].every(Boolean);
            })()
            "#,
        );
        assert_eq!(native_shape, "true");

        let native_aggregate = text(
            &mut runtime,
            r#"
            (() => {
              function eA(action, cleanup) {
                try {
                  throw action(), Error("");
                } catch (error) {
                  return (error.name + error.message).length;
                } finally {
                  cleanup && cleanup();
                }
              }
              function ZA(api, selected) {
                if (!api) return 0;
                const name = api.name;
                const instance =
                  /^Screen|Navigator$/.test(name) &&
                  window[name.toLowerCase()];
                const prototype =
                  "prototype" in api
                    ? api.prototype
                    : Object.getPrototypeOf(api);
                const contribution =
                  (selected && selected.length
                    ? selected
                    : Object.getOwnPropertyNames(prototype)
                  ).reduce((sum, key) => {
                    let fn;
                    try {
                      const descriptor =
                        Object.getOwnPropertyDescriptor(prototype, key);
                      fn = descriptor && (descriptor.value || descriptor.get);
                    } catch (_) {
                      fn = null;
                    }
                    if (!fn) return sum;
                    const errors = [
                      eA(() => fn().catch(() => {})),
                      eA(() => { throw Error(Object.create(fn)); }),
                      eA(() => { fn.arguments; fn.caller; }),
                      eA(() => {
                        fn.toString.arguments;
                        fn.toString.caller;
                      }),
                      eA(() => Object.create(fn).toString())
                    ];
                    if (fn.name === "toString") {
                      const parent = Object.getPrototypeOf(fn);
                      errors.push(
                        eA(
                          () => Object
                            .setPrototypeOf(fn, Object.create(fn))
                            .toString(),
                          () => Object.setPrototypeOf(fn, parent)
                        ),
                        eA(
                          () => Reflect
                            .setPrototypeOf(fn, Object.create(fn)),
                          () => Object.setPrototypeOf(fn, parent)
                        )
                      );
                    }
                    return sum +
                      (instance
                        ? typeof Object.getOwnPropertyDescriptor(
                            instance,
                            key
                          ).length
                        : 0) +
                      Object.getOwnPropertyNames(fn).length +
                      Number(errors.join("")) +
                      (fn.toString() + fn.toString.toString()).length;
                  }, 0);
                return (
                  (instance
                    ? Object.getOwnPropertyNames(instance).length
                    : 0) +
                  contribution
                );
              }
              return [
                ZA(Function, ["call", "apply", "toString"]),
                ZA(
                  Document,
                  ["createElement", "createComment", "createEvent"]
                ),
                ZA(Element, ["getAttribute", "setAttribute"]),
                ZA(URL, ["href", "toString"])
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            native_aggregate,
            "719038543000278|831342943354|554228628902|277391428685838"
        );

        if std::env::var_os("EDGE_FULL_PROXY_AUDIT").is_some() {
            let proxied_surface = native_text_without_trace_entries(
                &mut runtime,
                r#"
            (() => {
              const hash = text => {
                let value = 2166136261;
                for (let index = 0; index < text.length; index += 1) {
                  value = Math.imul(
                    value ^ text.charCodeAt(index),
                    16777619
                  );
                }
                return (value >>> 0).toString(16).padStart(8, "0");
              };
              const keyName = key =>
                typeof key === "symbol"
                  ? "@@" + (key.description || "")
                  : key;
              const functionValue = value => {
                if (typeof value === "function") {
                  return (
                    value.name + "/" + value.length + "/" +
                    Function.prototype.toString.call(value)
                  );
                }
                if (value === null) return "null";
                const kind = typeof value;
                return (
                  kind +
                  (kind !== "object" && kind !== "function"
                    ? "/" + String(value)
                    : "")
                );
              };
              const descriptor = (object, key) => {
                const value =
                  Object.getOwnPropertyDescriptor(object, key);
                return (
                  keyName(key) + ":" +
                  ("value" in value ? "d" : "a") + ":" +
                  Number(value.enumerable) +
                  Number(value.configurable) +
                  Number(Boolean(value.writable)) + ":" +
                  functionValue(value.value) + ":" +
                  functionValue(value.get) + ":" +
                  functionValue(value.set)
                );
              };
              const windowDescriptor = (object, key) => {
                const value =
                  Object.getOwnPropertyDescriptor(object, key);
                return (
                  keyName(key) + ":" +
                  ("value" in value ? "d" : "a") + ":" +
                  Number(value.enumerable) +
                  Number(value.configurable) +
                  Number(Boolean(value.writable)) + ":" +
                  Number(Boolean(value.get)) +
                  Number(Boolean(value.set))
                );
              };
              const names = Object.getOwnPropertyNames(globalThis);
              const windowDescriptors = names.map(name =>
                windowDescriptor(globalThis, name)
              );
              const constructors = [];
              const functions = [];
              const objects = [];
              const primitives = [];
              for (const name of names) {
                let value;
                try {
                  value = globalThis[name];
                } catch (_) {
                  continue;
                }
                if (typeof value === "function") {
                  functions.push(name + "\t" + functionValue(value));
                  if (
                    value.prototype &&
                    typeof value.prototype === "object"
                  ) {
                    const prototype = value.prototype;
                    const parent = Object.getPrototypeOf(prototype);
                    const parentName =
                      parent && parent.constructor &&
                      parent.constructor.name || "";
                    constructors.push(
                      name + "\t" + value.name + "/" + value.length +
                      "\t" + parentName + "\t" +
                      Reflect.ownKeys(prototype)
                        .map(key => descriptor(prototype, key))
                        .join("\u001e")
                    );
                  }
                } else if (
                  value !== null &&
                  typeof value === "object"
                ) {
                  const parent = Object.getPrototypeOf(value);
                  const parentName =
                    parent && parent.constructor &&
                    parent.constructor.name || "";
                  objects.push(
                    name + "\t" +
                    Object.prototype.toString.call(value) + "\t" +
                    parentName + "\t" +
                    Reflect.ownKeys(value)
                      .map(key => descriptor(value, key))
                      .join("\u001e")
                  );
                } else {
                  primitives.push(
                    name + "\t" + functionValue(value)
                  );
                }
              }
              return [
                names.length,
                hash(names.join("\u001f")),
                hash(windowDescriptors.join("\u001f")),
                constructors.length,
                hash(constructors.join("\n")),
                functions.length,
                hash(functions.join("\n")),
                objects.length,
                hash(objects.join("\n")),
                primitives.length,
                hash(primitives.join("\n"))
              ].join("|");
            })()
                "#,
            );
            assert_eq!(
                proxied_surface,
                "1232|60594b80|946e759f|961|e1f42f61|1024|a8b3e550|53|59942bf5|155|907ccdc7"
            );
        }
        runtime.clear_proxy_trace();

        let result = text(
            &mut runtime,
            r#"
            const tracedDiv = document.createElement("div");
            tracedDiv.id = "alpha";
            tracedDiv.setAttribute("data-edge", "trace");
            const firstAgent = navigator.userAgent;
            const secondAgent = navigator.userAgent;
            const tracedUrl = new URL("/proxy", location.href);
            [
              firstAgent === secondAgent,
              tracedDiv.getAttribute("data-edge"),
              tracedUrl.href,
              Object.getPrototypeOf(DOMException) === Function.prototype,
              Object.getPrototypeOf(Window.prototype)[Symbol.toStringTag],
              Function.prototype.toString.call(URL),
              URL.toString(),
              URL.toString.toString(),
              Object.getOwnPropertyNames(window).length
            ].join("|")
            "#,
        );
        assert_eq!(
            result,
            "true|trace|https://sandbox.test/proxy|true|WindowProperties|function URL() { [native code] }|function URL() { [native code] }|function toString() { [native code] }|1232"
        );

        let object_shape = text(
            &mut runtime,
            r#"
            (() => {
              const element = document.createElement("div");
              element.edgeTraceOwn = 1;
              const enumerated = [];
              for (const key in element) {
                if (key === "edgeTraceOwn") enumerated.push(key);
              }
              const ownShape = [
                "edgeTraceOwn" in element,
                Object.keys(element).includes("edgeTraceOwn"),
                Reflect.ownKeys(element).includes("edgeTraceOwn"),
                Object.getOwnPropertyDescriptor(
                  element,
                  "edgeTraceOwn"
                ).value,
                enumerated.join(",")
              ].join(",");
              const deleted = delete element.edgeTraceOwn;
              const url = new URL("/shape", location.href);
              return [
                Object.getPrototypeOf(window) === Window.prototype,
                Object.getPrototypeOf(document) ===
                  HTMLDocument.prototype,
                document instanceof HTMLDocument,
                document instanceof Document,
                navigator instanceof Navigator,
                element instanceof HTMLDivElement,
                element instanceof HTMLElement,
                element instanceof Element,
                element instanceof Node,
                element instanceof EventTarget,
                url instanceof URL,
                Object.prototype.toString.call(document),
                Object.prototype.toString.call(element),
                Object.prototype.toString.call(url),
                element.constructor === HTMLDivElement,
                URL.prototype.constructor === URL,
                self === window,
                top === window,
                parent === window,
                frames === window,
                ownShape,
                deleted,
                "edgeTraceOwn" in element
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            object_shape,
            "true|true|true|true|true|true|true|true|true|true|true|[object HTMLDocument]|[object HTMLDivElement]|[object URL]|true|true|true|true|true|true|true,true,true,1,edgeTraceOwn|true|false"
        );

        let exotic_shape = text(
            &mut runtime,
            r#"
            (() => {
              const names =
                Object.getOwnPropertyNames(URL).slice(0, 3);
              const bytes = new Uint8Array([1, 2, 3]);
              const map = new Map([["a", 1]]);
              const set = new Set([2, 3]);
              const date = new Date(0);
              const regexp = /edge/gi;
              const promise = Promise.resolve(7);
              return [
                Array.isArray(names),
                Object.prototype.toString.call(names),
                [...names].join(","),
                bytes instanceof Uint8Array,
                ArrayBuffer.isView(bytes),
                Object.prototype.toString.call(bytes),
                bytes.length,
                bytes[1],
                map instanceof Map,
                Object.prototype.toString.call(map),
                map.size,
                map.get("a"),
                [...map][0].join(","),
                set instanceof Set,
                set.has(3),
                [...set].join(","),
                date instanceof Date,
                Object.prototype.toString.call(date),
                date.getTime(),
                +date,
                regexp instanceof RegExp,
                Object.prototype.toString.call(regexp),
                String(regexp),
                promise instanceof Promise,
                Object.prototype.toString.call(promise),
                typeof promise.then
              ].join("|");
            })()
            "#,
        );
        assert_eq!(
            exotic_shape,
            "true|[object Array]|length,name,prototype|true|true|[object Uint8Array]|3|2|true|[object Map]|1|1|a,1|true|true|2,3|true|[object Date]|0|0|true|[object RegExp]|/edge/gi|true|[object Promise]|function"
        );

        let trace = runtime.proxy_trace();
        assert!(trace.iter().any(|entry| {
            entry.operation == "get" && entry.api == "window.navigator.userAgent"
        }));
        assert_eq!(
            trace
                .iter()
                .filter(|entry| entry.api == "window.navigator.userAgent")
                .count(),
            2
        );
        assert!(trace.iter().any(|entry| {
            entry.operation == "get" && entry.api == "Document.prototype.createElement"
        }));
        assert!(trace.iter().any(|entry| {
            entry.operation == "call"
                && entry.api.ends_with(".createElement")
                && entry.arguments == "\"div\""
        }));
        assert!(trace.iter().any(|entry| {
            entry.operation == "set"
                && entry.api.ends_with(".createElement().id")
                && entry.arguments == "\"alpha\""
        }));
        assert!(trace.iter().any(|entry| {
            entry.operation == "get" && entry.api == "Element.prototype.setAttribute"
        }));
        assert!(
            trace
                .iter()
                .any(|entry| entry.operation == "call" && entry.api.ends_with(".setAttribute"))
        );
        assert!(trace.iter().any(|entry| {
            entry.operation == "call" && entry.api == "Object.getOwnPropertyDescriptor"
        }));
        assert!(
            trace
                .iter()
                .any(|entry| { entry.operation == "call" && entry.api == "Reflect.ownKeys" })
        );
        assert!(trace.iter().any(|entry| {
            entry.operation == "construct"
                && entry.api == "URL"
                && entry.arguments == "\"/proxy\",\"https://sandbox.test/\""
        }));

        runtime.clear_proxy_trace();
        assert_eq!(
            text(
                &mut runtime,
                "globalThis.tracedPersistentNavigator = navigator; 'saved'",
            ),
            "saved"
        );
        assert_eq!(
            text(
                &mut runtime,
                "tracedPersistentNavigator.userAgent.includes('Chrome/150') &&
                 !tracedPersistentNavigator.userAgent.includes('Edg/') &&
                 !tracedPersistentNavigator.userAgent.includes('HeadlessChrome/')",
            ),
            "true"
        );
        assert!(
            runtime
                .evaluate("throw new Error('trace recovery')")
                .expect_err("trace exception")
                .contains("trace recovery")
        );
        assert_eq!(text(&mut runtime, "document.URL"), "https://sandbox.test/");
        assert!(
            runtime
                .proxy_trace()
                .iter()
                .any(|entry| { entry.operation == "get" && entry.api == "window.document.URL" })
        );
        assert_eq!(
            text(&mut runtime, "delete globalThis.tracedPersistentNavigator",),
            "true"
        );

        runtime.disable_proxy_trace();
        runtime.clear_proxy_trace();
        assert!(runtime.proxy_trace().is_empty());
        assert_eq!(
            text(
                &mut runtime,
                "Object.getOwnPropertyNames(window).length.toString()",
            ),
            "1232"
        );
        assert!(runtime.proxy_trace().is_empty());
    }

    #[test]
    fn top_level_promises_return_their_fulfillment_or_rejection() {
        let mut runtime = EdgeRuntime::new().expect("Edge runtime");
        assert_eq!(text(&mut runtime, "Promise.resolve(42)"), "42");
        assert_eq!(
            text(
                &mut runtime,
                "new Promise(resolve => setTimeout(() => resolve('timer-value'), 1))",
            ),
            "timer-value"
        );

        let rejection = runtime
            .evaluate("Promise.reject(new TypeError('promise-failure'))")
            .expect_err("top-level rejection");
        assert!(rejection.contains("top-level Promise rejected"));
        assert!(rejection.contains("promise-failure"));

        let pending = runtime
            .evaluate("new Promise(() => {})")
            .expect_err("permanently pending promise");
        assert!(pending.contains("remained pending"));
    }
}
