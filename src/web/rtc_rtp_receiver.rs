use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcRtpReceiverStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ReceiverRecord>,
}

#[derive(Clone)]
struct ReceiverRecord {
    track: v8::Global<v8::Object>,
    transport: Option<v8::Global<v8::Object>>,
    rtcp_transport: Option<v8::Global<v8::Object>>,
    playout_delay_hint: Option<f64>,
    jitter_buffer_target: Option<f64>,
    transform: Option<v8::Global<v8::Value>>,
    contributing_sources: Vec<v8::Global<v8::Object>>,
    synchronization_sources: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcRtpReceiverStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCRtpReceiver", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcRtpReceiverStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCRtpReceiver",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "track", get_track)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transport", get_transport)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rtcpTransport", get_rtcp_transport)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "playoutDelayHint",
        get_playout_delay_hint,
        set_playout_delay_hint,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createEncodedStreams",
        0,
        create_encoded_streams,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getContributingSources",
        0,
        get_contributing_sources,
    )?;
    crate::webidl::define_method(scope, prototype, "getParameters", 0, get_parameters)?;
    crate::webidl::define_method(scope, prototype, "getStats", 0, get_stats)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getSynchronizationSources",
        0,
        get_synchronization_sources,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "jitterBufferTarget",
        get_jitter_buffer_target,
        set_jitter_buffer_target,
    )?;
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
        .get_slot_mut::<RtcRtpReceiverStore>()
        .ok_or_else(|| "RTCRtpReceiver state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    track: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let receiver = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, receiver, prototype.into()) != Some(true) {
        return Err("cannot create RTCRtpReceiver".to_owned());
    }
    let track = v8::Global::new(scope, track);
    scope
        .get_slot_mut::<RtcRtpReceiverStore>()
        .ok_or_else(|| "RTCRtpReceiver state was not prepared".to_owned())?
        .records
        .insert(
            receiver.get_identity_hash().get(),
            ReceiverRecord {
                track,
                transport: None,
                rtcp_transport: None,
                playout_delay_hint: None,
                jitter_buffer_target: None,
                transform: None,
                contributing_sources: Vec::new(),
                synchronization_sources: Vec::new(),
            },
        );
    Ok(receiver)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCRtpReceiver': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ReceiverRecord> {
    scope
        .get_slot::<RtcRtpReceiverStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Local::new(scope, &record.track).into()),
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

fn return_optional_number(
    scope: &v8::PinScope<'_, '_>,
    value: Option<f64>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Number::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_playout_delay_hint(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_number(scope, record.playout_delay_hint, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_playout_delay_hint(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null() {
        None
    } else {
        arguments.get(0).number_value(scope)
    };
    if let Some(record) = scope
        .get_slot_mut::<RtcRtpReceiverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.playout_delay_hint = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_jitter_buffer_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => return_optional_number(scope, record.jitter_buffer_target, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_jitter_buffer_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null() {
        None
    } else {
        arguments.get(0).number_value(scope)
    };
    if let Some(record) = scope
        .get_slot_mut::<RtcRtpReceiverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.jitter_buffer_target = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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

fn source_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    values: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, value).into());
    }
    array
}

fn get_contributing_sources(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(source_array(scope, &record.contributing_sources).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_synchronization_sources(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(source_array(scope, &record.synchronization_sources).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let object = v8::Object::new(scope);
    define_value(scope, object, "codecs", v8::Array::new(scope, 0).into());
    define_value(
        scope,
        object,
        "headerExtensions",
        v8::Array::new(scope, 0).into(),
    );
    define_value(scope, object, "rtcp", v8::Object::new(scope).into());
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
    if let Some(record) = scope
        .get_slot_mut::<RtcRtpReceiverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
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
    if kind == "audio" || kind == "video" {
        result.set(super::rtc_rtp_sender::capabilities(scope, &kind).into());
    } else {
        result.set(v8::null(scope).into());
    }
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
