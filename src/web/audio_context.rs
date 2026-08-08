use std::collections::HashMap;

#[derive(Clone)]
struct AudioContextRecord {
    base_latency: f64,
    output_latency: f64,
    onerror: Option<v8::Global<v8::Value>>,
    sink_id: v8::Global<v8::Value>,
    onsinkchange: Option<v8::Global<v8::Value>>,
    playback_stats: v8::Global<v8::Object>,
    closed: bool,
}

#[derive(Default)]
pub(crate) struct AudioContextStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioContextRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioContext", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioContextStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioContext",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "baseLatency", get_base_latency)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "outputLatency", get_output_latency)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createMediaElementSource",
        1,
        create_media_element_source,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createMediaStreamDestination",
        0,
        create_media_stream_destination,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createMediaStreamSource",
        1,
        create_media_stream_source,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getOutputTimestamp",
        0,
        get_output_timestamp,
    )?;
    crate::webidl::define_method(scope, prototype, "resume", 0, resume)?;
    crate::webidl::define_method(scope, prototype, "suspend", 0, suspend)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "playbackStats", get_playback_stats)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sinkId", get_sink_id)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onsinkchange",
        get_onsinkchange,
        set_onsinkchange,
    )?;
    crate::webidl::define_method(scope, prototype, "setSinkId", 1, set_sink_id)?;
    let parent = super::base_audio_context::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioContextStore>()
        .ok_or_else(|| "AudioContext state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn option_number(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: f64,
) -> f64 {
    let Some(options) = options else {
        return default;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return default;
    };
    options
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(default)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioContext': Please use the 'new' operator",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let (configured_sample_rate, base_latency, output_latency) = {
        let audio_profile = &crate::fingerprint::edge(scope).rendering.audio;
        (
            audio_profile.sample_rate,
            audio_profile.base_latency,
            audio_profile.output_latency,
        )
    };
    let sample_rate = option_number(scope, options, "sampleRate", configured_sample_rate);
    if !sample_rate.is_finite() || !(3_000.0..=768_000.0).contains(&sample_rate) {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The provided sample rate is outside the supported range".to_owned(),
            "NotSupportedError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    if let Err(message) =
        super::base_audio_context::attach(scope, arguments.this(), sample_rate, "running", false)
    {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    let playback_stats = match super::audio_playback_stats::create(scope) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let sink_value: v8::Local<v8::Value> = v8::String::new(scope, "")
        .expect("empty AudioContext sink identifier")
        .into();
    let sink_id = v8::Global::new(scope, sink_value);
    let record = AudioContextRecord {
        base_latency,
        output_latency,
        onerror: None,
        sink_id,
        onsinkchange: None,
        playback_stats: v8::Global::new(scope, playback_stats),
        closed: false,
    };
    scope
        .get_slot_mut::<AudioContextStore>()
        .expect("AudioContext state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioContextRecord> {
    scope
        .get_slot::<AudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AudioContextRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_base_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.base_latency)
}
fn get_output_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.output_latency)
}

fn get_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.onerror {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, arguments.get(0)))
    };
    if let Some(record) = scope.get_slot_mut::<AudioContextStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.onerror = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_sink_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.sink_id));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_onsinkchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.onsinkchange {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_onsinkchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    if let Some(record) = scope.get_slot_mut::<AudioContextStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.onsinkchange = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_sink_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }

    let input = arguments.get(0);
    let sink_id: v8::Local<v8::Value> = if input.is_null_or_undefined() {
        reject_sink_change(
            scope,
            result,
            "Failed to execute 'setSinkId' on 'AudioContext': The provided value is not of type 'AudioSinkOptions'.",
            false,
        );
        return;
    } else if let Ok(options) = v8::Local::<v8::Object>::try_from(input) {
        let Some(type_key) = v8::String::new(scope, "type") else {
            return;
        };
        let sink_type = options
            .get(scope, type_key.into())
            .map(|value| crate::webidl::value_to_string(scope, value))
            .unwrap_or_default();
        if sink_type != "none" {
            reject_sink_change(
                scope,
                result,
                "Failed to execute 'setSinkId' on 'AudioContext': The provided value is not of type 'AudioSinkOptions'.",
                false,
            );
            return;
        }
        let Ok(info) = super::audio_sink_info::create(scope, "none") else {
            return;
        };
        info.into()
    } else {
        let identifier = crate::webidl::value_to_string(scope, input);
        if !identifier.is_empty() {
            reject_sink_change(
                scope,
                result,
                &format!("AudioContext.setSinkId(): failed: the device {identifier} is not found."),
                true,
            );
            return;
        }
        let Some(identifier) = v8::String::new(scope, "") else {
            return;
        };
        identifier.into()
    };
    let sink_id = v8::Global::new(scope, sink_id);
    if let Some(record) = scope.get_slot_mut::<AudioContextStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.sink_id = sink_id;
        resolved_undefined(scope, result);
    }
}

fn reject_sink_change(
    scope: &mut v8::PinScope<'_, '_>,
    mut result: v8::ReturnValue<'_>,
    message: &str,
    not_found: bool,
) {
    let exception: v8::Local<v8::Value> = if not_found {
        match super::dom_exception::create(scope, message.to_owned(), "NotFoundError".to_owned()) {
            Ok(exception) => exception.into(),
            Err(_) => return,
        }
    } else {
        let Some(message) = v8::String::new(scope, message) else {
            return;
        };
        v8::Exception::type_error(scope, message)
    };
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
        result.set(promise.into());
    }
}

fn resolved_undefined(scope: &mut v8::PinScope<'_, '_>, mut result: v8::ReturnValue<'_>) {
    let undefined = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
        result.set(promise.into());
    }
}

fn rejected_invalid_state(
    scope: &mut v8::PinScope<'_, '_>,
    mut result: v8::ReturnValue<'_>,
    message: &str,
) {
    let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "InvalidStateError".to_owned())
    else {
        return;
    };
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception.into()) {
        result.set(promise.into());
    }
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.closed {
        rejected_invalid_state(scope, result, "Cannot close a closed AudioContext.");
        return;
    }
    if let Some(record) = scope.get_slot_mut::<AudioContextStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.closed = true;
    }
    super::base_audio_context::set_state(scope, arguments.this(), "closed");
    resolved_undefined(scope, result);
}

fn resume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        rejected_invalid_state(scope, result, "Cannot resume a closed AudioContext.");
        return;
    }
    super::base_audio_context::set_state(scope, arguments.this(), "running");
    resolved_undefined(scope, result);
}

fn suspend(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        rejected_invalid_state(scope, result, "Cannot suspend a closed AudioContext.");
        return;
    }
    super::base_audio_context::set_state(scope, arguments.this(), "suspended");
    resolved_undefined(scope, result);
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
    }
}

fn get_output_timestamp(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let current_time =
        super::base_audio_context::current_time(scope, arguments.this()).unwrap_or(0.0);
    let context_time = if current_time <= 0.0 {
        0.0
    } else {
        (current_time - record.base_latency).max(0.0)
    };
    let performance_time = if context_time <= 0.0 {
        0.0
    } else {
        let now = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
        (now - (current_time - context_time) * 1_000.0).max(0.0)
    };
    let timestamp = v8::Object::new(scope);
    define_number(scope, timestamp, "contextTime", context_time);
    define_number(scope, timestamp, "performanceTime", performance_time);
    result.set(timestamp.into());
}

fn create_options<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
    value: v8::Local<'s, v8::Value>,
) -> Option<v8::Local<'s, v8::Object>> {
    let options = v8::Object::new(scope);
    let key = v8::String::new(scope, name)?;
    options
        .create_data_property(scope, key.into(), value)
        .filter(|success| *success)?;
    Some(options)
}

fn create_media_element_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(options) = create_options(scope, "mediaElement", arguments.get(0)) else {
        return;
    };
    let Ok(constructor) = super::media_element_audio_source_node::ensure_constructor(scope) else {
        return;
    };
    if let Some(node) = constructor.new_instance(scope, &[arguments.this().into(), options.into()])
    {
        result.set(node.into());
    }
}

fn create_media_stream_destination(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(constructor) = super::media_stream_audio_destination_node::ensure_constructor(scope)
    else {
        return;
    };
    if let Some(node) = constructor.new_instance(scope, &[arguments.this().into()]) {
        result.set(node.into());
    }
}

fn create_media_stream_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(options) = create_options(scope, "mediaStream", arguments.get(0)) else {
        return;
    };
    let Ok(constructor) = super::media_stream_audio_source_node::ensure_constructor(scope) else {
        return;
    };
    if let Some(node) = constructor.new_instance(scope, &[arguments.this().into(), options.into()])
    {
        result.set(node.into());
    }
}

fn get_playback_stats(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.playback_stats).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
