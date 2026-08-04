use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcIceCandidateStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CandidateRecord>,
}

#[derive(Clone, Default)]
struct CandidateRecord {
    candidate: String,
    sdp_mid: Option<String>,
    sdp_m_line_index: Option<u32>,
    foundation: Option<String>,
    component: Option<String>,
    priority: Option<u32>,
    address: Option<String>,
    protocol: Option<String>,
    port: Option<u32>,
    candidate_type: Option<String>,
    tcp_type: Option<String>,
    related_address: Option<String>,
    related_port: Option<u32>,
    username_fragment: Option<String>,
    relay_protocol: Option<String>,
    url: Option<String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcIceCandidateStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCIceCandidate", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcIceCandidateStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCIceCandidate",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "candidate", get_candidate)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sdpMid", get_sdp_mid)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "sdpMLineIndex",
        get_sdp_m_line_index,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "foundation", get_foundation)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "component", get_component)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "priority", get_priority)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "address", get_address)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "protocol", get_protocol)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "port", get_port)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "tcpType", get_tcp_type)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "relatedAddress",
        get_related_address,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "relatedPort", get_related_port)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "usernameFragment",
        get_username_fragment,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "relayProtocol", get_relay_protocol)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcIceCandidateStore>()
        .ok_or_else(|| "RTCIceCandidate state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'RTCIceCandidate': use new");
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let sdp_mid = init.and_then(|object| optional_string(scope, object, "sdpMid"));
    let sdp_m_line_index = init.and_then(|object| optional_u32(scope, object, "sdpMLineIndex"));
    if sdp_mid.is_none() && sdp_m_line_index.is_none() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCIceCandidate': sdpMid and sdpMLineIndex are both null",
        );
        return;
    }
    let candidate = init
        .and_then(|object| optional_string(scope, object, "candidate"))
        .unwrap_or_default();
    let username_fragment =
        init.and_then(|object| optional_string(scope, object, "usernameFragment"));
    let mut record = parse_candidate(&candidate);
    record.candidate = candidate;
    record.sdp_mid = sdp_mid;
    record.sdp_m_line_index = sdp_m_line_index;
    if username_fragment.is_some() {
        record.username_fragment = username_fragment;
    }
    attach(scope, arguments.this(), record);
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    candidate: String,
    sdp_mid: Option<String>,
    sdp_m_line_index: Option<u32>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create RTCIceCandidate".to_owned());
    }
    let mut record = parse_candidate(&candidate);
    record.candidate = candidate;
    record.sdp_mid = sdp_mid;
    record.sdp_m_line_index = sdp_m_line_index;
    attach(scope, object, record);
    Ok(object)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    record: CandidateRecord,
) {
    scope
        .get_slot_mut::<RtcIceCandidateStore>()
        .expect("RTCIceCandidate state")
        .records
        .insert(object.get_identity_hash().get(), record);
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CandidateRecord> {
    scope
        .get_slot::<RtcIceCandidateStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn parse_candidate(value: &str) -> CandidateRecord {
    let mut record = CandidateRecord::default();
    let parts: Vec<&str> = value.split_ascii_whitespace().collect();
    if parts.len() < 8 || !parts[0].starts_with("candidate:") {
        return record;
    }
    record.foundation = Some(parts[0].trim_start_matches("candidate:").to_owned());
    record.component = match parts[1] {
        "1" => Some("rtp".to_owned()),
        "2" => Some("rtcp".to_owned()),
        value => Some(value.to_owned()),
    };
    record.protocol = Some(parts[2].to_ascii_lowercase());
    record.priority = parts[3].parse().ok();
    record.address = Some(parts[4].to_owned());
    record.port = parts[5].parse().ok();
    if parts[6].eq_ignore_ascii_case("typ") {
        record.candidate_type = Some(parts[7].to_ascii_lowercase());
    }
    let mut index = 8;
    while index + 1 < parts.len() {
        let key = parts[index].to_ascii_lowercase();
        let value = parts[index + 1];
        match key.as_str() {
            "tcptype" => record.tcp_type = Some(value.to_ascii_lowercase()),
            "raddr" => record.related_address = Some(value.to_owned()),
            "rport" => record.related_port = value.parse().ok(),
            "ufrag" => record.username_fragment = Some(value.to_owned()),
            "relay-protocol" => record.relay_protocol = Some(value.to_ascii_lowercase()),
            "url" => record.url = Some(value.to_owned()),
            _ => {}
        }
        index += 2;
    }
    record
}

fn optional_string(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null_or_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn optional_u32(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u32> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null_or_undefined() {
        None
    } else {
        value.uint32_value(scope)
    }
}

fn return_string(scope: &mut v8::PinScope<'_, '_>, value: &str, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn return_optional_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<String>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        return_string(scope, &value, result);
    } else {
        result.set(v8::null(scope).into());
    }
}

fn return_optional_u32(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<u32>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Integer::new_from_unsigned(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_candidate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.candidate, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_sdp_mid(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.sdp_mid, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_sdp_m_line_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_u32(scope, record.sdp_m_line_index, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_foundation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.foundation, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_component(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.component, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_priority(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_u32(scope, record.priority, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_address(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.address, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_protocol(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.protocol, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_u32(scope, record.port, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.candidate_type, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_tcp_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.tcp_type, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_related_address(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.related_address, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_related_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_u32(scope, record.related_port, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_username_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.username_fragment, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_relay_protocol(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.relay_protocol, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_string(scope, record.url, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    define_string(scope, object, "candidate", &record.candidate);
    if let Some(value) = record.sdp_mid {
        define_string(scope, object, "sdpMid", &value);
    }
    if let Some(value) = record.sdp_m_line_index {
        define_number(scope, object, "sdpMLineIndex", value);
    }
    if let Some(value) = record.username_fragment {
        define_string(scope, object, "usernameFragment", &value);
    }
    result.set(object.into());
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let Some(value) = v8::String::new(scope, value) else {
        return;
    };
    let _ = object.create_data_property(scope, key.into(), value.into());
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: u32,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(
            scope,
            key.into(),
            v8::Integer::new_from_unsigned(scope, value).into(),
        );
    }
}
