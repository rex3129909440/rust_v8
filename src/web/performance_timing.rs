use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PerformanceTimingRecord>,
}

#[derive(Clone, Default)]
struct PerformanceTimingRecord {
    navigation_start: f64,
    unload_event_start: f64,
    unload_event_end: f64,
    redirect_start: f64,
    redirect_end: f64,
    fetch_start: f64,
    domain_lookup_start: f64,
    domain_lookup_end: f64,
    connect_start: f64,
    connect_end: f64,
    secure_connection_start: f64,
    request_start: f64,
    response_start: f64,
    response_end: f64,
    dom_loading: f64,
    dom_interactive: f64,
    dom_content_loaded_event_start: f64,
    dom_content_loaded_event_end: f64,
    dom_complete: f64,
    load_event_start: f64,
    load_event_end: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceTimingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceTiming", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceTimingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "navigationStart",
        get_navigation_start,
    )?;
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
    crate::webidl::define_readonly_accessor(scope, prototype, "redirectStart", get_redirect_start)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "redirectEnd", get_redirect_end)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "fetchStart", get_fetch_start)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "domainLookupStart",
        get_domain_lookup_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "domainLookupEnd",
        get_domain_lookup_end,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "connectStart", get_connect_start)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "connectEnd", get_connect_end)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "secureConnectionStart",
        get_secure_connection_start,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "requestStart", get_request_start)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "responseStart", get_response_start)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "responseEnd", get_response_end)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "domLoading", get_dom_loading)?;
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
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceTimingStore>()
        .ok_or_else(|| "PerformanceTiming state was not prepared".to_owned())?
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
        "Failed to construct 'PerformanceTiming': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    navigation_start: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceTiming".to_owned());
    }
    scope
        .get_slot_mut::<PerformanceTimingStore>()
        .ok_or_else(|| "PerformanceTiming state was not prepared".to_owned())?
        .records
        .insert(
            timing.get_identity_hash().get(),
            PerformanceTimingRecord {
                navigation_start,
                ..PerformanceTimingRecord::default()
            },
        );
    Ok(timing)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PerformanceTimingRecord> {
    scope
        .get_slot::<PerformanceTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PerformanceTimingRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_navigation_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.navigation_start);
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
fn get_redirect_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.redirect_start);
}
fn get_redirect_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.redirect_end);
}
fn get_fetch_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.fetch_start);
}
fn get_domain_lookup_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.domain_lookup_start);
}
fn get_domain_lookup_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.domain_lookup_end);
}
fn get_connect_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.connect_start);
}
fn get_connect_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.connect_end);
}
fn get_secure_connection_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.secure_connection_start);
}
fn get_request_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.request_start);
}
fn get_response_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.response_start);
}
fn get_response_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.response_end);
}
fn get_dom_loading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.dom_loading);
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

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
    }
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
    let output = v8::Object::new(scope);
    define_number(scope, output, "connectStart", record.connect_start);
    define_number(
        scope,
        output,
        "secureConnectionStart",
        record.secure_connection_start,
    );
    define_number(scope, output, "unloadEventEnd", record.unload_event_end);
    define_number(
        scope,
        output,
        "domainLookupStart",
        record.domain_lookup_start,
    );
    define_number(scope, output, "domainLookupEnd", record.domain_lookup_end);
    define_number(scope, output, "responseStart", record.response_start);
    define_number(scope, output, "connectEnd", record.connect_end);
    define_number(scope, output, "responseEnd", record.response_end);
    define_number(scope, output, "requestStart", record.request_start);
    define_number(scope, output, "domLoading", record.dom_loading);
    define_number(scope, output, "redirectStart", record.redirect_start);
    define_number(scope, output, "loadEventEnd", record.load_event_end);
    define_number(scope, output, "domComplete", record.dom_complete);
    define_number(scope, output, "navigationStart", record.navigation_start);
    define_number(scope, output, "loadEventStart", record.load_event_start);
    define_number(
        scope,
        output,
        "domContentLoadedEventEnd",
        record.dom_content_loaded_event_end,
    );
    define_number(scope, output, "unloadEventStart", record.unload_event_start);
    define_number(scope, output, "redirectEnd", record.redirect_end);
    define_number(scope, output, "domInteractive", record.dom_interactive);
    define_number(scope, output, "fetchStart", record.fetch_start);
    define_number(
        scope,
        output,
        "domContentLoadedEventStart",
        record.dom_content_loaded_event_start,
    );
    result.set(output.into());
}
