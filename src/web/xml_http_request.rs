use std::collections::HashMap;

const UNSENT: i32 = 0;
const OPENED: i32 = 1;
const HEADERS_RECEIVED: i32 = 2;
const LOADING: i32 = 3;
const DONE: i32 = 4;

#[derive(Default)]
pub(crate) struct XmlHttpRequestStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RequestRecord>,
}

#[derive(Clone)]
struct RequestRecord {
    ready_state: i32,
    timeout: u32,
    with_credentials: bool,
    upload: v8::Global<v8::Object>,
    response_url: String,
    status: u16,
    status_text: String,
    response_type: String,
    response_text: String,
    response_mime: String,
    response_headers: Vec<(String, String)>,
    method: String,
    url: String,
    request_headers: Vec<(String, String)>,
    override_mime: Option<String>,
    on_ready_state_change: Option<v8::Global<v8::Value>>,
    attribution_reporting: Option<v8::Global<v8::Value>>,
    private_token: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XmlHttpRequestStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XMLHttpRequest", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<XmlHttpRequestStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::xml_http_request_event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "XMLHttpRequest",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onreadystatechange",
        get_on_ready_state_change,
        set_on_ready_state_change,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readyState", get_ready_state)?;
    crate::webidl::define_accessor(scope, prototype, "timeout", get_timeout, set_timeout)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "withCredentials",
        get_with_credentials,
        set_with_credentials,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "upload", get_upload)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "responseURL", get_response_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "status", get_status)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "statusText", get_status_text)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "responseType",
        get_response_type,
        set_response_type,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "response", get_response)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "responseText", get_response_text)?;
    define_request_constants(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getAllResponseHeaders",
        0,
        get_all_response_headers,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getResponseHeader",
        1,
        get_response_header,
    )?;
    crate::webidl::define_method(scope, prototype, "open", 2, open)?;
    crate::webidl::define_method(scope, prototype, "overrideMimeType", 1, override_mime_type)?;
    crate::webidl::define_method(scope, prototype, "send", 0, send)?;
    crate::webidl::define_method(scope, prototype, "setRequestHeader", 2, set_request_header)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "responseXML", get_response_xml)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setAttributionReporting",
        1,
        set_attribution_reporting,
    )?;
    crate::webidl::define_method(scope, prototype, "setPrivateToken", 1, set_private_token)?;
    define_request_constants(scope, constructor.into())?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XmlHttpRequestStore>()
        .ok_or_else(|| "XMLHttpRequest state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_request_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "UNSENT", UNSENT)?;
    crate::webidl::define_constant(scope, object, "OPENED", OPENED)?;
    crate::webidl::define_constant(scope, object, "HEADERS_RECEIVED", HEADERS_RECEIVED)?;
    crate::webidl::define_constant(scope, object, "LOADING", LOADING)?;
    crate::webidl::define_constant(scope, object, "DONE", DONE)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'XMLHttpRequest': Please use the 'new' operator",
        );
        return;
    }
    let upload = match super::xml_http_request_upload::create(scope) {
        Ok(upload) => upload,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let object = arguments.this();
    super::xml_http_request_event_target::attach(scope, object);
    let upload = v8::Global::new(scope, upload);
    scope
        .get_slot_mut::<XmlHttpRequestStore>()
        .expect("XMLHttpRequest state")
        .records
        .insert(
            object.get_identity_hash().get(),
            RequestRecord {
                ready_state: UNSENT,
                timeout: 0,
                with_credentials: false,
                upload,
                response_url: String::new(),
                status: 0,
                status_text: String::new(),
                response_type: String::new(),
                response_text: String::new(),
                response_mime: String::new(),
                response_headers: Vec::new(),
                method: String::new(),
                url: String::new(),
                request_headers: Vec::new(),
                override_mime: None,
                on_ready_state_change: None,
                attribution_reporting: None,
                private_token: None,
            },
        );
    result.set(object.into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RequestRecord> {
    scope
        .get_slot::<XmlHttpRequestStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut RequestRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<XmlHttpRequestStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(record);
    true
}

fn return_string(scope: &v8::PinScope<'_, '_>, result: &mut v8::ReturnValue<'_>, value: &str) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn fire_ready_state_change(scope: &mut v8::PinScope<'_, '_>, target: v8::Local<'_, v8::Object>) {
    let event = super::event_target::create_event(scope, "readystatechange");
    let handler = record(scope, target).and_then(|record| record.on_ready_state_change);
    if let Some(handler) = handler {
        if let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) {
            let _ = handler.call(scope, target.into(), &[event.into()]);
        }
    }
    super::event_target::dispatch(scope, target, event);
}

fn get_on_ready_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.on_ready_state_change {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_on_ready_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let value = value.is_function().then(|| v8::Global::new(scope, value));
    update(scope, arguments.this(), |record| {
        record.on_ready_state_change = value
    });
}

fn get_ready_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.ready_state).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.timeout).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let timeout = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| record.timeout = timeout);
}

fn get_with_credentials(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.with_credentials).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_with_credentials(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.with_credentials = value
    });
}

fn get_upload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.upload).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_response_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.response_url);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_status(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, u32::from(record.status)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_status_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.status_text);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_response_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.response_type);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_response_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.ready_state == LOADING || snapshot.ready_state == DONE {
        crate::webidl::throw_type_error(scope, "responseType cannot be changed now");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.response_type = value
    });
}

fn get_response(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.ready_state != DONE {
        result.set(v8::null(scope).into());
        return;
    }
    return_string(scope, &mut result, &record.response_text);
}

fn get_response_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    return_string(scope, &mut result, &record.response_text);
}

fn get_response_xml(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = arguments.this();
    let Some(snapshot) = record(scope, target) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let active = snapshot.ready_state != UNSENT && snapshot.ready_state != DONE;
    update(scope, target, |record| {
        record.ready_state = UNSENT;
        record.status = 0;
        record.status_text.clear();
        record.response_text.clear();
    });
    if active {
        super::xml_http_request_event_target::fire(
            scope,
            target,
            "abort",
            super::xml_http_request_event_target::ProgressHandler::Abort,
        );
        super::xml_http_request_event_target::fire(
            scope,
            target,
            "loadend",
            super::xml_http_request_event_target::ProgressHandler::LoadEnd,
        );
    }
}

fn get_all_response_headers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = if record.ready_state < HEADERS_RECEIVED {
        String::new()
    } else {
        record
            .response_headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}\r\n"))
            .collect()
    };
    return_string(scope, &mut result, &value);
}

fn get_response_header(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = record
        .response_headers
        .iter()
        .filter(|(header, _)| header.eq_ignore_ascii_case(&name))
        .map(|(_, value)| value.as_str())
        .collect::<Vec<_>>();
    if record.ready_state >= HEADERS_RECEIVED && !values.is_empty() {
        return_string(scope, &mut result, &values.join(", "));
        return;
    }
    result.set(v8::null(scope).into());
}

fn open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "XMLHttpRequest.open requires 2 arguments");
        return;
    }
    let method = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_uppercase();
    let input_url = crate::webidl::value_to_string(scope, arguments.get(1));
    if method.is_empty() || input_url.is_empty() {
        crate::webidl::throw_type_error(scope, "Invalid request method or URL");
        return;
    }
    let url = match super::fetch_global::resolve_request_url(scope, &input_url) {
        Ok(url) => url,
        Err(_) => {
            crate::webidl::throw_type_error(scope, "Invalid request method or URL");
            return;
        }
    };
    let target = arguments.this();
    if !update(scope, target, |record| {
        record.ready_state = OPENED;
        record.method = method;
        record.url = url;
        record.request_headers.clear();
        record.response_url.clear();
        record.status = 0;
        record.status_text.clear();
        record.response_text.clear();
        record.response_mime.clear();
        record.response_headers.clear();
    }) {
        return;
    }
    fire_ready_state_change(scope, target);
}

fn override_mime_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mime = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        record.override_mime = Some(mime)
    });
}

fn send(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = arguments.this();
    let Some(snapshot) = record(scope, target) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.ready_state != OPENED {
        crate::webidl::throw_type_error(scope, "XMLHttpRequest is not opened");
        return;
    }
    let (body, content_type) = if matches!(snapshot.method.as_str(), "GET" | "HEAD") {
        (Vec::new(), None)
    } else {
        crate::network_capture::body_bytes(scope, arguments.get(0))
    };
    let mut request_headers = snapshot.request_headers.clone();
    crate::network_capture::append_content_type_if_missing(&mut request_headers, content_type);
    crate::network_capture::record(
        scope,
        crate::NetworkRequestSource::XmlHttpRequest,
        snapshot.method.clone(),
        snapshot.url.clone(),
        request_headers,
        body,
    );
    super::xml_http_request_event_target::fire(
        scope,
        target,
        "loadstart",
        super::xml_http_request_event_target::ProgressHandler::LoadStart,
    );
    let start_time = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
    let replay = crate::network_replay::lookup(scope, &snapshot.method, &snapshot.url);
    let replay_for_timing = replay.clone();
    match decode_data_url(&snapshot.url)
        .map(|(mime, body)| {
            (
                200,
                "OK".to_owned(),
                vec![("content-type".to_owned(), mime.clone())],
                mime,
                body,
            )
        })
        .or_else(|| {
            replay.map(|entry| {
                let mime = entry
                    .headers
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                    .map(|(_, value)| value.clone())
                    .unwrap_or_default();
                (
                    entry.status,
                    entry.status_text,
                    entry.headers,
                    mime,
                    String::from_utf8_lossy(&entry.body).into_owned(),
                )
            })
        }) {
        Some((status, status_text, headers, mime, body)) => {
            let mime = snapshot.override_mime.unwrap_or(mime);
            update(scope, target, |record| {
                record.ready_state = HEADERS_RECEIVED;
                record.status = status;
                record.status_text = status_text;
                record.response_url = record.url.clone();
                record.response_mime = mime;
                record.response_headers = headers;
            });
            fire_ready_state_change(scope, target);
            update(scope, target, |record| {
                record.ready_state = LOADING;
                record.response_text = body;
            });
            fire_ready_state_change(scope, target);
            super::xml_http_request_event_target::fire(
                scope,
                target,
                "progress",
                super::xml_http_request_event_target::ProgressHandler::Progress,
            );
            update(scope, target, |record| record.ready_state = DONE);
            fire_ready_state_change(scope, target);
            if let Some(replay) = replay_for_timing.as_ref() {
                super::performance_resource_timing::record_network_replay(
                    scope,
                    replay,
                    "xmlhttprequest",
                    start_time,
                );
            }
            super::xml_http_request_event_target::fire(
                scope,
                target,
                "load",
                super::xml_http_request_event_target::ProgressHandler::Load,
            );
        }
        None => {
            update(scope, target, |record| {
                record.ready_state = DONE;
                record.status = 0;
                record.status_text.clear();
                record.response_text.clear();
            });
            fire_ready_state_change(scope, target);
            super::xml_http_request_event_target::fire(
                scope,
                target,
                "error",
                super::xml_http_request_event_target::ProgressHandler::Error,
            );
        }
    }
    super::xml_http_request_event_target::fire(
        scope,
        target,
        "loadend",
        super::xml_http_request_event_target::ProgressHandler::LoadEnd,
    );
}

fn set_request_header(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "setRequestHeader requires 2 arguments");
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.ready_state != OPENED {
        crate::webidl::throw_type_error(scope, "XMLHttpRequest is not opened");
        return;
    }
    update(scope, arguments.this(), |record| {
        if let Some(existing) = record
            .request_headers
            .iter_mut()
            .find(|(header, _)| header.eq_ignore_ascii_case(&name))
        {
            existing.1.push_str(", ");
            existing.1.push_str(&value);
        } else {
            record.request_headers.push((name, value));
        }
    });
}

fn set_attribution_reporting(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = v8::Global::new(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<XmlHttpRequestStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.attribution_reporting = Some(value);
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_private_token(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = v8::Global::new(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<XmlHttpRequestStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.private_token = Some(value);
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn decode_data_url(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("data:")?;
    let (metadata, data) = rest.split_once(',')?;
    let mut mime = "text/plain;charset=US-ASCII".to_owned();
    let mut encoded = false;
    if !metadata.is_empty() {
        for part in metadata.split(';') {
            if part.eq_ignore_ascii_case("base64") {
                encoded = true;
            } else if part.contains('/') {
                mime = part.to_owned();
            } else if part.to_ascii_lowercase().starts_with("charset=") {
                mime.push(';');
                mime.push_str(part);
            }
        }
    }
    let bytes = if encoded {
        decode_base64(data)?
    } else {
        decode_percent(data)?
    };
    Some((mime, String::from_utf8_lossy(&bytes).into_owned()))
}

fn decode_percent(value: &str) -> Option<Vec<u8>> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex(*bytes.get(index + 1)?)?;
            let low = hex(*bytes.get(index + 2)?)?;
            output.push((high << 4) | low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    Some(output)
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn decode_base64(value: &str) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(value.len() * 3 / 4);
    let mut quartet = [0_u8; 4];
    let mut count = 0;
    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        quartet[count] = if byte == b'=' {
            64
        } else {
            base64_value(byte)?
        };
        count += 1;
        if count == 4 {
            output.push((quartet[0] << 2) | (quartet[1] >> 4));
            if quartet[2] != 64 {
                output.push((quartet[1] << 4) | (quartet[2] >> 2));
            }
            if quartet[3] != 64 {
                output.push((quartet[2] << 6) | quartet[3]);
            }
            count = 0;
        }
    }
    (count == 0).then_some(output)
}

fn base64_value(value: u8) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<XmlHttpRequestStore>() {
        store.constructor.remove(realm_id);
    }
}
