use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceResourceTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ResourceTimingRecord>,
}

#[derive(Clone, Default)]
pub(crate) struct ResourceTimingRecord {
    initiator_type: String,
    next_hop_protocol: String,
    delivery_type: String,
    worker_start: f64,
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
    transfer_size: f64,
    encoded_body_size: f64,
    decoded_body_size: f64,
    server_timing: Vec<v8::Global<v8::Object>>,
    response_status: i32,
    final_response_headers_start: f64,
    first_interim_response_start: f64,
    worker_router_evaluation_start: f64,
    worker_cache_lookup_start: f64,
    worker_matched_source_type: String,
    worker_final_source_type: String,
    render_blocking_status: String,
    content_type: String,
    content_encoding: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceResourceTimingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceResourceTiming", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<PerformanceResourceTimingStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceResourceTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "initiatorType", get_initiator_type)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "nextHopProtocol",
        get_next_hop_protocol,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "deliveryType", get_delivery_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "workerStart", get_worker_start)?;
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
    crate::webidl::define_readonly_accessor(scope, prototype, "transferSize", get_transfer_size)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "encodedBodySize",
        get_encoded_body_size,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "decodedBodySize",
        get_decoded_body_size,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "serverTiming", get_server_timing)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "responseStatus",
        get_response_status,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "finalResponseHeadersStart",
        get_final_response_headers_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "firstInterimResponseStart",
        get_first_interim_response_start,
    )?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "workerRouterEvaluationStart",
        get_worker_router_evaluation_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "workerCacheLookupStart",
        get_worker_cache_lookup_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "workerMatchedSourceType",
        get_worker_matched_source_type,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "workerFinalSourceType",
        get_worker_final_source_type,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "renderBlockingStatus",
        get_render_blocking_status,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "contentType", get_content_type)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "contentEncoding",
        get_content_encoding,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceResourceTimingStore>()
        .ok_or_else(|| "PerformanceResourceTiming state was not prepared".to_owned())?
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
        "Failed to construct 'PerformanceResourceTiming': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    initiator_type: String,
    start_time: f64,
    duration: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceResourceTiming".to_owned());
    }
    attach_default(scope, timing, name, initiator_type, start_time, duration);
    Ok(timing)
}

pub(crate) fn create_for_resource<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    initiator_type: String,
    start_time: f64,
    duration: f64,
    response_status: u16,
    body_size: usize,
    content_type: String,
    content_encoding: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let timing = create(scope, name, initiator_type, start_time, duration)?;
    configure_response(
        scope,
        timing,
        start_time,
        duration,
        response_status,
        body_size,
        content_type,
        content_encoding,
    );
    Ok(timing)
}

pub(crate) fn configure_response(
    scope: &mut v8::PinScope<'_, '_>,
    timing: v8::Local<'_, v8::Object>,
    start_time: f64,
    duration: f64,
    response_status: u16,
    body_size: usize,
    content_type: String,
    content_encoding: String,
) {
    let response_end = start_time + duration;
    if let Some(record) = scope
        .get_slot_mut::<PerformanceResourceTimingStore>()
        .and_then(|store| store.records.get_mut(&timing.get_identity_hash().get()))
    {
        let body_size = body_size as f64;
        record.next_hop_protocol = "h2".to_owned();
        record.domain_lookup_start = start_time;
        record.domain_lookup_end = start_time;
        record.connect_start = start_time;
        record.connect_end = start_time;
        record.request_start = start_time;
        record.response_start = response_end;
        record.response_end = response_end;
        record.final_response_headers_start = response_end;
        record.transfer_size = if response_status == 0 {
            0.0
        } else {
            body_size + 300.0
        };
        record.encoded_body_size = body_size;
        record.decoded_body_size = body_size;
        record.response_status = i32::from(response_status);
        record.content_type = content_type;
        record.content_encoding = content_encoding;
    }
}

pub(crate) fn record_network_replay(
    scope: &mut v8::PinScope<'_, '_>,
    replay: &crate::NetworkReplayEntry,
    initiator_type: &str,
    start_time: f64,
) {
    let end_time = super::performance::now_for_current_realm(scope).unwrap_or(start_time);
    let content_type = header_value(&replay.headers, "content-type")
        .map(media_type_essence)
        .unwrap_or_default();
    let content_encoding = header_value(&replay.headers, "content-encoding")
        .unwrap_or_default()
        .to_owned();
    if let Ok(entry) = create_for_resource(
        scope,
        replay.url.clone(),
        initiator_type.to_owned(),
        start_time,
        (end_time - start_time).max(0.0),
        replay.status,
        replay.body.len(),
        content_type,
        content_encoding,
    ) {
        super::performance::add_entry_for_current_realm(scope, entry, "resource");
    }
}

pub(crate) fn record_evaluated_script(
    scope: &mut v8::PinScope<'_, '_>,
    source_url: &str,
    body_size: usize,
) {
    let start_time = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
    if let Ok(entry) = create_for_resource(
        scope,
        source_url.to_owned(),
        "script".to_owned(),
        start_time,
        0.0,
        200,
        body_size,
        "text/javascript".to_owned(),
        String::new(),
    ) {
        super::performance::add_entry_for_current_realm(scope, entry, "resource");
    }
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn media_type_essence(value: &str) -> String {
    value
        .split_once(';')
        .map_or(value, |(essence, _)| essence)
        .trim()
        .to_ascii_lowercase()
}

pub(crate) fn attach_default(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: String,
    initiator_type: String,
    start_time: f64,
    duration: f64,
) {
    super::performance_entry::attach(
        scope,
        object,
        name,
        "resource".to_owned(),
        start_time,
        duration,
    );
    if let Some(store) = scope.get_slot_mut::<PerformanceResourceTimingStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            ResourceTimingRecord {
                initiator_type,
                next_hop_protocol: String::new(),
                delivery_type: String::new(),
                fetch_start: start_time,
                response_end: start_time + duration,
                render_blocking_status: "non-blocking".to_owned(),
                ..ResourceTimingRecord::default()
            },
        );
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ResourceTimingRecord> {
    scope
        .get_slot::<PerformanceResourceTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn response_end(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<f64> {
    record(scope, object).map(|record| record.response_end)
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ResourceTimingRecord) -> &str,
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
    select: impl FnOnce(&ResourceTimingRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_initiator_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.initiator_type);
}
fn get_next_hop_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.next_hop_protocol);
}
fn get_delivery_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.delivery_type);
}
fn get_worker_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.worker_start);
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
fn get_transfer_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.transfer_size);
}
fn get_encoded_body_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.encoded_body_size);
}
fn get_decoded_body_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.decoded_body_size);
}
fn get_final_response_headers_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.final_response_headers_start);
}
fn get_first_interim_response_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.first_interim_response_start);
}
fn get_worker_router_evaluation_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.worker_router_evaluation_start);
}
fn get_worker_cache_lookup_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.worker_cache_lookup_start);
}
fn get_worker_matched_source_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.worker_matched_source_type);
}
fn get_worker_final_source_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.worker_final_source_type);
}
fn get_render_blocking_status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.render_blocking_status);
}
fn get_content_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.content_type);
}
fn get_content_encoding(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.content_encoding);
}

fn get_response_status(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.response_status).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_server_timing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = v8::Array::new(scope, record.server_timing.len() as i32);
    for (index, timing) in record.server_timing.iter().enumerate() {
        let _ = values.set_index(scope, index as u32, v8::Local::new(scope, timing).into());
    }
    result.set(values.into());
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

pub(crate) fn to_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = record(scope, object)?;
    let base = super::performance_entry::record(scope, object)?;
    let output = super::performance_entry::to_object(scope, &base);
    define_string(scope, output, "initiatorType", &record.initiator_type);
    define_string(scope, output, "deliveryType", &record.delivery_type);
    define_string(scope, output, "nextHopProtocol", &record.next_hop_protocol);
    define_string(
        scope,
        output,
        "renderBlockingStatus",
        &record.render_blocking_status,
    );
    define_string(scope, output, "contentType", &record.content_type);
    define_string(scope, output, "contentEncoding", &record.content_encoding);
    define_number(scope, output, "workerStart", record.worker_start);
    define_number(
        scope,
        output,
        "workerRouterEvaluationStart",
        record.worker_router_evaluation_start,
    );
    define_number(
        scope,
        output,
        "workerCacheLookupStart",
        record.worker_cache_lookup_start,
    );
    define_string(
        scope,
        output,
        "workerMatchedSourceType",
        &record.worker_matched_source_type,
    );
    define_string(
        scope,
        output,
        "workerFinalSourceType",
        &record.worker_final_source_type,
    );
    define_number(scope, output, "redirectStart", record.redirect_start);
    define_number(scope, output, "redirectEnd", record.redirect_end);
    define_number(scope, output, "fetchStart", record.fetch_start);
    define_number(
        scope,
        output,
        "domainLookupStart",
        record.domain_lookup_start,
    );
    define_number(scope, output, "domainLookupEnd", record.domain_lookup_end);
    define_number(scope, output, "connectStart", record.connect_start);
    define_number(
        scope,
        output,
        "secureConnectionStart",
        record.secure_connection_start,
    );
    define_number(scope, output, "connectEnd", record.connect_end);
    define_number(scope, output, "requestStart", record.request_start);
    define_number(scope, output, "responseStart", record.response_start);
    define_number(
        scope,
        output,
        "firstInterimResponseStart",
        record.first_interim_response_start,
    );
    define_number(
        scope,
        output,
        "finalResponseHeadersStart",
        record.final_response_headers_start,
    );
    define_number(scope, output, "responseEnd", record.response_end);
    define_number(scope, output, "transferSize", record.transfer_size);
    define_number(scope, output, "encodedBodySize", record.encoded_body_size);
    define_number(scope, output, "decodedBodySize", record.decoded_body_size);
    define_value(
        scope,
        output,
        "responseStatus",
        v8::Integer::new(scope, record.response_status).into(),
    );
    let timings = v8::Array::new(scope, record.server_timing.len() as i32);
    for (index, timing) in record.server_timing.iter().enumerate() {
        let _ = timings.set_index(scope, index as u32, v8::Local::new(scope, timing).into());
    }
    define_value(scope, output, "serverTiming", timings.into());
    Some(output)
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(output) = to_object(scope, arguments.this()) {
        result.set(output.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PerformanceResourceTimingStore>() {
        store.constructor.remove(realm_id);
    }
}
