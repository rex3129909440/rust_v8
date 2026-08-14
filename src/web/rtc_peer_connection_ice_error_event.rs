use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcPeerConnectionIceErrorEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, ErrorRecord>,
}

#[derive(Clone)]
pub(crate) struct ErrorRecord {
    pub(crate) address: Option<String>,
    pub(crate) port: Option<u32>,
    pub(crate) host_candidate: String,
    pub(crate) url: String,
    pub(crate) error_code: u32,
    pub(crate) error_text: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcPeerConnectionIceErrorEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCPeerConnectionIceErrorEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcPeerConnectionIceErrorEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCPeerConnectionIceErrorEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_peer_connection_ice_error_event_address_property::define(scope, prototype)?;
    super::rtc_peer_connection_ice_error_event_port_property::define(scope, prototype)?;
    super::rtc_peer_connection_ice_error_event_host_candidate_property::define(scope, prototype)?;
    super::rtc_peer_connection_ice_error_event_url_property::define(scope, prototype)?;
    super::rtc_peer_connection_ice_error_event_error_code_property::define(scope, prototype)?;
    super::rtc_peer_connection_ice_error_event_error_text_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcPeerConnectionIceErrorEventStore>()
        .ok_or_else(|| "RTCPeerConnectionIceErrorEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCPeerConnectionIceErrorEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCPeerConnectionIceErrorEvent': The provided value is not of type 'RTCPeerConnectionIceErrorEventInit'.",
        );
        return;
    };
    let Some(error_code_value) = property(scope, init, "errorCode") else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCPeerConnectionIceErrorEvent': Failed to read the 'errorCode' property from 'RTCPeerConnectionIceErrorEventInit': Required member is undefined.",
        );
        return;
    };
    if error_code_value.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCPeerConnectionIceErrorEvent': Failed to read the 'errorCode' property from 'RTCPeerConnectionIceErrorEventInit': Required member is undefined.",
        );
        return;
    }
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        super::event::boolean_property(scope, init, "bubbles"),
        super::event::boolean_property(scope, init, "cancelable"),
        super::event::boolean_property(scope, init, "composed"),
    );
    let record = ErrorRecord {
        address: optional_string(scope, init, "address"),
        port: optional_u32(scope, init, "port"),
        host_candidate: optional_string(scope, init, "hostCandidate").unwrap_or_default(),
        url: optional_string(scope, init, "url").unwrap_or_default(),
        error_code: error_code_value.uint32_value(scope).unwrap_or(0),
        error_text: optional_string(scope, init, "errorText").unwrap_or_default(),
    };
    scope
        .get_slot_mut::<RtcPeerConnectionIceErrorEventStore>()
        .expect("RTCPeerConnectionIceErrorEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ErrorRecord> {
    scope
        .get_slot::<RtcPeerConnectionIceErrorEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn optional_string(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let value = property(scope, object, name)?;
    if value.is_null_or_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

pub(crate) fn optional_u32(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u32> {
    let value = property(scope, object, name)?;
    if value.is_null_or_undefined() {
        None
    } else {
        value.uint32_value(scope)
    }
}

pub(crate) fn string_result(
    scope: &mut v8::PinScope<'_, '_>,
    value: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

pub(crate) fn get_address(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.address {
            Some(value) => string_result(scope, &value, result),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.port {
            Some(value) => result.set(v8::Integer::new_from_unsigned(scope, value).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_host_candidate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => string_result(scope, &record.host_candidate, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => string_result(scope, &record.url, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_error_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Integer::new_from_unsigned(scope, record.error_code).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_error_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => string_result(scope, &record.error_text, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
