use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaRecorderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RecorderRecord>,
}

#[derive(Clone)]
struct RecorderRecord {
    object: v8::Global<v8::Object>,
    stream: v8::Global<v8::Object>,
    mime_type: String,
    state: String,
    onstart: Option<v8::Global<v8::Value>>,
    onstop: Option<v8::Global<v8::Value>>,
    ondataavailable: Option<v8::Global<v8::Value>>,
    onpause: Option<v8::Global<v8::Value>>,
    onresume: Option<v8::Global<v8::Value>>,
    onerror: Option<v8::Global<v8::Value>>,
    video_bits_per_second: u32,
    audio_bits_per_second: u32,
    audio_bitrate_mode: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaRecorderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaRecorder", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaRecorderStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "MediaRecorder",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "stream", get_stream)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mimeType", get_mime_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_accessor(scope, prototype, "onstart", get_onstart, set_onstart)?;
    crate::webidl::define_accessor(scope, prototype, "onstop", get_onstop, set_onstop)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ondataavailable",
        get_ondataavailable,
        set_ondataavailable,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onpause", get_onpause, set_onpause)?;
    crate::webidl::define_accessor(scope, prototype, "onresume", get_onresume, set_onresume)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "videoBitsPerSecond",
        get_video_bits_per_second,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "audioBitsPerSecond",
        get_audio_bits_per_second,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "audioBitrateMode",
        get_audio_bitrate_mode,
    )?;
    crate::webidl::define_method(scope, prototype, "pause", 0, pause)?;
    crate::webidl::define_method(scope, prototype, "requestData", 0, request_data)?;
    crate::webidl::define_method(scope, prototype, "resume", 0, resume)?;
    crate::webidl::define_method(scope, prototype, "start", 0, start)?;
    crate::webidl::define_method(scope, prototype, "stop", 0, stop)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "isTypeSupported",
        1,
        is_type_supported,
    )?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaRecorderStore>()
        .ok_or_else(|| "MediaRecorder state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaRecorder': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaRecorder': parameter 1 is not of type 'MediaStream'.",
        );
        return;
    };
    if !super::media_stream::is_stream(scope, stream) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaRecorder': parameter 1 is not of type 'MediaStream'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let mime_type = options
        .and_then(|options| string_property(scope, options, "mimeType"))
        .unwrap_or_default();
    if !crate::fingerprint_environment::media_capability_matches(
        &crate::fingerprint::edge(scope).media.media_recorder_types,
        &mime_type,
    ) {
        throw_dom_exception(
            scope,
            "NotSupportedError",
            "The specified MIME type is not supported.",
        );
        return;
    }
    let total_bits = options
        .and_then(|options| number_property(scope, options, "bitsPerSecond"))
        .map(|value| value.max(0.0) as u32);
    let audio_bits = options
        .and_then(|options| number_property(scope, options, "audioBitsPerSecond"))
        .map(|value| value.max(0.0) as u32)
        .or_else(|| total_bits.map(|value| value / 10))
        .unwrap_or(128_000);
    let video_bits = options
        .and_then(|options| number_property(scope, options, "videoBitsPerSecond"))
        .map(|value| value.max(0.0) as u32)
        .or_else(|| total_bits.map(|value| value - value / 10))
        .unwrap_or(2_500_000);
    let audio_bitrate_mode = options
        .and_then(|options| string_property(scope, options, "audioBitrateMode"))
        .filter(|value| value == "constant" || value == "variable")
        .unwrap_or_else(|| "variable".to_owned());
    super::event_target::attach(scope, arguments.this());
    let object = v8::Global::new(scope, arguments.this());
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<MediaRecorderStore>()
        .expect("MediaRecorder state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            RecorderRecord {
                object,
                stream,
                mime_type,
                state: "inactive".to_owned(),
                onstart: None,
                onstop: None,
                ondataavailable: None,
                onpause: None,
                onresume: None,
                onerror: None,
                video_bits_per_second: video_bits,
                audio_bits_per_second: audio_bits,
                audio_bitrate_mode,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RecorderRecord> {
    scope
        .get_slot::<MediaRecorderStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.stream).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&RecorderRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_mime_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.mime_type);
}
fn get_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.state);
}
fn get_audio_bitrate_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.audio_bitrate_mode);
}

fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&RecorderRecord) -> Option<&v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut RecorderRecord, Option<v8::Global<v8::Value>>),
) {
    let handler = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    if let Some(record) = scope
        .get_slot_mut::<MediaRecorderStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        update(record, handler);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_onstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onstart.as_ref());
}
fn set_onstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onstart = v);
}
fn get_onstop(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onstop.as_ref());
}
fn set_onstop(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onstop = v);
}
fn get_ondataavailable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.ondataavailable.as_ref());
}
fn set_ondataavailable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.ondataavailable = v);
}
fn get_onpause(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onpause.as_ref());
}
fn set_onpause(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onpause = v);
}
fn get_onresume(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onresume.as_ref());
}
fn set_onresume(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onresume = v);
}
fn get_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onerror.as_ref());
}
fn set_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onerror = v);
}

fn return_bits(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&RecorderRecord) -> u32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_video_bits_per_second(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bits(s, a, r, |x| x.video_bits_per_second);
}
fn get_audio_bits_per_second(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bits(s, a, r, |x| x.audio_bits_per_second);
}

fn pause(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    change_state(scope, arguments.this(), "recording", "paused", EVENT_PAUSE);
}

fn resume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    change_state(scope, arguments.this(), "paused", "recording", EVENT_RESUME);
}

fn start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    change_state(
        scope,
        arguments.this(),
        "inactive",
        "recording",
        EVENT_START,
    );
}

fn request_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.state == "inactive" {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "The MediaRecorder's state is inactive.",
        );
        return;
    }
    schedule_event(
        scope,
        arguments.this().get_identity_hash().get(),
        EVENT_DATA,
    );
}

fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<MediaRecorderStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.state == "inactive" {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "The MediaRecorder's state is inactive.",
        );
        return;
    }
    record.state = "inactive".to_owned();
    schedule_event(scope, id, EVENT_DATA);
    schedule_event(scope, id, EVENT_STOP);
}

fn change_state(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    required: &str,
    next: &str,
    event_code: i32,
) {
    let id = object.get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<MediaRecorderStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.state != required {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "The MediaRecorder's state is invalid.",
        );
        return;
    }
    record.state = next.to_owned();
    schedule_event(scope, id, event_code);
}

const EVENT_START: i32 = 1;
const EVENT_STOP: i32 = 2;
const EVENT_DATA: i32 = 3;
const EVENT_PAUSE: i32 = 4;
const EVENT_RESUME: i32 = 5;

fn schedule_event(scope: &mut v8::PinScope<'_, '_>, recorder_id: i32, event_code: i32) {
    let packed = ((recorder_id as i64) << 8) | event_code as i64;
    let data = v8::Number::new(scope, packed as f64);
    if let Some(function) = v8::Function::builder(deliver_event)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        scope.enqueue_microtask(function);
    }
}

fn deliver_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(packed) = arguments.data().integer_value(scope) else {
        return;
    };
    let recorder_id = (packed >> 8) as i32;
    let event_code = (packed & 255) as i32;
    let Some(record) = scope
        .get_slot::<MediaRecorderStore>()
        .and_then(|store| store.records.get(&recorder_id))
        .cloned()
    else {
        return;
    };
    let (event_type, handler) = match event_code {
        EVENT_START => ("start", record.onstart),
        EVENT_STOP => ("stop", record.onstop),
        EVENT_DATA => ("dataavailable", record.ondataavailable),
        EVENT_PAUSE => ("pause", record.onpause),
        EVENT_RESUME => ("resume", record.onresume),
        _ => return,
    };
    let event = super::event_target::create_event(scope, event_type);
    if event_code == EVENT_DATA
        && let Ok(blob) = super::blob::create(scope, Vec::new(), &record.mime_type)
    {
        define_data(scope, event, "data", blob.into());
        define_data(scope, event, "timecode", v8::Number::new(scope, 0.0).into());
    }
    let target = v8::Local::new(scope, &record.object);
    if let Some(handler) = handler
        && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
    let _ = super::event_target::dispatch(scope, target, event);
}

fn is_type_supported(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let configured = &crate::fingerprint::edge(scope).media.media_recorder_types;
    result.set(
        v8::Boolean::new(
            scope,
            crate::fingerprint_environment::media_capability_matches(configured, &media_type),
        )
        .into(),
    );
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        value.number_value(scope)
    }
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), value);
    }
}

fn throw_dom_exception(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), name.to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
