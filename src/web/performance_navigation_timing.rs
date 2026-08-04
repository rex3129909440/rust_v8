use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceNavigationTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationTimingRecord>,
}

#[derive(Clone)]
struct NavigationTimingRecord {
    unload_event_start: f64,
    unload_event_end: f64,
    dom_interactive: f64,
    dom_content_loaded_event_start: f64,
    dom_content_loaded_event_end: f64,
    dom_complete: f64,
    load_event_start: f64,
    load_event_end: f64,
    navigation_type: String,
    redirect_count: i32,
    critical_ch_restart: f64,
    activation_start: f64,
    confidence: v8::Global<v8::Object>,
    not_restored_reasons: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceNavigationTimingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceNavigationTiming", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceNavigationTimingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceNavigationTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "unloadEventStart",
        get_unload_event_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "unloadEventEnd",
        get_unload_event_end,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "domInteractive",
        get_dom_interactive,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "domContentLoadedEventStart",
        get_dom_content_loaded_event_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "domContentLoadedEventEnd",
        get_dom_content_loaded_event_end,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "domComplete", get_dom_complete)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "loadEventStart",
        get_load_event_start,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "loadEventEnd", get_load_event_end)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "redirectCount", get_redirect_count)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "criticalCHRestart",
        get_critical_ch_restart,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "activationStart",
        get_activation_start,
    )?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "confidence", get_confidence)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "notRestoredReasons",
        get_not_restored_reasons,
    )?;
    let parent = super::performance_resource_timing::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceNavigationTimingStore>()
        .ok_or_else(|| "PerformanceNavigationTiming state was not prepared".to_owned())?
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
        "Failed to construct 'PerformanceNavigationTiming': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    duration: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceNavigationTiming".to_owned());
    }
    super::performance_resource_timing::attach_default(
        scope,
        timing,
        name.clone(),
        "navigation".to_owned(),
        0.0,
        duration,
    );
    super::performance_entry::attach(scope, timing, name, "navigation".to_owned(), 0.0, duration);
    let confidence = v8::Object::new(scope);
    define_number(scope, confidence, "randomizedTriggerRate", 0.0);
    define_string(scope, confidence, "value", "high");
    let record = NavigationTimingRecord {
        unload_event_start: 0.0,
        unload_event_end: 0.0,
        dom_interactive: duration,
        dom_content_loaded_event_start: duration,
        dom_content_loaded_event_end: duration,
        dom_complete: duration,
        load_event_start: duration,
        load_event_end: duration,
        navigation_type: "navigate".to_owned(),
        redirect_count: 0,
        critical_ch_restart: 0.0,
        activation_start: 0.0,
        confidence: v8::Global::new(scope, confidence),
        not_restored_reasons: None,
    };
    scope
        .get_slot_mut::<PerformanceNavigationTimingStore>()
        .ok_or_else(|| "PerformanceNavigationTiming state was not prepared".to_owned())?
        .records
        .insert(timing.get_identity_hash().get(), record);
    Ok(timing)
}

pub(crate) fn create_for_navigation<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    duration: f64,
    response_status: u16,
    body_size: usize,
    content_type: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let timing = create(scope, name, duration)?;
    super::performance_resource_timing::configure_response(
        scope,
        timing,
        0.0,
        duration,
        response_status,
        body_size,
        content_type,
        String::new(),
    );
    Ok(timing)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationTimingRecord> {
    scope
        .get_slot::<PerformanceNavigationTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigationTimingRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_unload_event_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.unload_event_start);
}
fn get_unload_event_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.unload_event_end);
}
fn get_dom_interactive(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.dom_interactive);
}
fn get_dom_content_loaded_event_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.dom_content_loaded_event_start);
}
fn get_dom_content_loaded_event_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.dom_content_loaded_event_end);
}
fn get_dom_complete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.dom_complete);
}
fn get_load_event_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.load_event_start);
}
fn get_load_event_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.load_event_end);
}
fn get_critical_ch_restart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.critical_ch_restart);
}
fn get_activation_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.activation_start);
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.navigation_type) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_redirect_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.redirect_count).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_confidence(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.confidence).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_not_restored_reasons(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.not_restored_reasons {
        result.set(v8::Local::new(scope, &value).into());
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
    let Some(output) = super::performance_resource_timing::to_object(scope, arguments.this())
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    define_number(scope, output, "unloadEventStart", record.unload_event_start);
    define_number(scope, output, "unloadEventEnd", record.unload_event_end);
    define_number(scope, output, "domInteractive", record.dom_interactive);
    define_number(
        scope,
        output,
        "domContentLoadedEventStart",
        record.dom_content_loaded_event_start,
    );
    define_number(
        scope,
        output,
        "domContentLoadedEventEnd",
        record.dom_content_loaded_event_end,
    );
    define_number(scope, output, "domComplete", record.dom_complete);
    define_number(scope, output, "loadEventStart", record.load_event_start);
    define_number(scope, output, "loadEventEnd", record.load_event_end);
    define_string(scope, output, "type", &record.navigation_type);
    define_value(
        scope,
        output,
        "redirectCount",
        v8::Integer::new(scope, record.redirect_count).into(),
    );
    define_number(scope, output, "activationStart", record.activation_start);
    define_number(
        scope,
        output,
        "criticalCHRestart",
        record.critical_ch_restart,
    );
    if let Some(value) = record.not_restored_reasons {
        define_value(
            scope,
            output,
            "notRestoredReasons",
            v8::Local::new(scope, &value).into(),
        );
    } else {
        define_value(scope, output, "notRestoredReasons", v8::null(scope).into());
    }
    define_value(
        scope,
        output,
        "confidence",
        v8::Local::new(scope, &record.confidence).into(),
    );
    result.set(output.into());
}
