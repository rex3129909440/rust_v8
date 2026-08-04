use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MessageEventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, MessageEventRecord>,
}

#[derive(Clone)]
pub(crate) struct MessageEventRecord {
    pub(crate) data: v8::Global<v8::Value>,
    pub(crate) origin: String,
    pub(crate) last_event_id: String,
    pub(crate) source: Option<v8::Global<v8::Value>>,
    pub(crate) ports: Vec<v8::Global<v8::Object>>,
    pub(crate) user_activation: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MessageEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MessageEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MessageEventStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "MessageEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::message_event_data_property::define(scope, prototype)?;
    super::message_event_origin_property::define(scope, prototype)?;
    super::message_event_last_event_id_property::define(scope, prototype)?;
    super::message_event_source_property::define(scope, prototype)?;
    super::message_event_ports_property::define(scope, prototype)?;
    super::message_event_user_activation_property::define(scope, prototype)?;
    super::message_event_init_message_event::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MessageEventStore>()
        .ok_or_else(|| "MessageEvent state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create_uninitialized<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let event_type = crate::webidl::string(scope, "")?;
    constructor
        .new_instance(scope, &[event_type.into()])
        .ok_or_else(|| "cannot create MessageEvent".to_owned())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MessageEvent': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MessageEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles = init.is_some_and(|object| boolean_property(scope, object, "bubbles"));
    let cancelable = init.is_some_and(|object| boolean_property(scope, object, "cancelable"));
    let composed = init.is_some_and(|object| boolean_property(scope, object, "composed"));
    let data = init
        .and_then(|object| property(scope, object, "data"))
        .unwrap_or_else(|| v8::null(scope).into());
    let origin = init
        .and_then(|object| property(scope, object, "origin"))
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let last_event_id = init
        .and_then(|object| property(scope, object, "lastEventId"))
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let source = init
        .and_then(|object| property(scope, object, "source"))
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| v8::Global::new(scope, value));
    let ports = init
        .and_then(|object| property(scope, object, "ports"))
        .map(|value| read_ports(scope, value))
        .unwrap_or_default();
    attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
        data,
        origin,
        last_event_id,
        source,
        ports,
        None,
    );
    result.set(arguments.this().into());
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    data: v8::Local<'_, v8::Value>,
    origin: &str,
    source: Option<v8::Local<'_, v8::Value>>,
    ports: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create MessageEvent".to_owned());
    }
    let source = source.map(|value| v8::Global::new(scope, value));
    let ports = ports
        .into_iter()
        .map(|port| v8::Global::new(scope, port))
        .collect();
    attach(
        scope,
        event,
        event_type.to_owned(),
        false,
        false,
        false,
        data,
        origin.to_owned(),
        String::new(),
        source,
        ports,
        None,
    );
    Ok(event)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    data: v8::Local<'_, v8::Value>,
    origin: String,
    last_event_id: String,
    source: Option<v8::Global<v8::Value>>,
    ports: Vec<v8::Global<v8::Object>>,
    user_activation: Option<v8::Global<v8::Object>>,
) {
    super::event::attach(scope, event, event_type, bubbles, cancelable, composed);
    let record = MessageEventRecord {
        data: v8::Global::new(scope, data),
        origin,
        last_event_id,
        source,
        ports,
        user_activation,
    };
    if let Some(store) = scope.get_slot_mut::<MessageEventStore>() {
        store
            .records
            .insert(event.get_identity_hash().get(), record);
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MessageEventRecord> {
    scope
        .get_slot::<MessageEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.data));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MessageEventRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

pub(crate) fn get_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.origin);
}

pub(crate) fn get_last_event_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.last_event_id);
}

pub(crate) fn get_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(source) = record.source {
        result.set(v8::Local::new(scope, &source));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_ports(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let ports = v8::Array::new(scope, record.ports.len() as i32);
    for (index, port) in record.ports.iter().enumerate() {
        let _ = ports.set_index(scope, index as u32, v8::Local::new(scope, port).into());
    }
    result.set(ports.into());
}

pub(crate) fn get_user_activation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(activation) = record.user_activation {
        result.set(v8::Local::new(scope, &activation).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn init_message_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let data = arguments.get(3);
    let origin = crate::webidl::value_to_string(scope, arguments.get(4));
    let last_event_id = crate::webidl::value_to_string(scope, arguments.get(5));
    let source = if arguments.get(6).is_null_or_undefined() {
        None
    } else if arguments.get(6).is_object() {
        Some(v8::Global::new(scope, arguments.get(6)))
    } else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'initMessageEvent': parameter 7 is not of type 'EventTarget'",
        );
        return;
    };
    let ports = read_ports(scope, arguments.get(7));
    attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
        data,
        origin,
        last_event_id,
        source,
        ports,
        None,
    );
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn boolean_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    property(scope, object, name).is_some_and(|value| value.boolean_value(scope))
}

pub(crate) fn read_ports(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Vec<v8::Global<v8::Object>> {
    let Ok(array) = v8::Local::<v8::Array>::try_from(value) else {
        return Vec::new();
    };
    let mut ports = Vec::new();
    for index in 0..array.length() {
        if let Some(port) = array
            .get_index(scope, index)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        {
            ports.push(v8::Global::new(scope, port));
        }
    }
    ports
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<MessageEventStore>() {
        store.constructors.remove(&realm_id);
    }
}
