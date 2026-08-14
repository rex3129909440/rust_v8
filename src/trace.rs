use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const MAX_TRACE_LABEL_CHARACTERS: usize = 512;
const MAX_TRACE_PROPERTY_CHARACTERS: usize = 256;
const MAX_TRACE_ARRAY_DEPTH: usize = 3;
// Trace values are an opt-in diagnostic surface.  Keep label/property limits
// small, but retain enough array items to make a complete browser-profile
// aggregate observable when tracing is enabled.  The previous 32-item/512-
// character limits silently hid the tail of large argument arrays.
const MAX_TRACE_ARRAY_ITEMS: usize = 512;
const MAX_TRACE_ARRAY_CHARACTERS: usize = 65_536;
const MAX_TRACE_OBJECT_PROPERTIES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TraceEntry {
    pub sequence: u64,
    pub operation: String,
    pub api: String,
    pub receiver: String,
    pub arguments: String,
    pub result: String,
}

impl std::fmt::Display for TraceEntry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "TRACE\t{}\t{}\t{}\treceiver={}\targs={}\tresult={}",
            self.sequence, self.operation, self.api, self.receiver, self.arguments, self.result
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeCallbackKind {
    Getter,
    Setter,
    Function,
    Constructor,
}

struct NativeCallbackRecord {
    original: v8::FunctionCallback,
    original_data: Option<v8::Global<v8::Value>>,
    active: Arc<AtomicBool>,
    kind: NativeCallbackKind,
    length: i32,
    summary_kind: NativeSummaryKind,
    member: String,
    label: std::cell::RefCell<String>,
    pending_exception: std::cell::RefCell<Option<String>>,
}

thread_local! {
    static CURRENT_NATIVE_CALLBACK: std::cell::Cell<*const NativeCallbackRecord> =
        const { std::cell::Cell::new(std::ptr::null()) };
    static CURRENT_NATIVE_ARGUMENT_COUNT: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    static CURRENT_NATIVE_IS_CONSTRUCT_CALL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

struct CurrentNativeCallbackGuard {
    previous: *const NativeCallbackRecord,
    previous_argument_count: i32,
    previous_is_construct_call: bool,
}

impl CurrentNativeCallbackGuard {
    fn enter(
        callback: &NativeCallbackRecord,
        argument_count: i32,
        is_construct_call: bool,
    ) -> Self {
        let pointer = callback as *const NativeCallbackRecord;
        let previous = CURRENT_NATIVE_CALLBACK.with(|current| current.replace(pointer));
        let previous_argument_count =
            CURRENT_NATIVE_ARGUMENT_COUNT.with(|current| current.replace(argument_count));
        let previous_is_construct_call =
            CURRENT_NATIVE_IS_CONSTRUCT_CALL.with(|current| current.replace(is_construct_call));
        Self {
            previous,
            previous_argument_count,
            previous_is_construct_call,
        }
    }
}

impl Drop for CurrentNativeCallbackGuard {
    fn drop(&mut self) {
        CURRENT_NATIVE_CALLBACK.with(|current| current.set(self.previous));
        CURRENT_NATIVE_ARGUMENT_COUNT.with(|current| current.set(self.previous_argument_count));
        CURRENT_NATIVE_IS_CONSTRUCT_CALL
            .with(|current| current.set(self.previous_is_construct_call));
    }
}

pub(crate) fn current_constructor_name() -> Option<String> {
    CURRENT_NATIVE_CALLBACK.with(|current| {
        let pointer = current.get();
        if pointer.is_null() {
            return None;
        }
        // SAFETY: the pointer is installed only while the native callback's
        // boxed record is alive in the isolate-owned trace state.
        let callback = unsafe { &*pointer };
        (callback.kind == NativeCallbackKind::Constructor).then(|| callback.member.clone())
    })
}

pub(crate) fn current_constructor_missing_arguments() -> Option<(String, i32, i32)> {
    CURRENT_NATIVE_CALLBACK.with(|current| {
        let pointer = current.get();
        if pointer.is_null() || !CURRENT_NATIVE_IS_CONSTRUCT_CALL.with(std::cell::Cell::get) {
            return None;
        }
        // SAFETY: the pointer is installed only while the native callback's
        // boxed record is alive in the isolate-owned trace state.
        let callback = unsafe { &*pointer };
        let present = CURRENT_NATIVE_ARGUMENT_COUNT.with(std::cell::Cell::get);
        (callback.kind == NativeCallbackKind::Constructor && present < callback.length)
            .then(|| (callback.member.clone(), callback.length, present))
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeSummaryKind {
    Standard,
    Json,
}

struct NativeFunctionRecord {
    function: v8::Global<v8::Function>,
    callback: *mut NativeCallbackRecord,
}

struct NativePrototypeRecord {
    active: Arc<AtomicBool>,
    label: String,
}

struct NativeValueLabel {
    value: v8::Global<v8::Value>,
    label: String,
}

struct JsonIntrinsicMethodRecord {
    original: v8::Global<v8::Function>,
    wrapper: Option<v8::Global<v8::Function>>,
}

struct JsonIntrinsicRealmRecord {
    json: v8::Global<v8::Object>,
    label: String,
    parse: JsonIntrinsicMethodRecord,
    stringify: JsonIntrinsicMethodRecord,
}

#[derive(Clone, Copy)]
struct CompactTraceEntry {
    sequence: u64,
    operation: u32,
    api: u32,
    receiver: u32,
    arguments: u32,
    result: u32,
}

struct NativeTraceState {
    enabled: bool,
    recording: bool,
    interceptor_depth: u32,
    sequence: u64,
    entries: Vec<CompactTraceEntry>,
    trace_strings: Vec<String>,
    trace_string_ids: HashMap<String, u32>,
    excluded_apis: HashSet<String>,
    excluded_api_prefixes: Vec<String>,
    active: Arc<AtomicBool>,
    callbacks: Vec<std::pin::Pin<Box<NativeCallbackRecord>>>,
    prototypes: Vec<std::pin::Pin<Box<NativePrototypeRecord>>>,
    functions: Vec<NativeFunctionRecord>,
    functions_by_hash: HashMap<i32, Vec<usize>>,
    labels: Vec<NativeValueLabel>,
    labels_by_hash: HashMap<i32, Vec<usize>>,
    json_realms: Vec<JsonIntrinsicRealmRecord>,
}

impl Default for NativeTraceState {
    fn default() -> Self {
        Self {
            enabled: false,
            recording: false,
            interceptor_depth: 0,
            sequence: 0,
            entries: Vec::new(),
            trace_strings: Vec::new(),
            trace_string_ids: HashMap::new(),
            excluded_apis: HashSet::new(),
            excluded_api_prefixes: Vec::new(),
            active: Arc::new(AtomicBool::new(false)),
            callbacks: Vec::new(),
            prototypes: Vec::new(),
            functions: Vec::new(),
            functions_by_hash: HashMap::new(),
            labels: Vec::new(),
            labels_by_hash: HashMap::new(),
            json_realms: Vec::new(),
        }
    }
}

impl NativeTraceState {
    fn intern_trace_string(&mut self, value: String) -> u32 {
        if let Some(id) = self.trace_string_ids.get(&value) {
            return *id;
        }
        let id = u32::try_from(self.trace_strings.len()).expect("trace string table overflow");
        self.trace_strings.push(value.clone());
        self.trace_string_ids.insert(value, id);
        id
    }

    fn trace_string(&self, id: u32) -> &str {
        self.trace_strings
            .get(id as usize)
            .map(String::as_str)
            .unwrap_or_default()
    }

    fn expand_trace_entry(&self, entry: CompactTraceEntry) -> TraceEntry {
        TraceEntry {
            sequence: entry.sequence,
            operation: self.trace_string(entry.operation).to_owned(),
            api: self.trace_string(entry.api).to_owned(),
            receiver: self.trace_string(entry.receiver).to_owned(),
            arguments: self.trace_string(entry.arguments).to_owned(),
            result: self.trace_string(entry.result).to_owned(),
        }
    }

    #[inline]
    fn excludes_api(&self, api: &str) -> bool {
        self.excluded_apis.contains(api)
            || self
                .excluded_api_prefixes
                .iter()
                .any(|prefix| api.starts_with(prefix))
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NativeTraceState::default());
}

pub(crate) fn enable(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let state = scope
        .get_slot_mut::<NativeTraceState>()
        .ok_or_else(|| "native trace state was not prepared".to_owned())?;
    state.enabled = true;
    state.recording = false;
    state.active.store(false, Ordering::Relaxed);
    Ok(())
}

pub(crate) fn disable(isolate: &mut v8::OwnedIsolate) {
    if let Some(state) = isolate.get_slot_mut::<NativeTraceState>() {
        state.enabled = false;
        state.recording = false;
        state.active.store(false, Ordering::Relaxed);
    }
}

pub(crate) fn clear(isolate: &mut v8::OwnedIsolate) {
    if let Some(state) = isolate.get_slot_mut::<NativeTraceState>() {
        state.sequence = 0;
        state.entries.clear();
        state.trace_strings.clear();
        state.trace_string_ids.clear();
    }
}

pub(crate) fn set_excluded_apis(
    isolate: &mut v8::OwnedIsolate,
    rules: &[String],
) -> Result<(), String> {
    let mut exact = HashSet::new();
    let mut prefixes = HashSet::new();
    for rule in rules {
        if rule.is_empty() {
            return Err("native trace exclusion cannot be empty".to_owned());
        }
        if rule.chars().count() > MAX_TRACE_LABEL_CHARACTERS {
            return Err(format!(
                "native trace exclusion exceeds {MAX_TRACE_LABEL_CHARACTERS} characters"
            ));
        }
        if let Some(prefix) = rule.strip_suffix('*') {
            prefixes.insert(prefix.to_owned());
        } else {
            exact.insert(rule.clone());
        }
    }
    let mut prefixes = prefixes.into_iter().collect::<Vec<_>>();
    prefixes.sort_unstable();
    let state = isolate
        .get_slot_mut::<NativeTraceState>()
        .ok_or_else(|| "native trace state was not prepared".to_owned())?;
    state.excluded_apis = exact;
    state.excluded_api_prefixes = prefixes;
    Ok(())
}

pub(crate) fn entries(isolate: &v8::OwnedIsolate) -> Vec<TraceEntry> {
    isolate
        .get_slot::<NativeTraceState>()
        .map(|state| {
            state
                .entries
                .iter()
                .copied()
                .map(|entry| state.expand_trace_entry(entry))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn matching_entries(isolate: &v8::OwnedIsolate, needle: &str) -> Vec<TraceEntry> {
    let Some(state) = isolate.get_slot::<NativeTraceState>() else {
        return Vec::new();
    };
    if let Some((start, end)) = parse_sequence_range(needle) {
        return state
            .entries
            .iter()
            .copied()
            .filter(|entry| entry.sequence >= start && entry.sequence <= end)
            .map(|entry| state.expand_trace_entry(entry))
            .collect();
    }
    state
        .entries
        .iter()
        .copied()
        .filter(|entry| {
            [
                entry.operation,
                entry.api,
                entry.receiver,
                entry.arguments,
                entry.result,
            ]
            .into_iter()
            .any(|id| state.trace_string(id).contains(needle))
        })
        .map(|entry| state.expand_trace_entry(entry))
        .collect()
}

fn parse_sequence_range(needle: &str) -> Option<(u64, u64)> {
    let range = needle.strip_prefix("@sequence:")?;
    let (start, end) = range.split_once("..")?;
    let start = start.parse::<u64>().ok()?;
    let end = end.parse::<u64>().ok()?;
    (start <= end).then_some((start, end))
}

#[cfg(test)]
pub(crate) fn proxy_count(_: &v8::OwnedIsolate) -> usize {
    0
}

#[cfg(test)]
pub(crate) fn native_callback_count(isolate: &v8::OwnedIsolate) -> usize {
    isolate
        .get_slot::<NativeTraceState>()
        .map(|state| state.callbacks.len())
        .unwrap_or_default()
}

pub(crate) fn is_enabled(scope: &v8::PinScope<'_, '_>) -> bool {
    scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.enabled)
}

pub(crate) fn start_recording(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let state = scope
        .get_slot_mut::<NativeTraceState>()
        .ok_or_else(|| "native trace state was not prepared".to_owned())?;
    state.recording = true;
    state.active.store(true, Ordering::Relaxed);
    Ok(())
}

pub(crate) fn stop_recording(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
        state.recording = false;
        state.active.store(false, Ordering::Relaxed);
    }
}

pub(crate) struct UserExecutionGuard {
    depth: *mut u32,
    previous: u32,
}

impl Drop for UserExecutionGuard {
    fn drop(&mut self) {
        if !self.depth.is_null() {
            // SAFETY: the trace state isolate slot is not replaced while the
            // runtime is alive, so the depth field remains at this address.
            unsafe {
                *self.depth = self.previous;
            }
        }
    }
}

pub(crate) fn enter_user_execution(scope: &mut v8::PinScope<'_, '_>) -> Option<UserExecutionGuard> {
    let state = scope.get_slot_mut::<NativeTraceState>()?;
    let previous = state.interceptor_depth;
    state.interceptor_depth = 0;
    Some(UserExecutionGuard {
        depth: &mut state.interceptor_depth,
        previous,
    })
}

pub(crate) fn interceptor_data<'s>(
    scope: &v8::PinScope<'s, '_, ()>,
) -> Option<v8::Local<'s, v8::Value>> {
    let pointer = scope
        .get_slot::<NativeTraceState>()
        .map(|state| Arc::as_ptr(&state.active).cast_mut().cast())?;
    Some(v8::External::new(scope, pointer).into())
}

#[inline(always)]
pub(crate) fn interceptor_is_active(arguments: &v8::PropertyCallbackArguments<'_>) -> bool {
    let Ok(external) = v8::Local::<v8::External>::try_from(arguments.data()) else {
        return false;
    };
    let pointer = external.value().cast::<AtomicBool>();
    !pointer.is_null()
        // SAFETY: the External points into an Arc owned for the isolate lifetime.
        && unsafe { &*pointer }.load(Ordering::Relaxed)
}

pub(crate) fn record_named_intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
    operation: &str,
    owner: Option<&str>,
    key: v8::Local<'_, v8::Name>,
    argument: Option<v8::Local<'_, v8::Value>>,
) {
    if !interceptor_is_active(arguments) {
        return;
    }
    let owner = owner
        .map(str::to_owned)
        .or_else(|| native_label_for_value(scope, arguments.holder().into()))
        .unwrap_or_else(|| "object".to_owned());
    let property = property_name(scope, key.into());
    let api = format!("{owner}.{property}");
    let argument = argument
        .map(|value| summarize(scope, value))
        .unwrap_or_default();
    record_entry(
        scope,
        operation,
        api,
        owner,
        argument,
        "deferred".to_owned(),
    );
}

pub(crate) fn record_named_native_intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
    operation: &str,
    key: v8::Local<'_, v8::Name>,
    argument: Option<v8::Local<'_, v8::Value>>,
) {
    if !scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.recording && state.interceptor_depth == 0)
    {
        return;
    }
    let owner = native_label_for_value(scope, arguments.holder().into())
        .unwrap_or_else(|| "object".to_owned());
    let property = property_name(scope, key.into());
    let api = format!("{owner}.{property}");
    let argument = argument
        .map(|value| summarize(scope, value))
        .unwrap_or_default();
    record_entry(
        scope,
        operation,
        api,
        owner,
        argument,
        "deferred".to_owned(),
    );
}

pub(crate) fn record_indexed_intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
    operation: &str,
    owner: Option<&str>,
    index: u32,
    argument: Option<v8::Local<'_, v8::Value>>,
) {
    if !interceptor_is_active(arguments) {
        return;
    }
    let owner = owner
        .map(str::to_owned)
        .or_else(|| native_label_for_value(scope, arguments.holder().into()))
        .unwrap_or_else(|| "object".to_owned());
    let api = format!("{owner}[{index}]");
    let argument = argument
        .map(|value| summarize(scope, value))
        .unwrap_or_default();
    record_entry(
        scope,
        operation,
        api,
        owner,
        argument,
        "deferred".to_owned(),
    );
}

pub(crate) fn record_indexed_native_intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
    operation: &str,
    index: u32,
    argument: Option<v8::Local<'_, v8::Value>>,
) {
    if !scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.recording && state.interceptor_depth == 0)
    {
        return;
    }
    let owner = native_label_for_value(scope, arguments.holder().into())
        .unwrap_or_else(|| "object".to_owned());
    let api = format!("{owner}[{index}]");
    let argument = argument
        .map(|value| summarize(scope, value))
        .unwrap_or_default();
    record_entry(
        scope,
        operation,
        api,
        owner,
        argument,
        "deferred".to_owned(),
    );
}

pub(crate) fn record_enumerate_intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
    owner: Option<&str>,
) {
    if !interceptor_is_active(arguments) {
        return;
    }
    let owner = owner
        .map(str::to_owned)
        .or_else(|| native_label_for_value(scope, arguments.holder().into()))
        .unwrap_or_else(|| "object".to_owned());
    record_entry(
        scope,
        "ownKeys",
        owner.clone(),
        owner,
        String::new(),
        "deferred".to_owned(),
    );
}

pub(crate) fn record_native_enumeration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
) {
    if !scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.recording && state.interceptor_depth == 0)
    {
        return;
    }
    let owner = native_label_for_value(scope, arguments.holder().into())
        .unwrap_or_else(|| "object".to_owned());
    record_entry(
        scope,
        "ownKeys",
        owner.clone(),
        owner,
        String::new(),
        "deferred".to_owned(),
    );
}

pub(crate) fn create_native_function<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    length: i32,
    constructor_behavior: v8::ConstructorBehavior,
    original: v8::FunctionCallback,
    kind: NativeCallbackKind,
) -> Result<v8::Local<'s, v8::Function>, String> {
    create_native_function_raw(
        scope,
        name,
        length,
        constructor_behavior,
        original,
        kind,
        NativeSummaryKind::Standard,
        None,
    )
}

pub(crate) fn create_native_function_with_data<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    length: i32,
    constructor_behavior: v8::ConstructorBehavior,
    original: v8::FunctionCallback,
    kind: NativeCallbackKind,
    data: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let data = v8::Global::new(scope, data);
    create_native_function_raw(
        scope,
        name,
        length,
        constructor_behavior,
        original,
        kind,
        NativeSummaryKind::Standard,
        Some(data),
    )
}

fn create_native_function_raw<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    length: i32,
    constructor_behavior: v8::ConstructorBehavior,
    original: v8::FunctionCallback,
    kind: NativeCallbackKind,
    summary_kind: NativeSummaryKind,
    original_data: Option<v8::Global<v8::Value>>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let is_constructor = matches!(&constructor_behavior, v8::ConstructorBehavior::Allow);
    let active = scope
        .get_slot::<NativeTraceState>()
        .ok_or_else(|| "native trace state was not prepared".to_owned())?
        .active
        .clone();
    let member = name
        .strip_prefix("get ")
        .or_else(|| name.strip_prefix("set "))
        .unwrap_or(name)
        .to_owned();
    let record = Box::pin(NativeCallbackRecord {
        original,
        original_data,
        active: active.clone(),
        kind,
        length,
        summary_kind,
        member,
        label: std::cell::RefCell::new(name.to_owned()),
        pending_exception: std::cell::RefCell::new(None),
    });
    let callback = (&*record as *const NativeCallbackRecord).cast_mut();
    let data = v8::External::new(scope, callback.cast());
    let mut prototype_record = None;
    let function = if is_constructor {
        let template = v8::FunctionTemplate::builder_raw(native_callback_trampoline)
            .data(data.into())
            .length(length)
            .constructor_behavior(constructor_behavior)
            .build(scope);
        let prototype = Box::pin(NativePrototypeRecord {
            active: active.clone(),
            label: format!("{name}.prototype"),
        });
        let prototype_pointer = (&*prototype as *const NativePrototypeRecord).cast_mut();
        let prototype_data = v8::External::new(scope, prototype_pointer.cast());
        template
            .prototype_template(scope)
            .set_named_property_handler(
                v8::NamedPropertyHandlerConfiguration::new()
                    .getter(native_prototype_getter)
                    .setter(native_prototype_setter)
                    .data(prototype_data.into()),
            );
        prototype_record = Some(prototype);
        template
            .get_function(scope)
            .ok_or_else(|| format!("cannot create native function {name}"))?
    } else {
        v8::Function::builder_raw(native_callback_trampoline)
            .data(data.into())
            .length(length)
            .constructor_behavior(constructor_behavior)
            .build(scope)
            .ok_or_else(|| format!("cannot create native function {name}"))?
    };
    function.set_name(crate::webidl::string(scope, name)?);
    if is_constructor {
        let prototype_key = crate::webidl::string(scope, "prototype")?;
        let prototype = function
            .get(scope, prototype_key.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or_else(|| format!("native constructor {name} has no prototype"))?;
        let global = scope.get_current_context().global(scope);
        let object_key = crate::webidl::string(scope, "Object")?;
        let object_prototype = global
            .get(scope, object_key.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .and_then(|object| object.get(scope, prototype_key.into()))
            .ok_or_else(|| "Object.prototype is unavailable".to_owned())?;
        if prototype.set_prototype(scope, object_prototype) != Some(true) {
            return Err(format!(
                "cannot attach Object.prototype to {name}.prototype"
            ));
        }
        label_native_value(scope, prototype.into(), &format!("{name}.prototype"));
    }

    let hash = function.get_identity_hash().get();
    let saved = v8::Global::new(scope, function);
    let state = scope
        .get_slot_mut::<NativeTraceState>()
        .ok_or_else(|| "native trace state was not prepared".to_owned())?;
    state.callbacks.push(record);
    if let Some(prototype) = prototype_record {
        state.prototypes.push(prototype);
    }
    let index = state.functions.len();
    state.functions.push(NativeFunctionRecord {
        function: saved,
        callback,
    });
    state.functions_by_hash.entry(hash).or_default().push(index);
    label_native_value(scope, function.into(), name);
    Ok(function)
}

fn native_prototype_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    _: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    if !prototype_intercept_enabled(scope, &arguments) {
        return v8::Intercepted::kNo;
    }
    if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
        state.interceptor_depth += 1;
    }
    let is_own = arguments.holder().has_own_property(scope, key) == Some(true);
    if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
        state.interceptor_depth = state.interceptor_depth.saturating_sub(1);
    }
    if is_own {
        record_prototype_intercept(scope, &arguments, "get", key, None);
    }
    v8::Intercepted::kNo
}

fn native_prototype_setter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    _: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    if !prototype_intercept_enabled(scope, &arguments) {
        return v8::Intercepted::kNo;
    }
    if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
        state.interceptor_depth += 1;
    }
    let is_own = arguments.holder().has_own_property(scope, key) == Some(true);
    if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
        state.interceptor_depth = state.interceptor_depth.saturating_sub(1);
    }
    if is_own {
        record_prototype_intercept(scope, &arguments, "set", key, Some(value));
    }
    v8::Intercepted::kNo
}

fn prototype_intercept_enabled(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
) -> bool {
    let Ok(external) = v8::Local::<v8::External>::try_from(arguments.data()) else {
        return false;
    };
    let pointer = external.value().cast::<NativePrototypeRecord>();
    if pointer.is_null()
        // SAFETY: prototype records are boxed for the isolate lifetime.
        || !unsafe { &*pointer }.active.load(Ordering::Relaxed)
    {
        return false;
    }
    scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.recording && state.interceptor_depth == 0)
}

fn record_prototype_intercept(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
    operation: &str,
    key: v8::Local<'_, v8::Name>,
    argument: Option<v8::Local<'_, v8::Value>>,
) {
    let Ok(external) = v8::Local::<v8::External>::try_from(arguments.data()) else {
        return;
    };
    let pointer = external.value().cast::<NativePrototypeRecord>();
    if pointer.is_null() {
        return;
    }
    // SAFETY: prototype records are boxed for the isolate lifetime.
    let prototype = unsafe { &*pointer };
    if !prototype.active.load(Ordering::Relaxed) {
        return;
    }
    let property = property_name(scope, key.into());
    let receiver = native_label_for_value(scope, arguments.holder().into())
        .unwrap_or_else(|| prototype.label.clone());
    let argument = argument
        .map(|value| summarize(scope, value))
        .unwrap_or_default();
    record_entry(
        scope,
        operation,
        format!("{}.{property}", prototype.label),
        receiver,
        argument,
        "deferred".to_owned(),
    );
}

pub(crate) fn native_callback_data<'s>(
    scope: &v8::PinScope<'s, '_>,
    arguments: &v8::FunctionCallbackArguments<'s>,
) -> v8::Local<'s, v8::Value> {
    let Ok(external) = v8::Local::<v8::External>::try_from(arguments.data()) else {
        return arguments.data();
    };
    let callback = external.value().cast::<NativeCallbackRecord>();
    if callback.is_null() {
        return arguments.data();
    }
    // SAFETY: callers use this only for callbacks created by
    // create_native_function_with_data.
    unsafe { &*callback }
        .original_data
        .as_ref()
        .map(|data| v8::Local::new(scope, data))
        .unwrap_or_else(|| arguments.data())
}

pub(crate) fn install_json_intrinsic_trace(
    scope: &mut v8::PinScope<'_, '_>,
    realm_label: &str,
) -> Result<(), String> {
    let json = current_json_intrinsic(scope)?;
    let existing = scope.get_slot::<NativeTraceState>().and_then(|state| {
        state
            .json_realms
            .iter()
            .position(|record| v8::Local::new(scope, &record.json).strict_equals(json.into()))
    });
    if existing.is_some() {
        return relabel_json_intrinsic_trace(scope, realm_label);
    }
    let parse = json_intrinsic_method(scope, json, "parse")?;
    let stringify = json_intrinsic_method(scope, json, "stringify")?;
    let json_global = v8::Global::new(scope, json);
    let parse = v8::Global::new(scope, parse);
    let stringify = v8::Global::new(scope, stringify);
    let state = scope
        .get_slot_mut::<NativeTraceState>()
        .ok_or_else(|| "native trace state was not prepared".to_owned())?;
    let index = state.json_realms.len();
    state.json_realms.push(JsonIntrinsicRealmRecord {
        json: json_global,
        label: realm_label.to_owned(),
        parse: JsonIntrinsicMethodRecord {
            original: parse,
            wrapper: None,
        },
        stringify: JsonIntrinsicMethodRecord {
            original: stringify,
            wrapper: None,
        },
    });
    label_native_value(scope, json.into(), &format!("{realm_label}.JSON"));
    enable_json_realm(scope, index)?;
    Ok(())
}

pub(crate) fn relabel_json_intrinsic_trace(
    scope: &mut v8::PinScope<'_, '_>,
    realm_label: &str,
) -> Result<(), String> {
    let json = current_json_intrinsic(scope)?;
    let index = scope
        .get_slot::<NativeTraceState>()
        .and_then(|state| {
            state
                .json_realms
                .iter()
                .position(|record| v8::Local::new(scope, &record.json).strict_equals(json.into()))
        })
        .ok_or_else(|| "JSON intrinsic trace realm was not registered".to_owned())?;
    let (parse, stringify) = {
        let state = scope
            .get_slot_mut::<NativeTraceState>()
            .ok_or_else(|| "native trace state was not prepared".to_owned())?;
        let record = state
            .json_realms
            .get_mut(index)
            .ok_or_else(|| "JSON intrinsic trace realm disappeared".to_owned())?;
        record.label = realm_label.to_owned();
        (
            record.parse.wrapper.clone(),
            record.stringify.wrapper.clone(),
        )
    };
    label_native_value(scope, json.into(), &format!("{realm_label}.JSON"));
    for (name, function) in [("parse", parse), ("stringify", stringify)] {
        if let Some(function) = function {
            let function = v8::Local::new(scope, &function);
            relabel_native_function(scope, function, &format!("{realm_label}.JSON.{name}"));
        }
    }
    Ok(())
}

fn current_json_intrinsic<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let global = scope.get_current_context().global(scope);
    let json_key = crate::webidl::string(scope, "JSON")?;
    global
        .get(scope, json_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "JSON intrinsic is unavailable".to_owned())
}

fn json_intrinsic_method<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    json: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let key = crate::webidl::string(scope, name)?;
    json.get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| format!("JSON.{name} is unavailable"))
}

fn enable_json_realm(scope: &mut v8::PinScope<'_, '_>, index: usize) -> Result<(), String> {
    let (json, label, parse_original, parse_wrapper, stringify_original, stringify_wrapper) = {
        let state = scope
            .get_slot::<NativeTraceState>()
            .ok_or_else(|| "native trace state was not prepared".to_owned())?;
        let record = state
            .json_realms
            .get(index)
            .ok_or_else(|| "JSON intrinsic trace realm disappeared".to_owned())?;
        (
            record.json.clone(),
            record.label.clone(),
            record.parse.original.clone(),
            record.parse.wrapper.clone(),
            record.stringify.original.clone(),
            record.stringify.wrapper.clone(),
        )
    };
    let json = v8::Local::new(scope, &json);
    label_native_value(scope, json.into(), &format!("{label}.JSON"));
    let parse = match parse_wrapper {
        Some(wrapper) => v8::Local::new(scope, &wrapper),
        None => {
            let original = v8::Local::new(scope, &parse_original);
            create_json_wrapper(scope, original, &label, "parse", 2)?
        }
    };
    let stringify = match stringify_wrapper {
        Some(wrapper) => v8::Local::new(scope, &wrapper),
        None => {
            let original = v8::Local::new(scope, &stringify_original);
            create_json_wrapper(scope, original, &label, "stringify", 3)?
        }
    };
    define_json_method(scope, json, "parse", parse)?;
    define_json_method(scope, json, "stringify", stringify)?;
    let parse_global = v8::Global::new(scope, parse);
    let stringify_global = v8::Global::new(scope, stringify);
    let state = scope
        .get_slot_mut::<NativeTraceState>()
        .ok_or_else(|| "native trace state was not prepared".to_owned())?;
    let record = state
        .json_realms
        .get_mut(index)
        .ok_or_else(|| "JSON intrinsic trace realm disappeared".to_owned())?;
    if record.parse.wrapper.is_none() {
        record.parse.wrapper = Some(parse_global);
    }
    if record.stringify.wrapper.is_none() {
        record.stringify.wrapper = Some(stringify_global);
    }
    Ok(())
}

fn create_json_wrapper<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    original: v8::Local<'_, v8::Function>,
    realm_label: &str,
    name: &str,
    length: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let original_value: v8::Local<v8::Value> = original.into();
    let original_data = v8::Global::new(scope, original_value);
    let wrapper = create_native_function_raw(
        scope,
        name,
        length,
        v8::ConstructorBehavior::Throw,
        v8::MapFnTo::map_fn_to(forward_json_intrinsic),
        NativeCallbackKind::Function,
        NativeSummaryKind::Json,
        Some(original_data),
    )?;
    relabel_native_function(scope, wrapper, &format!("{realm_label}.JSON.{name}"));
    Ok(wrapper)
}

fn define_json_method(
    scope: &mut v8::PinScope<'_, '_>,
    json: v8::Local<'_, v8::Object>,
    name: &str,
    function: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if json.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot install JSON.{name} trace callback"))
    }
}

fn forward_json_intrinsic(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let original = native_callback_data(scope, &arguments);
    let Ok(original) = v8::Local::<v8::Function>::try_from(original) else {
        crate::webidl::throw_type_error(scope, "JSON intrinsic trace target is unavailable");
        return;
    };
    let receiver: v8::Local<v8::Value> = arguments.this().into();
    let recording = scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.recording);
    if !recording {
        let value = match arguments.length() {
            0 => original.call(scope, receiver, &[]),
            1 => original.call(scope, receiver, &[arguments.get(0)]),
            2 => original.call(scope, receiver, &[arguments.get(0), arguments.get(1)]),
            3 => original.call(
                scope,
                receiver,
                &[arguments.get(0), arguments.get(1), arguments.get(2)],
            ),
            _ => {
                let values = (0..arguments.length())
                    .map(|index| arguments.get(index))
                    .collect::<Vec<_>>();
                original.call(scope, receiver, &values)
            }
        };
        if let Some(value) = value {
            result.set(value);
        }
        return;
    }
    let values = (0..arguments.length())
        .map(|index| arguments.get(index))
        .collect::<Vec<_>>();
    v8::tc_scope!(let try_catch, scope);
    let _user_execution = enter_user_execution(try_catch);
    if let Some(value) = original.call(try_catch, receiver, &values) {
        result.set(value);
        return;
    }
    let exception = try_catch
        .exception()
        .map(|value| summarize_json(try_catch, value))
        .unwrap_or_else(|| "exception".to_owned());
    set_native_callback_exception(try_catch, &arguments, exception);
    let _ = try_catch.rethrow();
}

fn set_native_callback_exception(
    _: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    exception: String,
) {
    let Ok(external) = v8::Local::<v8::External>::try_from(arguments.data()) else {
        return;
    };
    let callback = external.value().cast::<NativeCallbackRecord>();
    if !callback.is_null() {
        // SAFETY: the External points at a callback record retained by the
        // isolate trace state for the lifetime of the native function.
        unsafe { &*callback }
            .pending_exception
            .replace(Some(exception));
    }
}

pub(crate) fn relabel_native_function(
    scope: &mut v8::PinScope<'_, '_>,
    function: v8::Local<'_, v8::Function>,
    label: &str,
) {
    let label = bounded_text(label, MAX_TRACE_LABEL_CHARACTERS);
    let hash = function.get_identity_hash().get();
    let pointer = scope
        .get_slot::<NativeTraceState>()
        .and_then(|state| {
            state
                .functions_by_hash
                .get(&hash)
                .map(|indices| (state, indices))
        })
        .and_then(|(state, indices)| {
            indices
                .iter()
                .filter_map(|index| state.functions.get(*index))
                .find(|record| {
                    v8::Local::new(scope, &record.function).strict_equals(function.into())
                })
                .map(|record| record.callback)
        });
    if let Some(pointer) = pointer {
        // SAFETY: callback records are boxed for the lifetime of the isolate.
        unsafe { &*pointer }.label.replace(label.clone());
    }
    label_native_value(scope, function.into(), &label);
}

pub(crate) fn label_native_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    label: &str,
) {
    let label = bounded_text(label, MAX_TRACE_LABEL_CHARACTERS);
    let Some(hash) = value_identity_hash(value) else {
        return;
    };
    let existing = scope.get_slot::<NativeTraceState>().and_then(|state| {
        state.labels_by_hash.get(&hash).and_then(|indices| {
            indices.iter().copied().find(|index| {
                state
                    .labels
                    .get(*index)
                    .is_some_and(|record| v8::Local::new(scope, &record.value).strict_equals(value))
            })
        })
    });
    if let Some(index) = existing {
        if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
            state.labels[index].label = label;
        }
        return;
    }
    let saved = v8::Global::new(scope, value);
    let Some(state) = scope.get_slot_mut::<NativeTraceState>() else {
        return;
    };
    let index = state.labels.len();
    state.labels.push(NativeValueLabel {
        value: saved,
        label,
    });
    state.labels_by_hash.entry(hash).or_default().push(index);
}

pub(crate) fn label_native_value_once(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    label: &str,
) {
    if native_label_for_value(scope, value).is_none() {
        label_native_value(scope, value, label);
    }
}

pub(crate) fn native_label_for_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<String> {
    let hash = value_identity_hash(value)?;
    let state = scope.get_slot::<NativeTraceState>()?;
    state
        .labels_by_hash
        .get(&hash)?
        .iter()
        .filter_map(|index| state.labels.get(*index))
        .find(|record| v8::Local::new(scope, &record.value).strict_equals(value))
        .map(|record| record.label.clone())
}

unsafe extern "C" fn native_callback_trampoline(info: *const v8::FunctionCallbackInfo) {
    // SAFETY: V8 invokes this callback with a live FunctionCallbackInfo.
    let info = unsafe { &*info };
    let parts = info.get_parts();
    let Ok(external) = v8::Local::<v8::External>::try_from(parts.data) else {
        return;
    };
    let callback = external.value().cast::<NativeCallbackRecord>();
    if callback.is_null() {
        return;
    }
    // SAFETY: the External points at a boxed record owned by the isolate slot.
    let callback = unsafe { &*callback };
    let _current_callback =
        CurrentNativeCallbackGuard::enter(callback, parts.length, info.is_construct_call());
    crate::webidl::clear_pending_conversion_type_error();
    callback.pending_exception.borrow_mut().take();
    if !callback.active.load(Ordering::Relaxed) {
        invoke_original_with_try_catch(info, callback.original);
        throw_pending_conversion_type_error(info);
        register_constructed_platform_object(info, callback);
        return;
    }

    let started = {
        let storage = std::pin::pin!(unsafe { v8::CallbackScope::new(info) });
        let mut scope = storage.init();
        native_callback_begin(&mut scope, info, callback)
    };

    // The original receives the untouched callback info. The surrounding V8
    // TryCatch preserves exceptions raised during Web IDL ToPrimitive and
    // ToString conversions even if the Rust callback continues after a V8
    // conversion method returns None.
    invoke_original_with_try_catch(info, callback.original);
    throw_pending_conversion_type_error(info);
    register_constructed_platform_object(info, callback);

    if let Some((sequence, result_label, kind, summary_kind, opaque_html_all)) = started {
        let storage = std::pin::pin!(unsafe { v8::CallbackScope::new(info) });
        let mut scope = storage.init();
        let exception = callback.pending_exception.borrow_mut().take();
        native_callback_finish(
            &mut scope,
            info,
            sequence,
            &result_label,
            kind,
            summary_kind,
            opaque_html_all,
            exception,
        );
    }
}

fn invoke_original_with_try_catch(info: &v8::FunctionCallbackInfo, original: v8::FunctionCallback) {
    let storage = std::pin::pin!(unsafe { v8::CallbackScope::new(info) });
    let mut callback_scope = storage.init();
    v8::tc_scope!(let try_catch, &mut callback_scope);
    // SAFETY: the original callback uses the same V8 ABI and untouched
    // FunctionCallbackInfo supplied by V8.
    unsafe { original(info) };
    if try_catch.has_caught() {
        let primitive_conversion_failed = try_catch
            .exception()
            .and_then(|exception| exception.to_string(try_catch))
            .map(|text| text.to_rust_string_lossy(try_catch))
            .is_some_and(|text| text == "TypeError: Cannot convert object to primitive value");
        if primitive_conversion_failed && let Some(constructor_name) = current_constructor_name() {
            try_catch.reset();
            let message = format!(
                "Failed to construct '{constructor_name}': Cannot convert object to primitive value"
            );
            if let Some(message) = v8::String::new(try_catch, &message) {
                try_catch.throw_exception(v8::Exception::type_error(try_catch, message));
                let _ = try_catch.rethrow();
            }
        } else {
            let _ = try_catch.rethrow();
        }
    }
}

fn throw_pending_conversion_type_error(info: &v8::FunctionCallbackInfo) {
    let Some(message) = crate::webidl::take_pending_conversion_type_error() else {
        return;
    };
    let storage = std::pin::pin!(unsafe { v8::CallbackScope::new(info) });
    let scope = storage.init();
    if let Some(message) = v8::String::new(&scope, &message) {
        scope.throw_exception(v8::Exception::type_error(&scope, message));
    }
}

fn register_constructed_platform_object(
    info: &v8::FunctionCallbackInfo,
    callback: &NativeCallbackRecord,
) {
    if callback.kind != NativeCallbackKind::Constructor {
        return;
    }
    let storage = std::pin::pin!(unsafe { v8::CallbackScope::new(info) });
    let mut scope = storage.init();
    let arguments = v8::FunctionCallbackArguments::from_function_callback_info(info);
    if arguments.is_construct_call() {
        crate::web::structured_clone::register_constructed_platform_object(
            &mut scope,
            arguments.this(),
        );
    }
    let value = v8::ReturnValue::from_function_callback_info(info).get(&mut scope);
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        crate::web::structured_clone::register_constructed_platform_object(&mut scope, object);
    }
}

fn native_callback_begin(
    scope: &mut v8::PinScope<'_, '_>,
    info: &v8::FunctionCallbackInfo,
    callback: &NativeCallbackRecord,
) -> Option<(
    Option<u64>,
    String,
    NativeCallbackKind,
    NativeSummaryKind,
    bool,
)> {
    let should_record = scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.recording && state.interceptor_depth == 0);
    if !should_record {
        return None;
    }
    if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
        state.interceptor_depth += 1;
    }

    let arguments = v8::FunctionCallbackArguments::from_function_callback_info(info);
    let receiver_value: v8::Local<v8::Value> = arguments.this().into();
    let receiver = native_label_for_value(scope, receiver_value)
        .unwrap_or_else(|| summarize(scope, receiver_value));
    let fixed_label = callback.label.borrow().clone();
    let api = match callback.kind {
        NativeCallbackKind::Getter | NativeCallbackKind::Setter | NativeCallbackKind::Function
            if native_label_for_value(scope, receiver_value).is_some() =>
        {
            format!("{receiver}.{}", callback.member)
        }
        NativeCallbackKind::Getter | NativeCallbackKind::Setter | NativeCallbackKind::Function => {
            fixed_label
        }
        NativeCallbackKind::Constructor => {
            let realm = scope.get_current_context().global(scope);
            match native_label_for_value(scope, realm.into()) {
                Some(realm) if realm != "window" => format!("{realm}.{}", callback.member),
                _ => fixed_label,
            }
        }
    };
    if scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.excludes_api(&api))
    {
        return Some((
            None,
            String::new(),
            callback.kind,
            callback.summary_kind,
            false,
        ));
    }
    let operation = match callback.kind {
        NativeCallbackKind::Getter => "get",
        NativeCallbackKind::Setter => "set",
        NativeCallbackKind::Function => "call",
        NativeCallbackKind::Constructor if info.is_construct_call() => "construct",
        NativeCallbackKind::Constructor => "call",
    };
    let mut argument_text = Vec::with_capacity(arguments.length().max(0) as usize);
    for index in 0..arguments.length() {
        let value = arguments.get(index);
        argument_text.push(match callback.summary_kind {
            NativeSummaryKind::Standard => summarize(scope, value),
            NativeSummaryKind::Json => summarize_json(scope, value),
        });
    }
    let result_label = match callback.kind {
        NativeCallbackKind::Getter => api.clone(),
        NativeCallbackKind::Constructor => format!("new {api}()"),
        NativeCallbackKind::Function => format!("{api}()"),
        NativeCallbackKind::Setter => String::new(),
    };
    let sequence = push_entry(
        scope,
        operation,
        api,
        receiver,
        argument_text.join(","),
        "pending".to_owned(),
    );
    let opaque_html_all = callback.kind == NativeCallbackKind::Getter
        && callback.label.borrow().as_str() == "Document.prototype.get all";
    Some((
        sequence,
        result_label,
        callback.kind,
        callback.summary_kind,
        opaque_html_all,
    ))
}

fn native_callback_finish(
    scope: &mut v8::PinScope<'_, '_>,
    info: &v8::FunctionCallbackInfo,
    sequence: Option<u64>,
    result_label: &str,
    kind: NativeCallbackKind,
    summary_kind: NativeSummaryKind,
    opaque_html_all: bool,
    exception: Option<String>,
) {
    let Some(sequence) = sequence else {
        if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
            state.interceptor_depth = state.interceptor_depth.saturating_sub(1);
        }
        return;
    };
    let value = v8::ReturnValue::from_function_callback_info(info).get(scope);
    // document.all is V8's deliberately undetectable legacy exotic object.
    // It is simultaneously object-backed and `typeof`-undefined. Constructor
    // introspection on that representation reaches an empty V8 optional in
    // no-exceptions builds, so trace records the browser-visible brand without
    // probing the value.
    let summary = if let Some(exception) = exception {
        format!("threw {exception}")
    } else if opaque_html_all {
        "[object HTMLAllCollection]".to_owned()
    } else {
        match summary_kind {
            NativeSummaryKind::Standard => summarize(scope, value),
            NativeSummaryKind::Json => summarize_json(scope, value),
        }
    };
    if !result_label.is_empty() && value.is_object() {
        let existing = native_label_for_value(scope, value);
        let realm_object = existing.as_deref().is_some_and(|label| {
            label == "window"
                || label.starts_with("iframe[")
                || label.starts_with("worker[")
                || label.starts_with("sharedWorker[")
                || label.starts_with("serviceWorker[")
                || label.starts_with("audioWorklet[")
                || label.starts_with("paintWorklet[")
        });
        let would_extend_existing_path = existing.as_deref().is_some_and(|existing| {
            result_label.len() > existing.len() && result_label.starts_with(existing)
        });
        let replace = !would_extend_existing_path
            && (kind == NativeCallbackKind::Function
                || (kind == NativeCallbackKind::Getter && !realm_object)
                || existing
                    .as_deref()
                    .is_some_and(|label| label.starts_with("new ")));
        if replace {
            label_native_value(scope, value, result_label);
        } else {
            label_native_value_once(scope, value, result_label);
        }
    }
    if let Some(state) = scope.get_slot_mut::<NativeTraceState>() {
        state.interceptor_depth = state.interceptor_depth.saturating_sub(1);
        let result = state.intern_trace_string(summary);
        if let Some(entry) = state
            .entries
            .iter_mut()
            .rev()
            .find(|entry| entry.sequence == sequence)
        {
            entry.result = result;
        }
    }
}

fn push_entry(
    scope: &mut v8::PinScope<'_, '_>,
    operation: &str,
    api: String,
    receiver: String,
    arguments: String,
    result: String,
) -> Option<u64> {
    let state = scope.get_slot_mut::<NativeTraceState>()?;
    if state.excludes_api(&api) {
        return None;
    }
    state.sequence += 1;
    let sequence = state.sequence;
    let operation = state.intern_trace_string(operation.to_owned());
    let api = state.intern_trace_string(api);
    let receiver = state.intern_trace_string(receiver);
    let arguments = state.intern_trace_string(arguments);
    let result = state.intern_trace_string(result);
    state.entries.push(CompactTraceEntry {
        sequence,
        api,
        receiver,
        arguments,
        result,
        operation,
    });
    Some(sequence)
}

fn record_entry(
    scope: &mut v8::PinScope<'_, '_>,
    operation: &str,
    api: String,
    receiver: String,
    arguments: String,
    result: String,
) {
    let should_record = scope
        .get_slot::<NativeTraceState>()
        .is_some_and(|state| state.recording && state.interceptor_depth == 0);
    if !should_record {
        return;
    }
    let _ = push_entry(scope, operation, api, receiver, arguments, result);
}

pub(crate) fn unwrap_traced_value<'s>(
    _scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> v8::Local<'s, v8::Value> {
    value
}

pub(crate) fn visible_callback_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    label: &str,
) -> v8::Local<'s, v8::Value> {
    if is_enabled(scope) {
        label_native_value_once(scope, value, label);
    }
    value
}

pub(crate) fn reserve_prototype_property_label_from_global(
    scope: &mut v8::PinScope<'_, '_>,
    interface: &str,
    property: &str,
    label: &str,
) -> Result<(), String> {
    if !is_enabled(scope) {
        return Ok(());
    }
    let global = scope.get_current_context().global(scope);
    let interface_key = crate::webidl::string(scope, interface)?;
    let constructor = global
        .get(scope, interface_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| format!("cannot obtain trace interface {interface}"))?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    let prototype = constructor
        .get(scope, prototype_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| format!("cannot obtain {interface}.prototype"))?;
    let property_key = crate::webidl::string(scope, property)?;
    let descriptor = prototype
        .get_own_property_descriptor(scope, property_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| format!("{interface}.prototype.{property} is unavailable"))?;
    let value_key = crate::webidl::string(scope, "value")?;
    let value = descriptor
        .get(scope, value_key.into())
        .ok_or_else(|| format!("{interface}.prototype.{property} has no value"))?;
    label_native_value_once(scope, value, label);
    if let Ok(function) = v8::Local::<v8::Function>::try_from(value) {
        relabel_native_function(scope, function, label);
    }
    Ok(())
}

fn value_identity_hash(value: v8::Local<'_, v8::Value>) -> Option<i32> {
    // A revoked JavaScript Proxy has no usable target/handler pair. Asking V8
    // for object identity metadata on that value reaches an empty internal
    // optional in no-exceptions builds. Trace must treat user-created proxies
    // as opaque values and must never inspect or trigger their traps.
    if value.is_proxy() {
        return None;
    }
    v8::Local::<v8::Object>::try_from(value)
        .ok()
        .map(|object| object.get_identity_hash().get())
}

fn property_name(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> String {
    if value.is_symbol() {
        let symbol = v8::Local::<v8::Symbol>::try_from(value).ok();
        let description = symbol
            .map(|symbol| symbol.description(scope))
            .and_then(|description| description.to_string(scope))
            .map(|description| description.to_rust_string_lossy(scope));
        return description
            .map(|description| bounded_text(&description, MAX_TRACE_PROPERTY_CHARACTERS))
            .map(|description| format!("@@{description}"))
            .unwrap_or_else(|| "@@Symbol".to_owned());
    }
    bounded_text(
        &crate::webidl::value_to_string(scope, value),
        MAX_TRACE_PROPERTY_CHARACTERS,
    )
}

fn summarize<'s>(scope: &v8::PinScope<'s, '_>, value: v8::Local<'s, v8::Value>) -> String {
    summarize_inner(scope, value, 0, &mut Vec::new())
}

fn summarize_json<'s>(scope: &v8::PinScope<'s, '_>, value: v8::Local<'s, v8::Value>) -> String {
    if value.is_string() {
        let text = crate::webidl::value_to_string(scope, value);
        return format!(
            "\"{}\"",
            escape_with_limit(&text, MAX_TRACE_ARRAY_CHARACTERS.saturating_sub(2))
        );
    }
    summarize_json_inner(scope, value, 0, &mut Vec::new())
}

fn summarize_json_inner<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    depth: usize,
    ancestors: &mut Vec<v8::Local<'s, v8::Object>>,
) -> String {
    if value.is_string() {
        let text = crate::webidl::value_to_string(scope, value);
        return format!(
            "\"{}\"",
            escape_with_limit(&text, MAX_TRACE_ARRAY_CHARACTERS.saturating_sub(2))
        );
    }
    if value.is_proxy() {
        return "[object Proxy]".to_owned();
    }
    if let Ok(array) = v8::Local::<v8::Array>::try_from(value) {
        return summarize_json_array(scope, array, depth, ancestors);
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return summarize_inner(scope, value, depth, ancestors);
    };
    let constructor = object.get_constructor_name().to_rust_string_lossy(scope);
    if constructor != "Object" || native_label_for_value(scope, value).is_some() {
        return summarize_inner(scope, value, depth, ancestors);
    }
    summarize_json_object(scope, object, depth, ancestors)
}

fn summarize_json_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    array: v8::Local<'s, v8::Array>,
    depth: usize,
    ancestors: &mut Vec<v8::Local<'s, v8::Object>>,
) -> String {
    if depth >= MAX_TRACE_ARRAY_DEPTH {
        return "[Array]".to_owned();
    }
    let object = v8::Local::<v8::Object>::from(array);
    if ancestors
        .iter()
        .any(|ancestor| ancestor.strict_equals(object.into()))
    {
        return "[Circular]".to_owned();
    }
    ancestors.push(object);
    let length = array.length() as usize;
    let retained = length.min(MAX_TRACE_ARRAY_ITEMS);
    let mut values = Vec::with_capacity(retained + usize::from(length > retained));
    for index in 0..retained {
        values.push(
            own_data_value_by_index(scope, object, index as u32)
                .map(|value| summarize_json_inner(scope, value, depth + 1, ancestors))
                .unwrap_or_else(|| "[accessor-or-empty]".to_owned()),
        );
    }
    if length > retained {
        values.push(format!("... {} more", length - retained));
    }
    ancestors.pop();
    bounded_text(
        &format!("[{}]", values.join(",")),
        MAX_TRACE_ARRAY_CHARACTERS,
    )
}

fn summarize_json_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    depth: usize,
    ancestors: &mut Vec<v8::Local<'s, v8::Object>>,
) -> String {
    if depth >= MAX_TRACE_ARRAY_DEPTH {
        return "{Object}".to_owned();
    }
    if ancestors
        .iter()
        .any(|ancestor| ancestor.strict_equals(object.into()))
    {
        return "[Circular]".to_owned();
    }
    ancestors.push(object);
    let arguments = v8::GetPropertyNamesArgs {
        mode: v8::KeyCollectionMode::OwnOnly,
        property_filter: v8::PropertyFilter::ONLY_ENUMERABLE | v8::PropertyFilter::SKIP_SYMBOLS,
        index_filter: v8::IndexFilter::IncludeIndices,
        key_conversion: v8::KeyConversionMode::ConvertToString,
    };
    let Some(keys) = object.get_own_property_names(scope, arguments) else {
        ancestors.pop();
        return "{Object}".to_owned();
    };
    let actual = keys.length() as usize;
    let retained = actual.min(MAX_TRACE_OBJECT_PROPERTIES);
    let mut properties = Vec::with_capacity(retained + usize::from(actual > retained));
    for index in 0..retained {
        let Some(key_value) = keys.get_index(scope, index as u32) else {
            continue;
        };
        let Some(key) = key_value.to_string(scope) else {
            continue;
        };
        let name = key.to_rust_string_lossy(scope);
        let value = own_data_value_by_name(scope, object, key.into())
            .map(|value| summarize_json_inner(scope, value, depth + 1, ancestors))
            .unwrap_or_else(|| "[accessor]".to_owned());
        properties.push(format!(
            "\"{}\":{}",
            escape_with_limit(&name, MAX_TRACE_PROPERTY_CHARACTERS),
            value
        ));
    }
    if actual > retained {
        properties.push(format!("... {} more", actual - retained));
    }
    ancestors.pop();
    bounded_text(
        &format!("{{{}}}", properties.join(",")),
        MAX_TRACE_ARRAY_CHARACTERS,
    )
}

fn own_data_value_by_index<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    index: u32,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, &index.to_string())?;
    own_data_value_by_name(scope, object, key.into())
}

fn own_data_value_by_name<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    key: v8::Local<'s, v8::Name>,
) -> Option<v8::Local<'s, v8::Value>> {
    let descriptor = object.get_own_property_descriptor(scope, key)?;
    let descriptor = v8::Local::<v8::Object>::try_from(descriptor).ok()?;
    let value_key = v8::String::new(scope, "value")?;
    (descriptor.has_own_property(scope, value_key.into()) == Some(true))
        .then(|| descriptor.get(scope, value_key.into()))
        .flatten()
}

fn summarize_inner<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    depth: usize,
    ancestors: &mut Vec<v8::Local<'s, v8::Object>>,
) -> String {
    if value.is_undefined() {
        return "undefined".to_owned();
    }
    if value.is_null() {
        return "null".to_owned();
    }
    if value.is_boolean() {
        return value.boolean_value(scope).to_string();
    }
    if value.is_number() || value.is_big_int() {
        return crate::webidl::value_to_string(scope, value);
    }
    if value.is_string() {
        let text = crate::webidl::value_to_string(scope, value);
        return format!("\"{}\"", escape_and_limit(&text));
    }
    if value.is_symbol() {
        return property_name(scope, value);
    }
    if value.is_proxy() {
        return "[object Proxy]".to_owned();
    }
    if let Ok(array) = v8::Local::<v8::Array>::try_from(value) {
        return summarize_array(scope, array, depth, ancestors);
    }
    if let Some(label) = native_label_for_value(scope, value) {
        if value.is_function() {
            return format!("[function {label}]");
        }
        return format!("[object {label}]");
    }
    if value.is_function() {
        let name = v8::Local::<v8::Function>::try_from(value)
            .ok()
            .map(|function| function.get_name(scope).to_rust_string_lossy(scope))
            .unwrap_or_default();
        return format!(
            "[function {}]",
            bounded_text(&name, MAX_TRACE_PROPERTY_CHARACTERS)
        );
    }
    // V8 obtains this from the object's internal map; unlike reading the
    // JavaScript-visible `constructor` property it does not run accessors or
    // user Proxy traps.
    let constructor = v8::Local::<v8::Object>::try_from(value)
        .ok()
        .map(|object| object.get_constructor_name().to_rust_string_lossy(scope))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Object".to_owned());
    format!(
        "[object {}]",
        bounded_text(&constructor, MAX_TRACE_PROPERTY_CHARACTERS)
    )
}

fn summarize_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    array: v8::Local<'s, v8::Array>,
    depth: usize,
    ancestors: &mut Vec<v8::Local<'s, v8::Object>>,
) -> String {
    if depth >= MAX_TRACE_ARRAY_DEPTH {
        return "[Array]".to_owned();
    }
    let object = v8::Local::<v8::Object>::from(array);
    if ancestors
        .iter()
        .any(|ancestor| ancestor.strict_equals(object.into()))
    {
        return "[Circular]".to_owned();
    }
    ancestors.push(object);
    let length = array.length() as usize;
    let retained = length.min(MAX_TRACE_ARRAY_ITEMS);
    let mut values = Vec::with_capacity(retained + usize::from(length > retained));
    for index in 0..retained {
        let Some(key) = v8::String::new(scope, &index.to_string()) else {
            values.push("<empty>".to_owned());
            continue;
        };
        let Some(descriptor) = object.get_own_property_descriptor(scope, key.into()) else {
            values.push("<empty>".to_owned());
            continue;
        };
        let Ok(descriptor) = v8::Local::<v8::Object>::try_from(descriptor) else {
            values.push("<empty>".to_owned());
            continue;
        };
        let Some(value_key) = v8::String::new(scope, "value") else {
            values.push("[accessor]".to_owned());
            continue;
        };
        if descriptor.has_own_property(scope, value_key.into()) != Some(true) {
            values.push("[accessor]".to_owned());
            continue;
        }
        let Some(value) = descriptor.get(scope, value_key.into()) else {
            values.push("undefined".to_owned());
            continue;
        };
        values.push(summarize_inner(scope, value, depth + 1, ancestors));
    }
    if length > retained {
        values.push(format!("... {} more", length - retained));
    }
    ancestors.pop();
    bounded_text(
        &format!("[{}]", values.join(",")),
        MAX_TRACE_ARRAY_CHARACTERS,
    )
}

fn bounded_text(value: &str, maximum_characters: usize) -> String {
    if value.chars().count() <= maximum_characters {
        return value.to_owned();
    }
    let mut output = value.chars().take(maximum_characters).collect::<String>();
    output.push('…');
    output
}

fn escape_and_limit(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars().take(96) {
        match character {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            other => output.push(other),
        }
    }
    if value.chars().count() > 96 {
        output.push('…');
    }
    output
}

fn escape_with_limit(value: &str, maximum_characters: usize) -> String {
    let mut output = String::new();
    let mut output_characters = 0usize;
    let mut truncated = false;
    for character in value.chars() {
        let escaped = match character {
            '\n' => "\\n".to_owned(),
            '\r' => "\\r".to_owned(),
            '\t' => "\\t".to_owned(),
            '\\' => "\\\\".to_owned(),
            '"' => "\\\"".to_owned(),
            other => other.to_string(),
        };
        let escaped_characters = escaped.chars().count();
        if output_characters.saturating_add(escaped_characters) > maximum_characters {
            truncated = true;
            break;
        }
        output.push_str(&escaped);
        output_characters += escaped_characters;
    }
    if truncated && output_characters < maximum_characters {
        output.push('…');
    }
    output
}
