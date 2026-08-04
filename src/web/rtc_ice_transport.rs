use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcIceTransportStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TransportRecord>,
}

#[derive(Clone)]
struct TransportRecord {
    role: String,
    state: String,
    gathering_state: String,
    on_state_change: Option<v8::Global<v8::Value>>,
    on_gathering_state_change: Option<v8::Global<v8::Value>>,
    on_selected_candidate_pair_change: Option<v8::Global<v8::Value>>,
    local_candidates: Vec<v8::Global<v8::Object>>,
    remote_candidates: Vec<v8::Global<v8::Object>>,
    local_parameters: Option<v8::Global<v8::Object>>,
    remote_parameters: Option<v8::Global<v8::Object>>,
    selected_pair: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcIceTransportStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCIceTransport", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcIceTransportStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCIceTransport",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "role", get_role)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "gatheringState",
        get_gathering_state,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onstatechange",
        get_on_state_change,
        set_on_state_change,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ongatheringstatechange",
        get_on_gathering_state_change,
        set_on_gathering_state_change,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onselectedcandidatepairchange",
        get_on_selected_candidate_pair_change,
        set_on_selected_candidate_pair_change,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getLocalCandidates",
        0,
        get_local_candidates,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getLocalParameters",
        0,
        get_local_parameters,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getRemoteCandidates",
        0,
        get_remote_candidates,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getRemoteParameters",
        0,
        get_remote_parameters,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getSelectedCandidatePair",
        0,
        get_selected_candidate_pair,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcIceTransportStore>()
        .ok_or_else(|| "RTCIceTransport state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let transport = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, transport, prototype.into()) != Some(true) {
        return Err("cannot create RTCIceTransport".to_owned());
    }
    super::event_target::attach(scope, transport);
    scope
        .get_slot_mut::<RtcIceTransportStore>()
        .ok_or_else(|| "RTCIceTransport state was not prepared".to_owned())?
        .records
        .insert(
            transport.get_identity_hash().get(),
            TransportRecord {
                role: "controlled".to_owned(),
                state: "new".to_owned(),
                gathering_state: "new".to_owned(),
                on_state_change: None,
                on_gathering_state_change: None,
                on_selected_candidate_pair_change: None,
                local_candidates: Vec::new(),
                remote_candidates: Vec::new(),
                local_parameters: None,
                remote_parameters: None,
                selected_pair: None,
            },
        );
    Ok(transport)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCIceTransport': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TransportRecord> {
    scope
        .get_slot::<RtcIceTransportStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut TransportRecord),
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<RtcIceTransportStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
        true
    } else {
        false
    }
}

fn return_string(scope: &mut v8::PinScope<'_, '_>, value: &str, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn get_role(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.role, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.state, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_gathering_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.gathering_state, result);
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

fn handler_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    if value.is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    }
}

fn get_on_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_handler(scope, record.on_state_change, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_on_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(scope, arguments.get(0));
    if !update(scope, arguments.this(), |record| {
        record.on_state_change = value
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_gathering_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_handler(scope, record.on_gathering_state_change, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_on_gathering_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(scope, arguments.get(0));
    if !update(scope, arguments.this(), |record| {
        record.on_gathering_state_change = value
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_selected_candidate_pair_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_handler(scope, record.on_selected_candidate_pair_change, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_on_selected_candidate_pair_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(scope, arguments.get(0));
    if !update(scope, arguments.this(), |record| {
        record.on_selected_candidate_pair_change = value
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn object_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    values: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, value).into());
    }
    array
}

fn get_local_candidates(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(object_array(scope, &record.local_candidates).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_remote_candidates(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(object_array(scope, &record.remote_candidates).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn return_optional_object(
    scope: &v8::PinScope<'_, '_>,
    value: Option<v8::Global<v8::Object>>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_local_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_object(scope, record.local_parameters, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_remote_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_object(scope, record.remote_parameters, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_selected_candidate_pair(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_object(scope, record.selected_pair, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
