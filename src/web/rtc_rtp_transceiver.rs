use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcRtpTransceiverStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TransceiverRecord>,
}

#[derive(Clone)]
struct TransceiverRecord {
    mid: Option<String>,
    sender: v8::Global<v8::Object>,
    receiver: v8::Global<v8::Object>,
    stopped: bool,
    direction: String,
    current_direction: Option<String>,
    header_extensions_to_negotiate: Vec<v8::Global<v8::Object>>,
    negotiated_header_extensions: Vec<v8::Global<v8::Object>>,
    codec_preferences: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcRtpTransceiverStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCRtpTransceiver", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcRtpTransceiverStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCRtpTransceiver",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mid", get_mid)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sender", get_sender)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "receiver", get_receiver)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "stopped", get_stopped)?;
    crate::webidl::define_accessor(scope, prototype, "direction", get_direction, set_direction)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "currentDirection",
        get_current_direction,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getHeaderExtensionsToNegotiate",
        0,
        get_header_extensions_to_negotiate,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getNegotiatedHeaderExtensions",
        0,
        get_negotiated_header_extensions,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setCodecPreferences",
        1,
        set_codec_preferences,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setHeaderExtensionsToNegotiate",
        1,
        set_header_extensions_to_negotiate,
    )?;
    crate::webidl::define_method(scope, prototype, "stop", 0, stop)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcRtpTransceiverStore>()
        .ok_or_else(|| "RTCRtpTransceiver state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    sender: v8::Local<'_, v8::Object>,
    receiver: v8::Local<'_, v8::Object>,
    direction: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create RTCRtpTransceiver".to_owned());
    }
    let record = TransceiverRecord {
        mid: None,
        sender: v8::Global::new(scope, sender),
        receiver: v8::Global::new(scope, receiver),
        stopped: false,
        direction,
        current_direction: None,
        header_extensions_to_negotiate: default_header_extensions(scope),
        negotiated_header_extensions: default_header_extensions(scope),
        codec_preferences: Vec::new(),
    };
    scope
        .get_slot_mut::<RtcRtpTransceiverStore>()
        .ok_or_else(|| "RTCRtpTransceiver state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn default_header_extensions(scope: &v8::PinScope<'_, '_>) -> Vec<v8::Global<v8::Object>> {
    let values = [
        "urn:ietf:params:rtp-hdrext:sdes:mid",
        "urn:ietf:params:rtp-hdrext:ssrc-audio-level",
        "http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time",
        "urn:3gpp:video-orientation",
        "http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01",
    ];
    let mut output = Vec::new();
    for uri in values {
        let object = v8::Object::new(scope);
        if let Some(key) = v8::String::new(scope, "uri") {
            if let Some(uri) = v8::String::new(scope, uri) {
                let _ = object.create_data_property(scope, key.into(), uri.into());
            }
        }
        output.push(v8::Global::new(scope, object));
    }
    output
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCRtpTransceiver': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TransceiverRecord> {
    scope
        .get_slot::<RtcRtpTransceiverStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut TransceiverRecord),
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<RtcRtpTransceiverStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
        true
    } else {
        false
    }
}

fn get_mid(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.mid {
            Some(value) => {
                if let Some(value) = v8::String::new(scope, &value) {
                    result.set(value.into());
                }
            }
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_sender(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Local::new(scope, &record.sender).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_receiver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Local::new(scope, &record.receiver).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_stopped(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Boolean::new(scope, record.stopped).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_direction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => {
            if let Some(value) = v8::String::new(scope, &record.direction) {
                result.set(value.into());
            }
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_direction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !matches!(
        value.as_str(),
        "sendrecv" | "sendonly" | "recvonly" | "inactive"
    ) {
        crate::webidl::throw_type_error(scope, "Invalid RTCRtpTransceiverDirection");
        return;
    }
    if !update(scope, arguments.this(), |record| record.direction = value) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_current_direction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.current_direction {
            Some(value) => {
                if let Some(value) = v8::String::new(scope, &value) {
                    result.set(value.into());
                }
            }
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
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

fn get_header_extensions_to_negotiate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => {
            result.set(object_array(scope, &record.header_extensions_to_negotiate).into())
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_negotiated_header_extensions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => {
            result.set(object_array(scope, &record.negotiated_header_extensions).into())
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn array_objects(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    let array = v8::Local::<v8::Array>::try_from(value).ok()?;
    let mut output = Vec::new();
    for index in 0..array.length() {
        if let Some(object) = array
            .get_index(scope, index)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        {
            output.push(v8::Global::new(scope, object));
        }
    }
    Some(output)
}

fn set_codec_preferences(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(values) = array_objects(scope, arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Codec preferences must be a sequence");
        return;
    };
    if !update(scope, arguments.this(), |record| {
        record.codec_preferences = values
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_header_extensions_to_negotiate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(values) = array_objects(scope, arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Header extensions must be a sequence");
        return;
    };
    if !update(scope, arguments.this(), |record| {
        record.header_extensions_to_negotiate = values
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !update(scope, arguments.this(), |record| {
        record.stopped = true;
        record.current_direction = None;
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
