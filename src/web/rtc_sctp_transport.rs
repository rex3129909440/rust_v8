use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcSctpTransportStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SctpRecord>,
}

#[derive(Clone)]
struct SctpRecord {
    transport: Option<v8::Global<v8::Object>>,
    state: String,
    max_message_size: f64,
    max_channels: Option<u32>,
    on_state_change: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcSctpTransportStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCSctpTransport", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcSctpTransportStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCSctpTransport",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transport", get_transport)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxMessageSize",
        get_max_message_size,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "maxChannels", get_max_channels)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onstatechange",
        get_on_state_change,
        set_on_state_change,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcSctpTransportStore>()
        .ok_or_else(|| "RTCSctpTransport state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    transport: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create RTCSctpTransport".to_owned());
    }
    super::event_target::attach(scope, object);
    let record = SctpRecord {
        transport: transport.map(|transport| v8::Global::new(scope, transport)),
        state: "connecting".to_owned(),
        max_message_size: 0.0,
        max_channels: None,
        on_state_change: None,
    };
    scope
        .get_slot_mut::<RtcSctpTransportStore>()
        .ok_or_else(|| "RTCSctpTransport state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCSctpTransport': Illegal constructor",
    );
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<SctpRecord> {
    scope
        .get_slot::<RtcSctpTransportStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_transport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.transport {
            Some(value) => result.set(v8::Local::new(scope, &value).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => {
            if let Some(value) = v8::String::new(scope, &record.state) {
                result.set(value.into());
            }
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_max_message_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Number::new(scope, record.max_message_size).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_max_channels(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.max_channels {
            Some(value) => result.set(v8::Integer::new_from_unsigned(scope, value).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_on_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.on_state_change {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_on_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let value = if value.is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    };
    if let Some(record) = scope
        .get_slot_mut::<RtcSctpTransportStore>()
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
