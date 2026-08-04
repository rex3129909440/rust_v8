use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct WebSocketStreamRecord {
    url: String,
    opened: v8::Global<v8::Promise>,
    closed: v8::Global<v8::Promise>,
    closed_resolver: v8::Global<v8::PromiseResolver>,
    is_closed: bool,
}

#[derive(Default)]
pub(crate) struct WebSocketStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WebSocketStreamRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WebSocketStreamStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WebSocketStream", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<WebSocketStreamStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WebSocketStream",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "opened", get_opened)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "closed", get_closed)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WebSocketStreamStore>()
        .ok_or_else(|| "WebSocketStream state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WebSocketStream': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WebSocketStream': 1 argument required, but only 0 present.",
        );
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(url) = normalize_url(scope, &input) else {
        crate::webidl::throw_type_error(scope, "Invalid WebSocket URL");
        return;
    };
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let protocols = read_protocols(scope, options);
    if has_duplicates(&protocols) {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The WebSocket subprotocol is duplicated.".to_owned(),
            "SyntaxError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let Ok(readable) = super::readable_stream::create_empty(scope) else {
        return;
    };
    let Ok(writable) = super::writable_stream::create_empty(scope) else {
        return;
    };
    let opened_info = v8::Object::new(scope);
    define_value(scope, opened_info, "readable", readable.into());
    define_value(scope, opened_info, "writable", writable.into());
    define_text(
        scope,
        opened_info,
        "protocol",
        protocols.first().map(String::as_str).unwrap_or(""),
    );
    define_text(scope, opened_info, "extensions", "");
    let Ok(opened) = super::writable_stream::resolved_promise(scope, opened_info.into()) else {
        return;
    };
    let Some(closed_resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let closed = closed_resolver.get_promise(scope);
    let record = WebSocketStreamRecord {
        url,
        opened: v8::Global::new(scope, opened),
        closed: v8::Global::new(scope, closed),
        closed_resolver: v8::Global::new(scope, closed_resolver),
        is_closed: false,
    };
    scope
        .get_slot_mut::<WebSocketStreamStore>()
        .expect("WebSocketStream state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn normalize_url(scope: &mut v8::PinScope<'_, '_>, input: &str) -> Option<String> {
    let mut url = if let Ok(url) = url::Url::parse(input) {
        url
    } else {
        let global = scope.get_current_context().global(scope);
        let base = v8::String::new(scope, "location")
            .and_then(|key| global.get(scope, key.into()))
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .and_then(|location| {
                v8::String::new(scope, "href")
                    .and_then(|key| location.get(scope, key.into()))
                    .map(|value| crate::webidl::value_to_string(scope, value))
            })?;
        url::Url::parse(&base).ok()?.join(input).ok()?
    };
    match url.scheme() {
        "http" => {
            let _ = url.set_scheme("ws");
        }
        "https" => {
            let _ = url.set_scheme("wss");
        }
        "ws" | "wss" => {}
        _ => return None,
    }
    Some(url.to_string())
}

fn read_protocols(
    scope: &mut v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Vec<String> {
    let Some(options) = options else {
        return Vec::new();
    };
    let Some(key) = v8::String::new(scope, "protocols") else {
        return Vec::new();
    };
    let Some(value) = options.get(scope, key.into()) else {
        return Vec::new();
    };
    let Ok(array) = v8::Local::<v8::Array>::try_from(value) else {
        return Vec::new();
    };
    let mut protocols = Vec::with_capacity(array.length() as usize);
    for index in 0..array.length() {
        if let Some(value) = array.get_index(scope, index) {
            protocols.push(crate::webidl::value_to_string(scope, value));
        }
    }
    protocols
}

fn has_duplicates(protocols: &[String]) -> bool {
    let mut seen = HashSet::new();
    protocols.iter().any(|protocol| !seen.insert(protocol))
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<WebSocketStreamRecord> {
    scope
        .get_slot::<WebSocketStreamStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.url) {
        result.set(value.into());
    }
}
fn get_opened(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.opened).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_closed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.closed).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.is_closed {
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let close_code = number_property(scope, options, "closeCode").unwrap_or(1000);
    if !super::web_socket_error::valid_close_code(close_code) {
        super::web_socket_error::throw_invalid_code(scope, close_code);
        return;
    }
    let reason = string_property(scope, options, "reason");
    let close_info = v8::Object::new(scope);
    define_value(
        scope,
        close_info,
        "closeCode",
        v8::Integer::new_from_unsigned(scope, close_code as u32).into(),
    );
    define_text(scope, close_info, "reason", &reason);
    let resolver = v8::Local::new(scope, &record.closed_resolver);
    let _ = resolver.resolve(scope, close_info.into());
    if let Some(current) = scope
        .get_slot_mut::<WebSocketStreamStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        current.is_closed = true;
    }
}

fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<u16> {
    let object = object?;
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        value
            .integer_value(scope)
            .map(|value| value.clamp(0, u16::MAX as i64) as u16)
    }
}
fn string_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> String {
    let Some(object) = object else {
        return String::new();
    };
    let Some(key) = v8::String::new(scope, name) else {
        return String::new();
    };
    let Some(value) = object.get(scope, key.into()) else {
        return String::new();
    };
    if value.is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, value)
    }
}
fn define_text(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define_value(scope, object, name, value.into());
    }
}
fn define_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::NONE);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebSocketStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
