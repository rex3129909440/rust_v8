use std::collections::HashMap;

pub(crate) const CONNECTING: i32 = 0;
pub(crate) const OPEN: i32 = 1;
pub(crate) const CLOSED: i32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SourceHandler {
    Open,
    Message,
    Error,
}

#[derive(Clone)]
pub(crate) struct EventSourceRecord {
    pub(crate) object: v8::Global<v8::Object>,
    pub(crate) url: String,
    pub(crate) with_credentials: bool,
    pub(crate) ready_state: i32,
    pub(crate) handlers: HashMap<SourceHandler, v8::Global<v8::Function>>,
}

#[derive(Default)]
pub(crate) struct EventSourceStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, EventSourceRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(EventSourceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "EventSource", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<EventSourceStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "EventSource",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::event_source_url_property::define(scope, prototype)?;
    super::event_source_with_credentials_property::define(scope, prototype)?;
    super::event_source_ready_state_property::define(scope, prototype)?;
    super::event_source_onopen_property::define(scope, prototype)?;
    super::event_source_onmessage_property::define(scope, prototype)?;
    super::event_source_onerror_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    super::event_source_close::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<EventSourceStore>()
        .ok_or_else(|| "EventSource state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "CONNECTING", CONNECTING)?;
    crate::webidl::define_constant(scope, object, "OPEN", OPEN)?;
    crate::webidl::define_constant(scope, object, "CLOSED", CLOSED)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'EventSource': 1 argument required",
        );
        return;
    }
    let url = crate::webidl::value_to_string(scope, arguments.get(0));
    if ::url::Url::parse(&url).is_err() {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The URL is invalid".to_owned(),
            "SyntaxError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let with_credentials = v8::Local::<v8::Object>::try_from(arguments.get(1))
        .ok()
        .is_some_and(|object| super::event::boolean_property(scope, object, "withCredentials"));
    super::event_target::attach(scope, arguments.this());
    let id = arguments.this().get_identity_hash().get();
    let record = EventSourceRecord {
        object: v8::Global::new(scope, arguments.this()),
        url: url.clone(),
        with_credentials,
        ready_state: CONNECTING,
        handlers: HashMap::new(),
    };
    scope
        .get_slot_mut::<EventSourceStore>()
        .expect("EventSource state")
        .records
        .insert(id, record);
    if url.starts_with("data:") {
        let data = v8::Integer::new(scope, id);
        if let Some(task) = v8::Function::builder(open_data_source)
            .data(data.into())
            .length(0)
            .constructor_behavior(v8::ConstructorBehavior::Throw)
            .build(scope)
        {
            scope.enqueue_microtask(task);
        }
    }
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<EventSourceRecord> {
    scope
        .get_slot::<EventSourceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.url) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_with_credentials(
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

pub(crate) fn get_ready_state(
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

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    slot: SourceHandler,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.handlers.get(&slot) {
            Some(value) => result.set(v8::Local::new(scope, value).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    slot: SourceHandler,
) {
    let value = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|function| v8::Global::new(scope, function));
    let Some(record) = scope.get_slot_mut::<EventSourceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = value {
        record.handlers.insert(slot, value);
    } else {
        record.handlers.remove(&slot);
    }
}

pub(crate) fn get_on_open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, SourceHandler::Open)
}
pub(crate) fn set_on_open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, SourceHandler::Open)
}
pub(crate) fn get_on_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, SourceHandler::Message)
}
pub(crate) fn set_on_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, SourceHandler::Message)
}
pub(crate) fn get_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, SourceHandler::Error)
}
pub(crate) fn set_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, SourceHandler::Error)
}

pub(crate) fn fire(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    slot: SourceHandler,
) {
    let handler = record(scope, target).and_then(|record| record.handlers.get(&slot).cloned());
    if let Some(handler) = handler {
        let handler = v8::Local::new(scope, &handler);
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
    super::event_target::dispatch(scope, target, event);
}

pub(crate) fn open_data_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(id) = arguments.data().int32_value(scope) else {
        return;
    };
    let Some(record) = scope
        .get_slot::<EventSourceStore>()
        .and_then(|store| store.records.get(&id))
        .cloned()
    else {
        return;
    };
    if record.ready_state == CLOSED {
        return;
    }
    if let Some(stored) = scope
        .get_slot_mut::<EventSourceStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        stored.ready_state = OPEN;
    }
    let target = v8::Local::new(scope, &record.object);
    if let Ok(event) = super::event::create(scope, "open") {
        fire(scope, target, event, SourceHandler::Open);
    }
    if let Some(message) = data_url_message(&record.url)
        && let Some(data) = v8::String::new(scope, &message)
        && let Ok(event) =
            super::message_event::create(scope, "message", data.into(), "", None, Vec::new())
    {
        fire(scope, target, event, SourceHandler::Message);
    }
}

pub(crate) fn data_url_message(url: &str) -> Option<String> {
    let (_, payload) = url.split_once(',')?;
    let decoded = percent_decode(payload);
    let mut lines = Vec::new();
    for line in decoded.lines() {
        if let Some(value) = line.strip_prefix("data:") {
            lines.push(value.strip_prefix(' ').unwrap_or(value));
        }
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

pub(crate) fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) = (hex(bytes[index + 1]), hex(bytes[index + 2]))
        {
            output.push((high << 4) | low);
            index += 3;
            continue;
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

pub(crate) fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<EventSourceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.ready_state = CLOSED;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<EventSourceStore>() {
        store.constructor.remove(realm_id);
    }
}
