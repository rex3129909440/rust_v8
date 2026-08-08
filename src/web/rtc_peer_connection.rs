use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcPeerConnectionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PeerConnectionRecord>,
}

#[derive(Clone)]
struct SessionDescription {
    kind: String,
    sdp: String,
}

#[derive(Clone)]
struct PeerConfiguration {
    always_negotiate_data_channels: bool,
    bundle_policy: String,
    encoded_insertable_streams: bool,
    ice_candidate_pool_size: u32,
    ice_transport_policy: String,
    rtcp_mux_policy: String,
}

impl Default for PeerConfiguration {
    fn default() -> Self {
        Self {
            always_negotiate_data_channels: false,
            bundle_policy: "balanced".to_owned(),
            encoded_insertable_streams: false,
            ice_candidate_pool_size: 0,
            ice_transport_policy: "all".to_owned(),
            rtcp_mux_policy: "require".to_owned(),
        }
    }
}

#[derive(Clone, Copy)]
enum HandlerSlot {
    NegotiationNeeded,
    IceCandidate,
    SignalingStateChange,
    IceConnectionStateChange,
    ConnectionStateChange,
    IceGatheringStateChange,
    IceCandidateError,
    Track,
    DataChannel,
    AddStream,
    RemoveStream,
}

#[derive(Clone)]
struct PeerConnectionRecord {
    configuration: PeerConfiguration,
    local_description: Option<SessionDescription>,
    remote_description: Option<SessionDescription>,
    signaling_state: String,
    ice_gathering_state: String,
    ice_connection_state: String,
    connection_state: String,
    can_trickle_ice_candidates: Option<bool>,
    negotiation_needed: Option<v8::Global<v8::Value>>,
    ice_candidate: Option<v8::Global<v8::Value>>,
    signaling_state_change: Option<v8::Global<v8::Value>>,
    ice_connection_state_change: Option<v8::Global<v8::Value>>,
    connection_state_change: Option<v8::Global<v8::Value>>,
    ice_gathering_state_change: Option<v8::Global<v8::Value>>,
    ice_candidate_error: Option<v8::Global<v8::Value>>,
    track: Option<v8::Global<v8::Value>>,
    data_channel: Option<v8::Global<v8::Value>>,
    add_stream: Option<v8::Global<v8::Value>>,
    remove_stream: Option<v8::Global<v8::Value>>,
    local_streams: Vec<v8::Global<v8::Object>>,
    remote_streams: Vec<v8::Global<v8::Object>>,
    senders: Vec<v8::Global<v8::Object>>,
    receivers: Vec<v8::Global<v8::Object>>,
    transceivers: Vec<v8::Global<v8::Object>>,
}

impl Default for PeerConnectionRecord {
    fn default() -> Self {
        Self {
            configuration: PeerConfiguration::default(),
            local_description: None,
            remote_description: None,
            signaling_state: "stable".to_owned(),
            ice_gathering_state: "new".to_owned(),
            ice_connection_state: "new".to_owned(),
            connection_state: "new".to_owned(),
            can_trickle_ice_candidates: None,
            negotiation_needed: None,
            ice_candidate: None,
            signaling_state_change: None,
            ice_connection_state_change: None,
            connection_state_change: None,
            ice_gathering_state_change: None,
            ice_candidate_error: None,
            track: None,
            data_channel: None,
            add_stream: None,
            remove_stream: None,
            local_streams: Vec::new(),
            remote_streams: Vec::new(),
            senders: Vec::new(),
            receivers: Vec::new(),
            transceivers: Vec::new(),
        }
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcPeerConnectionStore::default());
}

#[allow(dead_code)]
pub(crate) fn install_standard_name(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCPeerConnection", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcPeerConnectionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }

    let constructor = crate::webidl::create_function(
        scope,
        "RTCPeerConnection",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;

    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "localDescription",
        get_local_description,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "currentLocalDescription",
        get_current_local_description,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "pendingLocalDescription",
        get_pending_local_description,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "remoteDescription",
        get_remote_description,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "currentRemoteDescription",
        get_current_remote_description,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "pendingRemoteDescription",
        get_pending_remote_description,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "signalingState",
        get_signaling_state,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "iceGatheringState",
        get_ice_gathering_state,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "iceConnectionState",
        get_ice_connection_state,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "connectionState",
        get_connection_state,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "canTrickleIceCandidates",
        get_can_trickle_ice_candidates,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onnegotiationneeded",
        get_on_negotiation_needed,
        set_on_negotiation_needed,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onicecandidate",
        get_on_ice_candidate,
        set_on_ice_candidate,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onsignalingstatechange",
        get_on_signaling_state_change,
        set_on_signaling_state_change,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oniceconnectionstatechange",
        get_on_ice_connection_state_change,
        set_on_ice_connection_state_change,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onconnectionstatechange",
        get_on_connection_state_change,
        set_on_connection_state_change,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onicegatheringstatechange",
        get_on_ice_gathering_state_change,
        set_on_ice_gathering_state_change,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onicecandidateerror",
        get_on_ice_candidate_error,
        set_on_ice_candidate_error,
    )?;
    crate::webidl::define_accessor(scope, prototype, "ontrack", get_on_track, set_on_track)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sctp", get_sctp)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ondatachannel",
        get_on_data_channel,
        set_on_data_channel,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onaddstream",
        get_on_add_stream,
        set_on_add_stream,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onremovestream",
        get_on_remove_stream,
        set_on_remove_stream,
    )?;

    crate::webidl::define_method(scope, prototype, "addIceCandidate", 0, add_ice_candidate)?;
    crate::webidl::define_method(scope, prototype, "addStream", 1, add_stream)?;
    crate::webidl::define_method(scope, prototype, "addTrack", 1, add_track)?;
    crate::webidl::define_method(scope, prototype, "addTransceiver", 1, add_transceiver)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "createAnswer", 0, create_answer)?;
    crate::webidl::define_method(scope, prototype, "createDTMFSender", 1, create_dtmf_sender)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createDataChannel",
        1,
        create_data_channel,
    )?;
    crate::webidl::define_method(scope, prototype, "createOffer", 0, create_offer)?;
    crate::webidl::define_method(scope, prototype, "getConfiguration", 0, get_configuration)?;
    crate::webidl::define_method(scope, prototype, "getLocalStreams", 0, get_local_streams)?;
    crate::webidl::define_method(scope, prototype, "getReceivers", 0, get_receivers)?;
    crate::webidl::define_method(scope, prototype, "getRemoteStreams", 0, get_remote_streams)?;
    crate::webidl::define_method(scope, prototype, "getSenders", 0, get_senders)?;
    crate::webidl::define_method(scope, prototype, "getStats", 0, get_stats)?;
    crate::webidl::define_method(scope, prototype, "getTransceivers", 0, get_transceivers)?;
    crate::webidl::define_method(scope, prototype, "removeStream", 1, remove_stream)?;
    crate::webidl::define_method(scope, prototype, "removeTrack", 1, remove_track)?;
    crate::webidl::define_method(scope, prototype, "restartIce", 0, restart_ice)?;
    crate::webidl::define_method(scope, prototype, "setConfiguration", 0, set_configuration)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setLocalDescription",
        0,
        set_local_description,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setRemoteDescription",
        1,
        set_remote_description,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "generateCertificate",
        1,
        generate_certificate,
    )?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;

    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcPeerConnectionStore>()
        .ok_or_else(|| "RTCPeerConnection state was not prepared".to_owned())?
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
            "Failed to construct 'RTCPeerConnection': Please use the 'new' operator",
        );
        return;
    }
    let mut record = PeerConnectionRecord::default();
    if arguments.get(0).is_object() {
        read_configuration(scope, arguments.get(0), &mut record.configuration);
    }
    let object = arguments.this();
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<RtcPeerConnectionStore>()
        .expect("RTCPeerConnection state")
        .records
        .insert(object.get_identity_hash().get(), record);
    result.set(object.into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PeerConnectionRecord> {
    scope
        .get_slot::<RtcPeerConnectionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut PeerConnectionRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<RtcPeerConnectionStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    operation(record);
    true
}

fn string_value<'s>(scope: &v8::PinScope<'s, '_>, value: &str) -> v8::Local<'s, v8::Value> {
    v8::String::new(scope, value)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into())
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(name) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, name.into(), value);
    }
}

fn get_named<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let name = v8::String::new(scope, name)?;
    object.get(scope, name.into())
}

fn read_string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let value = get_named(scope, object, name)?;
    if value.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn read_configuration(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    target: &mut PeerConfiguration,
) {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return;
    };
    if let Some(value) = get_named(scope, object, "alwaysNegotiateDataChannels") {
        if !value.is_undefined() {
            target.always_negotiate_data_channels = value.boolean_value(scope);
        }
    }
    if let Some(value) = read_string_property(scope, object, "bundlePolicy") {
        target.bundle_policy = value;
    }
    if let Some(value) = get_named(scope, object, "encodedInsertableStreams") {
        if !value.is_undefined() {
            target.encoded_insertable_streams = value.boolean_value(scope);
        }
    }
    if let Some(value) = get_named(scope, object, "iceCandidatePoolSize") {
        if !value.is_undefined() {
            target.ice_candidate_pool_size = value
                .uint32_value(scope)
                .unwrap_or(target.ice_candidate_pool_size);
        }
    }
    if let Some(value) = read_string_property(scope, object, "iceTransportPolicy") {
        target.ice_transport_policy = value;
    }
    if let Some(value) = read_string_property(scope, object, "rtcpMuxPolicy") {
        target.rtcp_mux_policy = value;
    }
}

fn description_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    description: Option<&SessionDescription>,
) -> v8::Local<'s, v8::Value> {
    let Some(description) = description else {
        return v8::null(scope).into();
    };
    let object = v8::Object::new(scope);
    define_data(
        scope,
        object,
        "type",
        string_value(scope, &description.kind),
    );
    define_data(scope, object, "sdp", string_value(scope, &description.sdp));
    object.into()
}

fn read_description(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    default_kind: &str,
) -> SessionDescription {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return SessionDescription {
            kind: default_kind.to_owned(),
            sdp: String::new(),
        };
    };
    SessionDescription {
        kind: read_string_property(scope, object, "type")
            .unwrap_or_else(|| default_kind.to_owned()),
        sdp: read_string_property(scope, object, "sdp").unwrap_or_default(),
    }
}

fn resolved_promise(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let promise = resolver.get_promise(scope);
    let _ = resolver.resolve(scope, value);
    result.set(promise.into());
}

fn return_record_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PeerConnectionRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(string_value(scope, select(&record)));
}

fn return_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PeerConnectionRecord) -> Option<&SessionDescription>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(description_object(scope, select(&record)));
}

fn get_local_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_description(scope, arguments, result, |record| {
        record.local_description.as_ref()
    });
}

fn get_current_local_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_description(scope, arguments, result, |record| {
        record.local_description.as_ref()
    });
}

fn get_pending_local_description(
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

fn get_remote_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_description(scope, arguments, result, |record| {
        record.remote_description.as_ref()
    });
}

fn get_current_remote_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_description(scope, arguments, result, |record| {
        record.remote_description.as_ref()
    });
}

fn get_pending_remote_description(
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

fn get_signaling_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_record_string(scope, arguments, result, |record| &record.signaling_state);
}

fn get_ice_gathering_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_record_string(scope, arguments, result, |record| {
        &record.ice_gathering_state
    });
}

fn get_ice_connection_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_record_string(scope, arguments, result, |record| {
        &record.ice_connection_state
    });
}

fn get_connection_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_record_string(scope, arguments, result, |record| &record.connection_state);
}

fn get_can_trickle_ice_candidates(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.can_trickle_ice_candidates {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_sctp(
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

fn handler(record: &PeerConnectionRecord, slot: HandlerSlot) -> Option<v8::Global<v8::Value>> {
    match slot {
        HandlerSlot::NegotiationNeeded => record.negotiation_needed.clone(),
        HandlerSlot::IceCandidate => record.ice_candidate.clone(),
        HandlerSlot::SignalingStateChange => record.signaling_state_change.clone(),
        HandlerSlot::IceConnectionStateChange => record.ice_connection_state_change.clone(),
        HandlerSlot::ConnectionStateChange => record.connection_state_change.clone(),
        HandlerSlot::IceGatheringStateChange => record.ice_gathering_state_change.clone(),
        HandlerSlot::IceCandidateError => record.ice_candidate_error.clone(),
        HandlerSlot::Track => record.track.clone(),
        HandlerSlot::DataChannel => record.data_channel.clone(),
        HandlerSlot::AddStream => record.add_stream.clone(),
        HandlerSlot::RemoveStream => record.remove_stream.clone(),
    }
}

fn set_handler_value(
    record: &mut PeerConnectionRecord,
    slot: HandlerSlot,
    value: Option<v8::Global<v8::Value>>,
) {
    match slot {
        HandlerSlot::NegotiationNeeded => record.negotiation_needed = value,
        HandlerSlot::IceCandidate => record.ice_candidate = value,
        HandlerSlot::SignalingStateChange => record.signaling_state_change = value,
        HandlerSlot::IceConnectionStateChange => record.ice_connection_state_change = value,
        HandlerSlot::ConnectionStateChange => record.connection_state_change = value,
        HandlerSlot::IceGatheringStateChange => record.ice_gathering_state_change = value,
        HandlerSlot::IceCandidateError => record.ice_candidate_error = value,
        HandlerSlot::Track => record.track = value,
        HandlerSlot::DataChannel => record.data_channel = value,
        HandlerSlot::AddStream => record.add_stream = value,
        HandlerSlot::RemoveStream => record.remove_stream = value,
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
    if let Some(value) = handler(&record, slot) {
        result.set(v8::Local::new(scope, &value));
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
    let stored = if value.is_function() {
        Some(v8::Global::new(scope, value))
    } else {
        None
    };
    update(scope, arguments.this(), |record| {
        set_handler_value(record, slot, stored)
    });
}

fn get_on_negotiation_needed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::NegotiationNeeded);
}
fn set_on_negotiation_needed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::NegotiationNeeded);
}
fn get_on_ice_candidate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::IceCandidate);
}
fn set_on_ice_candidate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::IceCandidate);
}
fn get_on_signaling_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::SignalingStateChange);
}
fn set_on_signaling_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::SignalingStateChange);
}
fn get_on_ice_connection_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::IceConnectionStateChange);
}
fn set_on_ice_connection_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::IceConnectionStateChange);
}
fn get_on_connection_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::ConnectionStateChange);
}
fn set_on_connection_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::ConnectionStateChange);
}
fn get_on_ice_gathering_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::IceGatheringStateChange);
}
fn set_on_ice_gathering_state_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::IceGatheringStateChange);
}
fn get_on_ice_candidate_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::IceCandidateError);
}
fn set_on_ice_candidate_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::IceCandidateError);
}
fn get_on_track(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::Track);
}
fn set_on_track(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::Track);
}
fn get_on_data_channel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::DataChannel);
}
fn set_on_data_channel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::DataChannel);
}
fn get_on_add_stream(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::AddStream);
}
fn set_on_add_stream(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::AddStream);
}
fn get_on_remove_stream(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, HandlerSlot::RemoveStream);
}
fn set_on_remove_stream(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, HandlerSlot::RemoveStream);
}

fn add_ice_candidate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let undefined = v8::undefined(scope);
    resolved_promise(scope, undefined.into(), result);
}

fn add_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "addStream requires a MediaStream");
        return;
    };
    let stream = v8::Global::new(scope, stream);
    update(scope, arguments.this(), |record| {
        record.local_streams.push(stream)
    });
}

fn add_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(track) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "addTrack requires a MediaStreamTrack");
        return;
    };
    let mut streams = Vec::new();
    for index in 1..arguments.length() {
        if let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(index)) {
            streams.push(stream);
        }
    }
    let sender = match super::rtc_rtp_sender::create(scope, Some(track), streams) {
        Ok(sender) => sender,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let sender_global = v8::Global::new(scope, sender);
    if update(scope, arguments.this(), |record| {
        record.senders.push(sender_global)
    }) {
        result.set(sender.into());
    }
}

fn add_transceiver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let source = arguments.get(0);
    if source.is_undefined() {
        crate::webidl::throw_type_error(scope, "addTransceiver requires a track or kind");
        return;
    }
    let (sender_track, receiver_track, media_kind) =
        if let Ok(track) = v8::Local::<v8::Object>::try_from(source) {
            let kind = read_string_property(scope, track, "kind").unwrap_or_default();
            (Some(track), track, kind)
        } else {
            let kind = crate::webidl::value_to_string(scope, source);
            if kind != "audio" && kind != "video" {
                crate::webidl::throw_type_error(scope, "The media kind must be audio or video");
                return;
            }
            let track = v8::Object::new(scope);
            define_data(scope, track, "kind", string_value(scope, &kind));
            define_data(
                scope,
                track,
                "enabled",
                v8::Boolean::new(scope, true).into(),
            );
            define_data(scope, track, "muted", v8::Boolean::new(scope, false).into());
            (None, track, kind)
        };
    let sender = match super::rtc_rtp_sender::create_with_kind(
        scope,
        sender_track,
        Vec::new(),
        media_kind,
    ) {
        Ok(sender) => sender,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let receiver = match super::rtc_rtp_receiver::create(scope, receiver_track) {
        Ok(receiver) => receiver,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let direction = v8::Local::<v8::Object>::try_from(arguments.get(1))
        .ok()
        .and_then(|init| read_string_property(scope, init, "direction"))
        .unwrap_or_else(|| "sendrecv".to_owned());
    let transceiver = match super::rtc_rtp_transceiver::create(scope, sender, receiver, direction) {
        Ok(transceiver) => transceiver,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let sender_global = v8::Global::new(scope, sender);
    let receiver_global = v8::Global::new(scope, receiver);
    let transceiver_global = v8::Global::new(scope, transceiver);
    if update(scope, arguments.this(), |record| {
        record.senders.push(sender_global);
        record.receivers.push(receiver_global);
        record.transceivers.push(transceiver_global);
    }) {
        result.set(transceiver.into());
    }
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.signaling_state = "closed".to_owned();
        record.ice_connection_state = "closed".to_owned();
        record.connection_state = "closed".to_owned();
    });
}

fn make_description<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: &str,
    sdp: String,
) -> v8::Local<'s, v8::Object> {
    super::rtc_session_description::create(scope, Some(kind.to_owned()), sdp)
        .unwrap_or_else(|_| v8::Object::new(scope))
}

fn create_answer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let sdp = crate::fingerprint::edge(scope).media.rtc_answer_sdp.clone();
    let description = make_description(scope, "answer", sdp);
    resolved_promise(scope, description.into(), result);
}

fn create_dtmf_sender(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::rtc_dtmf_sender::create(scope, false) {
        Ok(sender) => result.set(sender.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_data_channel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let label = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(channel) = super::rtc_data_channel::create(scope, label, arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "Failed to create RTCDataChannel");
        return;
    };
    super::rtc_data_channel_event::register_channel(scope, channel);
    result.set(channel.into());
}

fn create_offer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let sdp = crate::fingerprint::edge(scope).media.rtc_offer_sdp.clone();
    let description = make_description(scope, "offer", sdp);
    resolved_promise(scope, description.into(), result);
}

fn get_configuration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let configuration = v8::Object::new(scope);
    define_data(
        scope,
        configuration,
        "alwaysNegotiateDataChannels",
        v8::Boolean::new(scope, record.configuration.always_negotiate_data_channels).into(),
    );
    define_data(
        scope,
        configuration,
        "bundlePolicy",
        string_value(scope, &record.configuration.bundle_policy),
    );
    define_data(
        scope,
        configuration,
        "certificates",
        v8::Array::new(scope, 0).into(),
    );
    define_data(
        scope,
        configuration,
        "encodedInsertableStreams",
        v8::Boolean::new(scope, record.configuration.encoded_insertable_streams).into(),
    );
    define_data(
        scope,
        configuration,
        "iceCandidatePoolSize",
        v8::Integer::new_from_unsigned(scope, record.configuration.ice_candidate_pool_size).into(),
    );
    define_data(
        scope,
        configuration,
        "iceServers",
        v8::Array::new(scope, 0).into(),
    );
    define_data(
        scope,
        configuration,
        "iceTransportPolicy",
        string_value(scope, &record.configuration.ice_transport_policy),
    );
    define_data(
        scope,
        configuration,
        "rtcpMuxPolicy",
        string_value(scope, &record.configuration.rtcp_mux_policy),
    );
    result.set(configuration.into());
}

fn object_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    values: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let value = v8::Local::new(scope, value);
        let _ = array.set_index(scope, index as u32, value.into());
    }
    array
}

fn return_object_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PeerConnectionRecord) -> &[v8::Global<v8::Object>],
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(object_array(scope, select(&record)).into());
}

fn get_local_streams(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object_list(s, a, r, |record| &record.local_streams);
}
fn get_receivers(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object_list(s, a, r, |record| &record.receivers);
}
fn get_remote_streams(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object_list(s, a, r, |record| &record.remote_streams);
}
fn get_senders(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object_list(s, a, r, |record| &record.senders);
}
fn get_transceivers(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object_list(s, a, r, |record| &record.transceivers);
}

fn get_stats(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::rtc_stats_report::create(scope, Vec::new()) {
        Ok(report) => resolved_promise(scope, report.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn remove_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        return;
    };
    let identity = stream.get_identity_hash().get();
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let position = snapshot.local_streams.iter().position(|candidate| {
        let candidate = v8::Local::new(scope, candidate);
        candidate.get_identity_hash().get() == identity
    });
    update(scope, arguments.this(), |record| {
        if let Some(position) = position {
            record.local_streams.remove(position);
        }
    });
}

fn remove_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(sender) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        return;
    };
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let known = snapshot.senders.iter().any(|candidate| {
        let candidate = v8::Local::new(scope, candidate);
        candidate.strict_equals(sender.into())
    });
    if known {
        let _ = super::rtc_rtp_sender::set_track(scope, sender, None);
    }
}

fn restart_ice(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        if record.signaling_state != "closed" {
            record.ice_gathering_state = "new".to_owned();
            record.ice_connection_state = "checking".to_owned();
            record.connection_state = "connecting".to_owned();
        }
    });
}

fn set_configuration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let mut configuration = match record(scope, arguments.this()) {
        Some(record) => record.configuration,
        None => {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        }
    };
    read_configuration(scope, value, &mut configuration);
    update(scope, arguments.this(), |record| {
        record.configuration = configuration
    });
}

fn set_local_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let description = read_description(scope, arguments.get(0), "offer");
    if !update(scope, arguments.this(), |record| {
        record.local_description = Some(description);
    }) {
        return;
    }
    let undefined = v8::undefined(scope);
    resolved_promise(scope, undefined.into(), result);
}

fn set_remote_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let description = read_description(scope, arguments.get(0), "answer");
    if !update(scope, arguments.this(), |record| {
        record.remote_description = Some(description);
        record.can_trickle_ice_candidates = Some(false);
    }) {
        return;
    }
    let undefined = v8::undefined(scope);
    resolved_promise(scope, undefined.into(), result);
}

fn generate_certificate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match super::rtc_certificate::create(scope, arguments.get(0)) {
        Ok(certificate) => resolved_promise(scope, certificate.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
