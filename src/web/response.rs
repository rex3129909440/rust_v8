use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ResponseStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, ResponseRecord>,
}

#[derive(Clone)]
struct ResponseRecord {
    response_type: String,
    url: String,
    redirected: bool,
    status: u16,
    status_text: String,
    headers: v8::Global<v8::Object>,
    body: Option<v8::Global<v8::Object>>,
    bytes: Vec<u8>,
    body_used: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ResponseStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Response", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ResponseStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Response",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "redirected", get_redirected)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "status", get_status)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ok", get_ok)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "statusText", get_status_text)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "headers", get_headers)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "body", get_body)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "bodyUsed", get_body_used)?;
    crate::webidl::define_method(scope, prototype, "arrayBuffer", 0, array_buffer)?;
    crate::webidl::define_method(scope, prototype, "blob", 0, blob)?;
    crate::webidl::define_method(scope, prototype, "clone", 0, clone_response)?;
    crate::webidl::define_method(scope, prototype, "formData", 0, form_data)?;
    crate::webidl::define_method(scope, prototype, "json", 0, json)?;
    crate::webidl::define_method(scope, prototype, "text", 0, text)?;
    crate::webidl::define_method(scope, prototype, "bytes", 0, bytes)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "error", 0, static_error)?;
    crate::webidl::define_method(scope, constructor.into(), "json", 1, static_json)?;
    crate::webidl::define_method(scope, constructor.into(), "redirect", 1, static_redirect)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ResponseStore>()
        .ok_or_else(|| "Response state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Response': use new");
        return;
    }
    let bytes = if arguments.length() == 0 || arguments.get(0).is_null_or_undefined() {
        Vec::new()
    } else {
        body_bytes(scope, arguments.get(0))
    };
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let status = init
        .and_then(|value| property(scope, value, "status"))
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(200);
    if status != 0 && !(200..=599).contains(&status) {
        crate::webidl::throw_type_error(scope, "Response status must be between 200 and 599");
        return;
    }
    let status_text = init
        .and_then(|value| property(scope, value, "statusText"))
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let headers = match headers_from_init(scope, init) {
        Ok(headers) => headers,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let body = if arguments.length() == 0 || arguments.get(0).is_null_or_undefined() {
        None
    } else {
        match body_stream(scope, &bytes) {
            Ok(stream) => Some(stream),
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    };
    let record = ResponseRecord {
        response_type: "default".to_owned(),
        url: String::new(),
        redirected: false,
        status: status as u16,
        status_text,
        headers: v8::Global::new(scope, headers),
        body: body.map(|stream| v8::Global::new(scope, stream)),
        bytes,
        body_used: false,
    };
    scope
        .get_slot_mut::<ResponseStore>()
        .expect("Response state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn body_bytes(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Vec<u8> {
    if let Ok(bytes) = super::text_decoder::bytes_from_value(scope, value) {
        return bytes;
    }
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value)
        && let Some((bytes, _)) = super::blob::byte_snapshot(scope, object)
    {
        return bytes;
    }
    crate::webidl::value_to_string(scope, value).into_bytes()
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn headers_from_init<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    init: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let Some(value) = init.and_then(|init| property(scope, init, "headers")) else {
        return super::headers::create(scope, Vec::new());
    };
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return super::headers::create(scope, Vec::new());
    };
    if let Some(values) = super::headers::snapshot(scope, object) {
        return super::headers::create(scope, values);
    }
    let mut values = Vec::new();
    if let Some(names) = object.get_own_property_names(
        scope,
        v8::GetPropertyNamesArgs {
            mode: v8::KeyCollectionMode::OwnOnly,
            property_filter: v8::PropertyFilter::ONLY_ENUMERABLE,
            index_filter: v8::IndexFilter::IncludeIndices,
            key_conversion: v8::KeyConversionMode::ConvertToString,
        },
    ) {
        for index in 0..names.length() {
            let Some(key) = names.get_index(scope, index) else {
                continue;
            };
            let name = crate::webidl::value_to_string(scope, key);
            let value = object
                .get(scope, key)
                .map(|value| crate::webidl::value_to_string(scope, value))
                .unwrap_or_default();
            values.push((name, value));
        }
    }
    super::headers::create(scope, values)
}

fn body_stream<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    bytes: &[u8],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let stream = super::readable_stream::create_empty(scope)?;
    let chunk = super::text_encoder::uint8_array(scope, bytes.to_vec())?;
    super::readable_stream::enqueue(scope, stream, chunk.into());
    super::readable_stream::close(scope, stream);
    Ok(stream)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ResponseRecord> {
    scope
        .get_slot::<ResponseStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn validate_wasm_streaming_source(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), &'static str> {
    let Some(record) = record(scope, object) else {
        return Err("WebAssembly streaming source must be a Response");
    };
    if record.body_used {
        return Err("Cannot compile WebAssembly.Module from an already read Response");
    }
    let headers = v8::Local::new(scope, &record.headers);
    let content_type = super::headers::snapshot(scope, headers)
        .unwrap_or_default()
        .into_iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value)
        .collect::<Vec<_>>()
        .join(", ");
    let essence = content_type
        .split_once(';')
        .map_or(content_type.as_str(), |(value, _)| value)
        .trim();
    if !essence.eq_ignore_ascii_case("application/wasm") {
        return Err("Incorrect response MIME type. Expected 'application/wasm'.");
    }
    Ok(())
}

fn return_string(scope: &mut v8::PinScope<'_, '_>, value: &str, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into())
    }
}
fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.response_type, r)
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
        return_string(s, &v.url, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_redirected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.redirected).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.status as u32).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_ok(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, (200..=299).contains(&v.status)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_status_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.status_text, r)
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

fn consume(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<Vec<u8>, String> {
    let Some(record) = scope
        .get_slot_mut::<ResponseStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return Err("Illegal invocation".to_owned());
    };
    if record.body_used {
        return Err("Body has already been consumed".to_owned());
    }
    record.body_used = true;
    Ok(record.bytes.clone())
}
fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into())
    }
}
fn reject(scope: &mut v8::PinScope<'_, '_>, message: &str, mut result: v8::ReturnValue<'_>) {
    let value = v8::String::new(scope, message)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into());
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, value) {
        result.set(promise.into())
    }
}
fn bytes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match consume(s, a.this()) {
        Ok(data) => match super::text_encoder::uint8_array(s, data) {
            Ok(value) => resolve(s, value.into(), r),
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
            let backing = v8::ArrayBuffer::new_backing_store_from_vec(data).make_shared();
            let value = v8::ArrayBuffer::with_backing_store(s, &backing);
            resolve(s, value.into(), r)
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
            let value = String::from_utf8_lossy(&data);
            if let Some(value) = v8::String::new(s, &value) {
                resolve(s, value.into(), r)
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
            let object = v8::Object::new(s);
            let size_key = v8::String::new(s, "size").unwrap();
            let size = v8::Integer::new_from_unsigned(s, data.len() as u32);
            let _ = object.define_own_property(
                s,
                size_key.into(),
                size.into(),
                v8::PropertyAttribute::READ_ONLY,
            );
            resolve(s, object.into(), r)
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
        Ok(data) => {
            let object = v8::Object::new(s);
            let text = String::from_utf8_lossy(&data);
            for pair in text.split('&') {
                if let Some((name, value)) = pair.split_once('=') {
                    if let Some(key) = v8::String::new(s, name) {
                        if let Some(value) = v8::String::new(s, value) {
                            let _ = object.set(s, key.into(), value.into());
                        }
                    }
                }
            }
            resolve(s, object.into(), r)
        }
        Err(e) => reject(s, &e, r),
    }
}

fn create_from_record<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: ResponseRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create Response".to_owned());
    }
    scope
        .get_slot_mut::<ResponseStore>()
        .ok_or_else(|| "Response state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn create_fetch_response<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    url: String,
    status: u16,
    status_text: String,
    header_values: Vec<(String, String)>,
    bytes: Vec<u8>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let headers = super::headers::create(scope, header_values)?;
    let body = body_stream(scope, &bytes)
        .ok()
        .map(|stream| v8::Global::new(scope, stream));
    create_from_record(
        scope,
        ResponseRecord {
            response_type: "basic".to_owned(),
            url,
            redirected: false,
            status,
            status_text,
            headers: v8::Global::new(scope, headers),
            body,
            bytes,
            body_used: false,
        },
    )
}

fn clone_response(
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
    if let Ok(object) = create_from_record(s, value) {
        r.set(object.into())
    }
}
fn static_error(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Ok(headers) = super::headers::create(s, Vec::new()) else {
        return;
    };
    let value = ResponseRecord {
        response_type: "error".to_owned(),
        url: String::new(),
        redirected: false,
        status: 0,
        status_text: String::new(),
        headers: v8::Global::new(s, headers),
        body: None,
        bytes: Vec::new(),
        body_used: false,
    };
    if let Ok(object) = create_from_record(s, value) {
        r.set(object.into())
    }
}
fn static_redirect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let url = crate::webidl::value_to_string(s, a.get(0));
    let status = a.get(1).uint32_value(s).unwrap_or(302);
    if !matches!(status, 301 | 302 | 303 | 307 | 308) {
        crate::webidl::throw_type_error(s, "Invalid redirect status");
        return;
    }
    let Ok(headers) = super::headers::create(s, vec![("location".to_owned(), url)]) else {
        return;
    };
    let value = ResponseRecord {
        response_type: "default".to_owned(),
        url: String::new(),
        redirected: false,
        status: status as u16,
        status_text: String::new(),
        headers: v8::Global::new(s, headers),
        body: None,
        bytes: Vec::new(),
        body_used: false,
    };
    if let Ok(object) = create_from_record(s, value) {
        r.set(object.into())
    }
}
fn static_json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Ok(text) = super::structured_data::encode(s, a.get(0)) else {
        crate::webidl::throw_type_error(s, "The value cannot be encoded as structured data");
        return;
    };
    let Ok(headers) = super::headers::create(
        s,
        vec![("content-type".to_owned(), "application/json".to_owned())],
    ) else {
        return;
    };
    let body_bytes = text.into_bytes();
    let body = body_stream(s, &body_bytes)
        .ok()
        .map(|body| v8::Global::new(s, body));
    let value = ResponseRecord {
        response_type: "default".to_owned(),
        url: String::new(),
        redirected: false,
        status: 200,
        status_text: String::new(),
        headers: v8::Global::new(s, headers),
        body,
        bytes: body_bytes,
        body_used: false,
    };
    if let Ok(object) = create_from_record(s, value) {
        r.set(object.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ResponseStore>() {
        store.constructors.remove(&realm_id);
    }
}
