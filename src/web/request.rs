use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RequestStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, RequestRecord>,
}

#[derive(Clone)]
struct RequestRecord {
    method: String,
    url: String,
    headers: v8::Global<v8::Object>,
    destination: String,
    referrer: String,
    referrer_policy: String,
    mode: String,
    credentials: String,
    cache: String,
    redirect: String,
    integrity: String,
    keepalive: bool,
    signal: v8::Global<v8::Object>,
    duplex: String,
    history_navigation: bool,
    reload_navigation: bool,
    target_address_space: String,
    body: Option<v8::Global<v8::Object>>,
    bytes: Vec<u8>,
    body_used: bool,
}

pub(crate) struct FetchRequestSnapshot {
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RequestStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Request", constructor.into())
}

pub(crate) fn create_from_input<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    input: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    constructor
        .new_instance(scope, &[input])
        .ok_or_else(|| "cannot create background fetch Request".to_owned())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RequestStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Request",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "method", get_method)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "headers", get_headers)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "destination", get_destination)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "referrer", get_referrer)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "referrerPolicy",
        get_referrer_policy,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mode", get_mode)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "credentials", get_credentials)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "cache", get_cache)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "redirect", get_redirect)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "integrity", get_integrity)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "keepalive", get_keepalive)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "signal", get_signal)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "duplex", get_duplex)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "isHistoryNavigation",
        get_history_navigation,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "bodyUsed", get_body_used)?;
    crate::webidl::define_method(scope, prototype, "arrayBuffer", 0, array_buffer)?;
    crate::webidl::define_method(scope, prototype, "blob", 0, blob)?;
    crate::webidl::define_method(scope, prototype, "clone", 0, clone_request)?;
    crate::webidl::define_method(scope, prototype, "formData", 0, form_data)?;
    crate::webidl::define_method(scope, prototype, "json", 0, json)?;
    crate::webidl::define_method(scope, prototype, "text", 0, text)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "targetAddressSpace",
        get_target_address_space,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "isReloadNavigation",
        get_reload_navigation,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "body", get_body)?;
    crate::webidl::define_method(scope, prototype, "bytes", 0, bytes)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RequestStore>()
        .ok_or_else(|| "Request state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Request'");
        return;
    }
    let input_object = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let inherited = input_object.and_then(|object| record(scope, object));
    let input_url = inherited
        .as_ref()
        .map(|record| record.url.clone())
        .unwrap_or_else(|| crate::webidl::value_to_string(scope, arguments.get(0)));
    let Ok(parsed_url) = url::Url::parse(&input_url) else {
        crate::webidl::throw_type_error(scope, "Request URL must be absolute");
        return;
    };
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let method = string_option(scope, init, "method")
        .or_else(|| inherited.as_ref().map(|record| record.method.clone()))
        .unwrap_or_else(|| "GET".to_owned());
    let method = normalize_method(&method);
    let body_override = init.and_then(|value| property(scope, value, "body"));
    let bytes = if let Some(value) = body_override {
        if value.is_null_or_undefined() {
            Vec::new()
        } else {
            crate::webidl::value_to_string(scope, value).into_bytes()
        }
    } else {
        inherited
            .as_ref()
            .map(|record| record.bytes.clone())
            .unwrap_or_default()
    };
    if matches!(method.as_str(), "GET" | "HEAD") && !bytes.is_empty() {
        crate::webidl::throw_type_error(scope, "GET or HEAD request cannot have a body");
        return;
    }
    let headers = match request_headers(scope, init, inherited.as_ref()) {
        Ok(headers) => headers,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let signal = if let Some(value) = init.and_then(|value| property(scope, value, "signal")) {
        v8::Local::<v8::Object>::try_from(value)
            .ok()
            .filter(|signal| super::abort_signal::record(scope, *signal).is_some())
            .or_else(|| super::abort_signal::create(scope, None).ok())
    } else if let Some(inherited) = inherited.as_ref() {
        Some(v8::Local::new(scope, &inherited.signal))
    } else {
        super::abort_signal::create(scope, None).ok()
    };
    let Some(signal) = signal else {
        crate::webidl::throw_type_error(scope, "Cannot create AbortSignal");
        return;
    };
    let body = if bytes.is_empty() {
        None
    } else {
        body_stream(scope, &bytes)
            .ok()
            .map(|stream| v8::Global::new(scope, stream))
    };
    let value = RequestRecord {
        method,
        url: parsed_url.to_string(),
        headers: v8::Global::new(scope, headers),
        destination: inherited
            .as_ref()
            .map(|record| record.destination.clone())
            .unwrap_or_default(),
        referrer: string_option(scope, init, "referrer")
            .or_else(|| inherited.as_ref().map(|record| record.referrer.clone()))
            .unwrap_or_else(|| "about:client".to_owned()),
        referrer_policy: option_or(
            scope,
            init,
            "referrerPolicy",
            inherited.as_ref().map(|r| r.referrer_policy.as_str()),
            "",
        ),
        mode: option_or(
            scope,
            init,
            "mode",
            inherited.as_ref().map(|r| r.mode.as_str()),
            "cors",
        ),
        credentials: option_or(
            scope,
            init,
            "credentials",
            inherited.as_ref().map(|r| r.credentials.as_str()),
            "same-origin",
        ),
        cache: option_or(
            scope,
            init,
            "cache",
            inherited.as_ref().map(|r| r.cache.as_str()),
            "default",
        ),
        redirect: option_or(
            scope,
            init,
            "redirect",
            inherited.as_ref().map(|r| r.redirect.as_str()),
            "follow",
        ),
        integrity: option_or(
            scope,
            init,
            "integrity",
            inherited.as_ref().map(|r| r.integrity.as_str()),
            "",
        ),
        keepalive: bool_option(scope, init, "keepalive")
            .or_else(|| inherited.as_ref().map(|record| record.keepalive))
            .unwrap_or(false),
        signal: v8::Global::new(scope, signal),
        duplex: "half".to_owned(),
        history_navigation: false,
        reload_navigation: false,
        target_address_space: "unknown".to_owned(),
        body,
        bytes,
        body_used: false,
    };
    scope
        .get_slot_mut::<RequestStore>()
        .expect("Request state")
        .records
        .insert(arguments.this().get_identity_hash().get(), value);
    result.set(arguments.this().into());
}

fn normalize_method(method: &str) -> String {
    let uppercase = method.to_ascii_uppercase();
    if matches!(
        uppercase.as_str(),
        "DELETE" | "GET" | "HEAD" | "OPTIONS" | "POST" | "PUT"
    ) {
        uppercase
    } else {
        method.to_owned()
    }
}
fn property<'s>(
    s: &v8::PinScope<'s, '_>,
    o: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(s, name)?;
    o.get(s, key.into())
}
fn string_option(
    s: &mut v8::PinScope<'_, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<String> {
    o.and_then(|o| property(s, o, name))
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(s, v))
}
fn bool_option(
    s: &v8::PinScope<'_, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<bool> {
    o.and_then(|o| property(s, o, name))
        .filter(|v| !v.is_undefined())
        .map(|v| v.boolean_value(s))
}
fn option_or(
    s: &mut v8::PinScope<'_, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    inherited: Option<&str>,
    default: &str,
) -> String {
    string_option(s, o, name)
        .or_else(|| inherited.map(str::to_owned))
        .unwrap_or_else(|| default.to_owned())
}
fn request_headers<'s>(
    s: &mut v8::PinScope<'s, '_>,
    init: Option<v8::Local<'s, v8::Object>>,
    inherited: Option<&RequestRecord>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if let Some(value) = init
        .and_then(|o| property(s, o, "headers"))
        .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok())
    {
        if let Some(values) = super::headers::snapshot(s, value) {
            return super::headers::create(s, values);
        }
        let mut values = Vec::new();
        if let Some(names) = value.get_own_property_names(
            s,
            v8::GetPropertyNamesArgs {
                mode: v8::KeyCollectionMode::OwnOnly,
                property_filter: v8::PropertyFilter::ONLY_ENUMERABLE,
                index_filter: v8::IndexFilter::IncludeIndices,
                key_conversion: v8::KeyConversionMode::ConvertToString,
            },
        ) {
            for index in 0..names.length() {
                let Some(key) = names.get_index(s, index) else {
                    continue;
                };
                let name = crate::webidl::value_to_string(s, key);
                let text = value
                    .get(s, key)
                    .map(|v| crate::webidl::value_to_string(s, v))
                    .unwrap_or_default();
                values.push((name, text));
            }
        }
        return super::headers::create(s, values);
    }
    let values = inherited
        .and_then(|r| super::headers::snapshot(s, v8::Local::new(s, &r.headers)))
        .unwrap_or_default();
    super::headers::create(s, values)
}
fn body_stream<'s>(
    s: &mut v8::PinScope<'s, '_>,
    bytes: &[u8],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let stream = super::readable_stream::create_empty(s)?;
    let chunk = super::text_encoder::uint8_array(s, bytes.to_vec())?;
    super::readable_stream::enqueue(s, stream, chunk.into());
    super::readable_stream::close(s, stream);
    Ok(stream)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<RequestRecord> {
    s.get_slot::<RequestStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn url(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.url)
}
pub(crate) fn method(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.method)
}

pub(crate) fn fetch_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<FetchRequestSnapshot> {
    let record = record(scope, object)?;
    let headers =
        super::headers::snapshot(scope, v8::Local::new(scope, &record.headers)).unwrap_or_default();
    Some(FetchRequestSnapshot {
        method: record.method,
        url: record.url,
        headers,
        bytes: record.bytes,
    })
}

pub(crate) fn init_headers<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    init: v8::Local<'s, v8::Object>,
) -> Option<Vec<(String, String)>> {
    property(scope, init, "headers")
        .filter(|value| !value.is_undefined())
        .and_then(|_| request_headers(scope, Some(init), None).ok())
        .and_then(|headers| super::headers::snapshot(scope, headers))
}
fn ret_string(s: &mut v8::PinScope<'_, '_>, v: &str, mut r: v8::ReturnValue<'_>) {
    if let Some(v) = v8::String::new(s, v) {
        r.set(v.into())
    }
}
fn get_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.method, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.url, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_headers(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.headers).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_destination(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.destination, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_referrer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.referrer, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.referrer_policy, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.mode, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_credentials(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.credentials, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_cache(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.cache, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_redirect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.redirect, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_integrity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.integrity, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_keepalive(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.keepalive).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_signal(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.signal).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_duplex(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.duplex, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_history_navigation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.history_navigation).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_reload_navigation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.reload_navigation).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_target_address_space(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret_string(s, &v.target_address_space, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_body_used(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.body_used).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_body(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(body) = v.body {
            r.set(v8::Local::new(s, &body).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn consume(s: &mut v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Result<Vec<u8>, String> {
    let Some(r) = s
        .get_slot_mut::<RequestStore>()
        .and_then(|store| store.records.get_mut(&o.get_identity_hash().get()))
    else {
        return Err("Illegal invocation".to_owned());
    };
    if r.body_used {
        return Err("Body has already been consumed".to_owned());
    }
    r.body_used = true;
    Ok(r.bytes.clone())
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn reject(s: &mut v8::PinScope<'_, '_>, m: &str, mut r: v8::ReturnValue<'_>) {
    let v = v8::String::new(s, m)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(s).into());
    if let Ok(p) = super::writable_stream::rejected_promise(s, v) {
        r.set(p.into())
    }
}
fn bytes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match consume(s, a.this()) {
        Ok(data) => match super::text_encoder::uint8_array(s, data) {
            Ok(v) => resolve(s, v.into(), r),
            Err(e) => reject(s, &e, r),
        },
        Err(e) => reject(s, &e, r),
    }
}
fn array_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match consume(s, a.this()) {
        Ok(data) => {
            let b = v8::ArrayBuffer::new_backing_store_from_vec(data).make_shared();
            let v = v8::ArrayBuffer::with_backing_store(s, &b);
            resolve(s, v.into(), r)
        }
        Err(e) => reject(s, &e, r),
    }
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match consume(s, a.this()) {
        Ok(data) => {
            let text = String::from_utf8_lossy(&data);
            if let Some(v) = v8::String::new(s, &text) {
                resolve(s, v.into(), r)
            }
        }
        Err(e) => reject(s, &e, r),
    }
}
fn json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match consume(s, a.this()) {
        Ok(data) => {
            let text = String::from_utf8_lossy(&data);
            match super::structured_data::decode(s, &text) {
                Ok(value) => resolve(s, value, r),
                Err(error) => reject(s, &error, r),
            }
        }
        Err(e) => reject(s, &e, r),
    }
}
fn blob(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match consume(s, a.this()) {
        Ok(data) => {
            let o = v8::Object::new(s);
            let key = v8::String::new(s, "size").unwrap();
            let size = v8::Integer::new_from_unsigned(s, data.len() as u32);
            let _ =
                o.define_own_property(s, key.into(), size.into(), v8::PropertyAttribute::READ_ONLY);
            resolve(s, o.into(), r)
        }
        Err(e) => reject(s, &e, r),
    }
}
fn form_data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match consume(s, a.this()) {
        Ok(_) => resolve(s, v8::Object::new(s).into(), r),
        Err(e) => reject(s, &e, r),
    }
}
fn create_from_record<'s>(
    s: &mut v8::PinScope<'s, '_>,
    value: RequestRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create Request".to_owned());
    }
    s.get_slot_mut::<RequestStore>()
        .ok_or_else(|| "Request state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), value);
    Ok(o)
}
fn clone_request(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(mut value) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if value.body_used {
        crate::webidl::throw_type_error(s, "Body has already been consumed");
        return;
    }
    let headers =
        super::headers::snapshot(s, v8::Local::new(s, &value.headers)).unwrap_or_default();
    let Ok(headers) = super::headers::create(s, headers) else {
        return;
    };
    value.headers = v8::Global::new(s, headers);
    value.body = if value.body.is_some() {
        body_stream(s, &value.bytes)
            .ok()
            .map(|body| v8::Global::new(s, body))
    } else {
        None
    };
    if let Ok(o) = create_from_record(s, value) {
        r.set(o.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RequestStore>() {
        store.constructors.remove(&realm_id);
    }
}
