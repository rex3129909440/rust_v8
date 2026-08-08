use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcDtlsTransportStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DtlsTransportRecord>,
}

#[derive(Clone)]
struct DtlsTransportRecord {
    ice_transport: v8::Global<v8::Object>,
    state: String,
    on_state_change: Option<v8::Global<v8::Value>>,
    on_error: Option<v8::Global<v8::Value>>,
    remote_certificates: Vec<Vec<u8>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcDtlsTransportStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCDtlsTransport", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcDtlsTransportStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCDtlsTransport",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "iceTransport", get_ice_transport)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onstatechange",
        get_on_state_change,
        set_on_state_change,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_on_error, set_on_error)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getRemoteCertificates",
        0,
        get_remote_certificates,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcDtlsTransportStore>()
        .ok_or_else(|| "RTCDtlsTransport state was not prepared".to_owned())?
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
        "Failed to construct 'RTCDtlsTransport': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    ice_transport: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let transport = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, transport, prototype.into()) != Some(true) {
        return Err("cannot create RTCDtlsTransport".to_owned());
    }
    super::event_target::attach(scope, transport);
    let record = DtlsTransportRecord {
        ice_transport: v8::Global::new(scope, ice_transport),
        state: "new".to_owned(),
        on_state_change: None,
        on_error: None,
        remote_certificates: Vec::new(),
    };
    scope
        .get_slot_mut::<RtcDtlsTransportStore>()
        .ok_or_else(|| "RTCDtlsTransport state was not prepared".to_owned())?
        .records
        .insert(transport.get_identity_hash().get(), record);
    Ok(transport)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DtlsTransportRecord> {
    scope
        .get_slot::<RtcDtlsTransportStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_ice_transport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.ice_transport).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.state) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<v8::Global<v8::Value>>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_on_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_handler(scope, record.on_state_change, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_on_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, arguments.get(0)))
    };
    if let Some(record) = scope
        .get_slot_mut::<RtcDtlsTransportStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.on_state_change = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_handler(scope, record.on_error, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_on_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, arguments.get(0)))
    };
    if let Some(record) = scope
        .get_slot_mut::<RtcDtlsTransportStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.on_error = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_remote_certificates(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let certificates = v8::Array::new(scope, record.remote_certificates.len() as i32);
    for (index, bytes) in record.remote_certificates.into_iter().enumerate() {
        let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
        let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
        let _ = certificates.set_index(scope, index as u32, buffer.into());
    }
    result.set(certificates.into());
}
