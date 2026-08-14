use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceScriptTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ScriptTimingRecord>,
}

#[derive(Clone)]
struct ScriptTimingRecord {
    invoker_type: String,
    invoker: String,
    window_attribution: String,
    execution_start: f64,
    forced_style_and_layout_duration: f64,
    pause_duration: f64,
    window: Option<v8::Global<v8::Value>>,
    source_url: String,
    source_function_name: String,
    source_char_position: i32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceScriptTimingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceScriptTiming", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceScriptTimingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceScriptTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "invokerType", get_invoker_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "invoker", get_invoker)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "windowAttribution",
        get_window_attribution,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "executionStart",
        get_execution_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "forcedStyleAndLayoutDuration",
        get_forced_style_and_layout_duration,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pauseDuration", get_pause_duration)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "window", get_window)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sourceURL", get_source_url)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "sourceFunctionName",
        get_source_function_name,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "sourceCharPosition",
        get_source_char_position,
    )?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceScriptTimingStore>()
        .ok_or_else(|| "PerformanceScriptTiming state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'PerformanceScriptTiming': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    start_time: f64,
    duration: f64,
    source_url: String,
    source_function_name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceScriptTiming".to_owned());
    }
    super::performance_entry::attach(
        scope,
        timing,
        name,
        "script".to_owned(),
        start_time,
        duration,
    );
    scope
        .get_slot_mut::<PerformanceScriptTimingStore>()
        .ok_or_else(|| "PerformanceScriptTiming state was not prepared".to_owned())?
        .records
        .insert(
            timing.get_identity_hash().get(),
            ScriptTimingRecord {
                invoker_type: "classic-script".to_owned(),
                invoker: String::new(),
                window_attribution: "self".to_owned(),
                execution_start: start_time,
                forced_style_and_layout_duration: 0.0,
                pause_duration: 0.0,
                window: None,
                source_url,
                source_function_name,
                source_char_position: 0,
            },
        );
    Ok(timing)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ScriptTimingRecord> {
    scope
        .get_slot::<PerformanceScriptTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ScriptTimingRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ScriptTimingRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_invoker_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.invoker_type);
}
fn get_invoker(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.invoker);
}
fn get_window_attribution(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.window_attribution);
}
fn get_execution_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.execution_start);
}
fn get_forced_style_and_layout_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.forced_style_and_layout_duration);
}
fn get_pause_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.pause_duration);
}
fn get_source_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.source_url);
}
fn get_source_function_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.source_function_name);
}

fn get_source_char_position(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.source_char_position).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(window) = record.window {
        result.set(v8::Local::new(scope, &window));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define_value(scope, object, name, value.into());
    }
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    define_value(scope, object, name, v8::Number::new(scope, value).into());
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(base) = super::performance_entry::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = super::performance_entry::to_object(scope, &base);
    define_string(scope, output, "invokerType", &record.invoker_type);
    define_string(scope, output, "invoker", &record.invoker);
    define_string(
        scope,
        output,
        "windowAttribution",
        &record.window_attribution,
    );
    define_number(scope, output, "executionStart", record.execution_start);
    define_number(
        scope,
        output,
        "forcedStyleAndLayoutDuration",
        record.forced_style_and_layout_duration,
    );
    define_number(scope, output, "pauseDuration", record.pause_duration);
    if let Some(window) = record.window {
        define_value(scope, output, "window", v8::Local::new(scope, &window));
    } else {
        define_value(scope, output, "window", v8::null(scope).into());
    }
    define_string(scope, output, "sourceURL", &record.source_url);
    define_string(
        scope,
        output,
        "sourceFunctionName",
        &record.source_function_name,
    );
    define_value(
        scope,
        output,
        "sourceCharPosition",
        v8::Integer::new(scope, record.source_char_position).into(),
    );
    result.set(output.into());
}
