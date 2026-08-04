use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcRtpSenderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SenderRecord>,
}

#[derive(Clone)]
struct SenderRecord {
    track: Option<v8::Global<v8::Object>>,
    transport: Option<v8::Global<v8::Object>>,
    rtcp_transport: Option<v8::Global<v8::Object>>,
    dtmf: Option<v8::Global<v8::Object>>,
    transform: Option<v8::Global<v8::Value>>,
    streams: Vec<v8::Global<v8::Object>>,
    encodings: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcRtpSenderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCRtpSender", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcRtpSenderStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCRtpSender",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "track", get_track)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transport", get_transport)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rtcpTransport", get_rtcp_transport)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "dtmf", get_dtmf)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createEncodedStreams",
        0,
        create_encoded_streams,
    )?;
    crate::webidl::define_method(scope, prototype, "getParameters", 0, get_parameters)?;
    crate::webidl::define_method(scope, prototype, "getStats", 0, get_stats)?;
    crate::webidl::define_method(scope, prototype, "replaceTrack", 1, replace_track)?;
    crate::webidl::define_method(scope, prototype, "setParameters", 1, set_parameters)?;
    crate::webidl::define_method(scope, prototype, "setStreams", 0, set_streams)?;
    crate::webidl::define_accessor(scope, prototype, "transform", get_transform, set_transform)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "getCapabilities",
        1,
        get_capabilities,
    )?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcRtpSenderStore>()
        .ok_or_else(|| "RTCRtpSender state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    track: Option<v8::Local<'_, v8::Object>>,
    streams: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let kind = track
        .and_then(|track| property_string(scope, track, "kind"))
        .unwrap_or_default();
    create_with_kind(scope, track, streams, kind)
}

pub(crate) fn create_with_kind<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    track: Option<v8::Local<'_, v8::Object>>,
    streams: Vec<v8::Local<'_, v8::Object>>,
    kind: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let sender = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, sender, prototype.into()) != Some(true) {
        return Err("cannot create RTCRtpSender".to_owned());
    }
    let dtmf = if kind == "audio" {
        let sender = super::rtc_dtmf_sender::create(scope, false)?;
        Some(v8::Global::new(scope, sender))
    } else {
        None
    };
    let record = SenderRecord {
        track: track.map(|track| v8::Global::new(scope, track)),
        transport: None,
        rtcp_transport: None,
        dtmf,
        transform: None,
        streams: streams
            .into_iter()
            .map(|stream| v8::Global::new(scope, stream))
            .collect(),
        encodings: Vec::new(),
    };
    scope
        .get_slot_mut::<RtcRtpSenderStore>()
        .ok_or_else(|| "RTCRtpSender state was not prepared".to_owned())?
        .records
        .insert(sender.get_identity_hash().get(), record);
    Ok(sender)
}

pub(crate) fn set_track(
    scope: &mut v8::PinScope<'_, '_>,
    sender: v8::Local<'_, v8::Object>,
    track: Option<v8::Local<'_, v8::Object>>,
) -> bool {
    let track = track.map(|track| v8::Global::new(scope, track));
    if let Some(record) = scope
        .get_slot_mut::<RtcRtpSenderStore>()
        .and_then(|store| store.records.get_mut(&sender.get_identity_hash().get()))
    {
        record.track = track;
        true
    } else {
        false
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCRtpSender': Illegal constructor",
    );
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<SenderRecord> {
    scope
        .get_slot::<RtcRtpSenderStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
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

fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_object(scope, record.track, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_transport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_object(scope, record.transport, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_rtcp_transport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_object(scope, record.rtcp_transport, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_dtmf(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_object(scope, record.dtmf, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn create_encoded_streams(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let object = v8::Object::new(scope);
    if let Ok(readable) = super::readable_stream::create_empty(scope) {
        define_value(scope, object, "readable", readable.into());
    }
    define_value(scope, object, "writable", v8::Object::new(scope).into());
    result.set(object.into());
}

fn get_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    define_string(scope, object, "transactionId", "");
    define_value(scope, object, "codecs", v8::Array::new(scope, 0).into());
    define_value(
        scope,
        object,
        "headerExtensions",
        v8::Array::new(scope, 0).into(),
    );
    define_value(scope, object, "rtcp", v8::Object::new(scope).into());
    let encodings = v8::Array::new(scope, record.encodings.len() as i32);
    for (index, encoding) in record.encodings.iter().enumerate() {
        let _ = encodings.set_index(scope, index as u32, v8::Local::new(scope, encoding).into());
    }
    define_value(scope, object, "encodings", encodings.into());
    result.set(object.into());
}

fn get_stats(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(report) = super::rtc_stats_report::create(scope, Vec::new()) {
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, report.into()) {
            result.set(promise.into());
        }
    }
}

fn replace_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let track = if value.is_null_or_undefined() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(value).ok()
    };
    if !set_track(scope, arguments.this(), track) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn set_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn set_streams(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mut streams = Vec::new();
    for index in 0..arguments.length() {
        if let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(index)) {
            streams.push(v8::Global::new(scope, stream));
        }
    }
    if let Some(record) = scope.get_slot_mut::<RtcRtpSenderStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.streams = streams;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.transform {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let transform = if value.is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    };
    if let Some(record) = scope.get_slot_mut::<RtcRtpSenderStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.transform = transform;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_capabilities(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let kind = crate::webidl::value_to_string(scope, arguments.get(0));
    if kind != "audio" && kind != "video" {
        result.set(v8::null(scope).into());
        return;
    }
    result.set(capabilities(scope, &kind).into());
}

pub(crate) fn capabilities<'s>(
    scope: &v8::PinScope<'s, '_>,
    kind: &str,
) -> v8::Local<'s, v8::Object> {
    let output = v8::Object::new(scope);
    let configured = &crate::fingerprint::edge(scope).media;
    let codec_profiles = if kind == "audio" {
        &configured.rtc_audio_codecs
    } else {
        &configured.rtc_video_codecs
    };
    let codecs = v8::Array::new(scope, codec_profiles.len() as i32);
    for (index, codec) in codec_profiles.iter().enumerate() {
        let object = v8::Object::new(scope);
        define_string(scope, object, "mimeType", &codec.mime_type);
        define_number(scope, object, "clockRate", codec.clock_rate as i32);
        if let Some(channels) = codec.channels {
            define_number(scope, object, "channels", channels as i32);
        }
        if let Some(line) = &codec.sdp_fmtp_line {
            define_string(scope, object, "sdpFmtpLine", line);
        }
        let _ = codecs.set_index(scope, index as u32, object.into());
    }
    define_value(scope, output, "codecs", codecs.into());
    let header_profiles: Vec<_> = configured
        .rtc_header_extensions
        .iter()
        .filter(|extension| extension.kind == kind)
        .collect();
    let header_extensions = v8::Array::new(scope, header_profiles.len() as i32);
    for (index, extension) in header_profiles.into_iter().enumerate() {
        let object = v8::Object::new(scope);
        define_string(scope, object, "uri", &extension.uri);
        let _ = header_extensions.set_index(scope, index as u32, object.into());
    }
    define_value(scope, output, "headerExtensions", header_extensions.into());
    output
}

fn property_string(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    object
        .get(scope, key.into())
        .map(|value| crate::webidl::value_to_string(scope, value))
}

fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define_value(scope, object, name, value.into());
    }
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: i32,
) {
    define_value(scope, object, name, v8::Integer::new(scope, value).into());
}
