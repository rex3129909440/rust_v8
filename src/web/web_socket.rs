use std::collections::HashMap;

const CONNECTING: i32 = 0;
const OPEN: i32 = 1;
const CLOSING: i32 = 2;
const CLOSED: i32 = 3;

#[derive(Default)]
pub(crate) struct WebSocketStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WebSocketRecord>,
}

#[derive(Clone, Copy)]
enum HandlerSlot {
    Open,
    Error,
    Close,
    Message,
}

#[derive(Clone)]
struct WebSocketRecord {
    url: String,
    ready_state: i32,
    buffered_amount: u32,
    extensions: String,
    protocol: String,
    binary_type: String,
    on_open: Option<v8::Global<v8::Value>>,
    on_error: Option<v8::Global<v8::Value>>,
    on_close: Option<v8::Global<v8::Value>>,
    on_message: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WebSocketStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WebSocket", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<WebSocketStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "WebSocket",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readyState", get_ready_state)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "bufferedAmount",
        get_buffered_amount,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onopen", get_on_open, set_on_open)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_on_error, set_on_error)?;
    crate::webidl::define_accessor(scope, prototype, "onclose", get_on_close, set_on_close)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "extensions", get_extensions)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "protocol", get_protocol)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onmessage",
        get_on_message,
        set_on_message,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "binaryType",
        get_binary_type,
        set_binary_type,
    )?;
    define_socket_constants(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "send", 1, send)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_socket_constants(scope, constructor.into())?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WebSocketStore>()
        .ok_or_else(|| "WebSocket state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_socket_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "CONNECTING", CONNECTING)?;
    crate::webidl::define_constant(scope, object, "OPEN", OPEN)?;
    crate::webidl::define_constant(scope, object, "CLOSING", CLOSING)?;
    crate::webidl::define_constant(scope, object, "CLOSED", CLOSED)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WebSocket': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WebSocket': 1 argument required",
        );
        return;
    }
    let raw_url = crate::webidl::value_to_string(scope, arguments.get(0));
    let parsed = match url::Url::parse(&raw_url) {
        Ok(parsed) if parsed.scheme() == "ws" || parsed.scheme() == "wss" => parsed,
        _ => {
            crate::webidl::throw_type_error(scope, "The WebSocket URL is invalid");
            return;
        }
    };
    let protocol = first_protocol(scope, arguments.get(1)).unwrap_or_default();
    let object = arguments.this();
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<WebSocketStore>()
        .expect("WebSocket state")
        .records
        .insert(
            object.get_identity_hash().get(),
            WebSocketRecord {
                url: parsed.to_string(),
                ready_state: CONNECTING,
                buffered_amount: 0,
                extensions: String::new(),
                protocol,
                binary_type: "blob".to_owned(),
                on_open: None,
                on_error: None,
                on_close: None,
                on_message: None,
            },
        );
    result.set(object.into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<WebSocketRecord> {
    scope
        .get_slot::<WebSocketStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut WebSocketRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<WebSocketStore>()
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

fn get_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        return_string(s, &mut r, &record.url)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_ready_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Integer::new(s, record.ready_state).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_buffered_amount(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, record.buffered_amount).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_extensions(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        return_string(s, &mut r, &record.extensions)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        return_string(s, &mut r, &record.protocol)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_binary_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        return_string(s, &mut r, &record.binary_type)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_binary_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "blob" && value != "arraybuffer" {
        return;
    }
    update(scope, arguments.this(), |record| record.binary_type = value);
}

fn handler(record: &WebSocketRecord, slot: HandlerSlot) -> Option<v8::Global<v8::Value>> {
    match slot {
        HandlerSlot::Open => record.on_open.clone(),
        HandlerSlot::Error => record.on_error.clone(),
        HandlerSlot::Close => record.on_close.clone(),
        HandlerSlot::Message => record.on_message.clone(),
    }
}

fn assign(record: &mut WebSocketRecord, slot: HandlerSlot, value: Option<v8::Global<v8::Value>>) {
    match slot {
        HandlerSlot::Open => record.on_open = value,
        HandlerSlot::Error => record.on_error = value,
        HandlerSlot::Close => record.on_close = value,
        HandlerSlot::Message => record.on_message = value,
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    slot: HandlerSlot,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = handler(&record, slot) {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    slot: HandlerSlot,
) {
    let value = arguments.get(0);
    let value = value.is_function().then(|| v8::Global::new(scope, value));
    update(scope, arguments.this(), |record| {
        assign(record, slot, value)
    });
}

fn get_on_open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::Open);
}
fn set_on_open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::Open);
}
fn get_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::Error);
}
fn set_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::Error);
}
fn get_on_close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::Close);
}
fn set_on_close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::Close);
}
fn get_on_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::Message);
}
fn set_on_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::Message);
}

fn fire(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    name: &str,
    slot: HandlerSlot,
    data: Option<v8::Local<'_, v8::Value>>,
) {
    let event = super::event_target::create_event(scope, name);
    if let Some(data) = data {
        define_data(scope, event, "data", data);
    }
    let callback = record(scope, target).and_then(|record| handler(&record, slot));
    if let Some(callback) = callback {
        if let Ok(callback) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &callback))
        {
            let _ = callback.call(scope, target.into(), &[event.into()]);
        }
    }
    super::event_target::dispatch(scope, target, event);
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let code = if arguments.get(0).is_undefined() {
        1000
    } else {
        arguments.get(0).int32_value(scope).unwrap_or(1000)
    };
    if code != 1000 && !(3000..=4999).contains(&code) {
        crate::webidl::throw_type_error(scope, "Invalid WebSocket close code");
        return;
    }
    let target = arguments.this();
    let Some(snapshot) = record(scope, target) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.ready_state == CLOSED {
        return;
    }
    update(scope, target, |record| record.ready_state = CLOSED);
    let event = super::event_target::create_event(scope, "close");
    define_data(scope, event, "code", v8::Integer::new(scope, code).into());
    let reason = arguments
        .get(1)
        .to_string(scope)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into());
    define_data(scope, event, "reason", reason);
    define_data(
        scope,
        event,
        "wasClean",
        v8::Boolean::new(scope, true).into(),
    );
    if let Some(callback) = snapshot.on_close {
        if let Ok(callback) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &callback))
        {
            let _ = callback.call(scope, target.into(), &[event.into()]);
        }
    }
    super::event_target::dispatch(scope, target, event);
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
    if snapshot.ready_state == CLOSED || snapshot.ready_state == CLOSING {
        return;
    }
    if snapshot.ready_state == CONNECTING {
        update(scope, target, |record| record.ready_state = OPEN);
        fire(scope, target, "open", HandlerSlot::Open, None);
    }
    let value = arguments.get(0);
    let size = value
        .to_string(scope)
        .map(|value| value.length())
        .unwrap_or(0)
        .min(u32::MAX as usize) as u32;
    update(scope, target, |record| record.buffered_amount = size);
    fire(scope, target, "message", HandlerSlot::Message, Some(value));
    update(scope, target, |record| record.buffered_amount = 0);
}

fn first_protocol(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Option<String> {
    if value.is_undefined() {
        return None;
    }
    if let Ok(array) = v8::Local::<v8::Array>::try_from(value) {
        return array
            .get_index(scope, 0)
            .map(|value| crate::webidl::value_to_string(scope, value));
    }
    Some(crate::webidl::value_to_string(scope, value))
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebSocketStore>() {
        store.constructor.remove(realm_id);
    }
}
