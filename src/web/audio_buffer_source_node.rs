use std::collections::HashMap;

#[derive(Clone)]
struct AudioBufferSourceNodeRecord {
    buffer: Option<v8::Global<v8::Object>>,
    playback_rate: v8::Global<v8::Object>,
    detune: v8::Global<v8::Object>,
    loop_enabled: bool,
    loop_start: f64,
    loop_end: f64,
    start_offset: Option<f64>,
    start_duration: Option<f64>,
}

#[derive(Default)]
pub(crate) struct AudioBufferSourceNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioBufferSourceNodeRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioBufferSourceNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioBufferSourceNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioBufferSourceNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioBufferSourceNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "buffer", get_buffer, set_buffer)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "playbackRate", get_playback_rate)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "detune", get_detune)?;
    crate::webidl::define_accessor(scope, prototype, "loop", get_loop, set_loop)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "loopStart",
        get_loop_start,
        set_loop_start,
    )?;
    crate::webidl::define_accessor(scope, prototype, "loopEnd", get_loop_end, set_loop_end)?;
    crate::webidl::define_method(scope, prototype, "start", 0, start)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_scheduled_source_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioBufferSourceNodeStore>()
        .ok_or_else(|| "AudioBufferSourceNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn option<'s>(
    scope: &v8::PinScope<'s, '_>,
    options: Option<v8::Local<'s, v8::Object>>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let options = options?;
    let key = v8::String::new(scope, name)?;
    let value = options.get(scope, key.into())?;
    (!value.is_undefined()).then_some(value)
}

fn option_number(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: f64,
) -> f64 {
    option(scope, options, name)
        .and_then(|value| value.number_value(scope))
        .unwrap_or(default)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioBufferSourceNode': 1 argument required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    if let Err(message) = attach(scope, arguments.this(), context, options) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if !super::base_audio_context::is_context(scope, context) {
        return Err("AudioBufferSourceNode requires a BaseAudioContext".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let source = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, source, prototype.into()) != Some(true) {
        return Err("cannot create AudioBufferSourceNode".to_owned());
    }
    attach(scope, source, context, options)?;
    Ok(source)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let buffer = match option(scope, options, "buffer") {
        Some(value) if value.is_null() => None,
        Some(value) => {
            let object = v8::Local::<v8::Object>::try_from(value)
                .map_err(|_| "buffer is not an AudioBuffer".to_owned())?;
            if !super::audio_buffer::is_buffer(scope, object) {
                return Err("buffer is not an AudioBuffer".to_owned());
            }
            Some(v8::Global::new(scope, object))
        }
        None => None,
    };
    let playback_rate_value = option_number(scope, options, "playbackRate", 1.0) as f32;
    let detune_value = option_number(scope, options, "detune", 0.0) as f32;
    let playback_rate = super::audio_param::create(scope, context, 1.0, -f32::MAX, f32::MAX)?;
    let detune = super::audio_param::create(scope, context, 0.0, -f32::MAX, f32::MAX)?;
    super::audio_param::set_current_value(scope, playback_rate, playback_rate_value);
    super::audio_param::set_current_value(scope, detune, detune_value);
    let loop_enabled =
        option(scope, options, "loop").is_some_and(|value| value.boolean_value(scope));
    let loop_start = option_number(scope, options, "loopStart", 0.0);
    let loop_end = option_number(scope, options, "loopEnd", 0.0);
    if !loop_start.is_finite() || !loop_end.is_finite() {
        return Err("loop boundaries must be finite".to_owned());
    }
    super::audio_node::attach(scope, object, Some(context), 0, 1);
    super::audio_scheduled_source_node::attach(scope, object);
    let playback_rate = v8::Global::new(scope, playback_rate);
    let detune = v8::Global::new(scope, detune);
    scope
        .get_slot_mut::<AudioBufferSourceNodeStore>()
        .ok_or_else(|| "AudioBufferSourceNode state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AudioBufferSourceNodeRecord {
                buffer,
                playback_rate,
                detune,
                loop_enabled,
                loop_start,
                loop_end,
                start_offset: None,
                start_duration: None,
            },
        );
    Ok(())
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioBufferSourceNodeRecord> {
    scope
        .get_slot::<AudioBufferSourceNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.buffer {
            Some(buffer) => result.set(v8::Local::new(scope, &buffer).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some_and(|record| record.buffer.is_some())
        && !arguments.get(0).is_null()
    {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "Cannot set buffer to non-null after it has already been set to a non-null buffer"
                .to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let buffer = if arguments.get(0).is_null() {
        None
    } else {
        let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "buffer is not an AudioBuffer");
            return;
        };
        if !super::audio_buffer::is_buffer(scope, object) {
            crate::webidl::throw_type_error(scope, "buffer is not an AudioBuffer");
            return;
        }
        Some(v8::Global::new(scope, object))
    };
    if let Some(record) = scope
        .get_slot_mut::<AudioBufferSourceNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.buffer = buffer;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AudioBufferSourceNodeRecord) -> &v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_playback_rate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.playback_rate)
}
fn get_detune(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.detune)
}

fn get_loop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.loop_enabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_loop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.loop_enabled = value
    });
}

fn return_f64(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AudioBufferSourceNodeRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_loop_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_f64(s, a, r, |v| v.loop_start)
}
fn get_loop_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_f64(s, a, r, |v| v.loop_end)
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut AudioBufferSourceNodeRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<AudioBufferSourceNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_loop_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "loopStart must be finite");
        return;
    }
    update(scope, arguments.this(), |record| record.loop_start = value);
}
fn set_loop_end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "loopEnd must be finite");
        return;
    }
    update(scope, arguments.this(), |record| record.loop_end = value);
}

fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>) {
    if let Ok(exception) = super::dom_exception::create(
        scope,
        "Failed to execute 'start' on 'AudioBufferSourceNode': cannot call start more than once."
            .to_owned(),
        "InvalidStateError".to_owned(),
    ) {
        scope.throw_exception(exception.into());
    }
}

fn start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let when = if arguments.get(0).is_undefined() {
        0.0
    } else {
        arguments.get(0).number_value(scope).unwrap_or(f64::NAN)
    };
    let offset = if arguments.get(1).is_undefined() {
        0.0
    } else {
        arguments.get(1).number_value(scope).unwrap_or(f64::NAN)
    };
    let duration = if arguments.get(2).is_undefined() {
        None
    } else {
        arguments.get(2).number_value(scope)
    };
    if !when.is_finite() || !offset.is_finite() || duration.is_some_and(|value| !value.is_finite())
    {
        crate::webidl::throw_type_error(scope, "start times must be finite and non-negative");
        return;
    }
    if when < 0.0 || offset < 0.0 || duration.is_some_and(|value| value < 0.0) {
        if let Some(message) = v8::String::new(
            scope,
            "The provided start time, offset, and duration must be non-negative",
        ) {
            scope.throw_exception(v8::Exception::range_error(scope, message));
        }
        return;
    }
    let snapshot = record(scope, arguments.this());
    let natural_end = snapshot.as_ref().and_then(|record| {
        let playback_rate = v8::Local::new(scope, &record.playback_rate);
        let detune = v8::Local::new(scope, &record.detune);
        let rate =
            f64::from(super::audio_param::value_at(scope, playback_rate, when).unwrap_or(1.0))
                * 2.0_f64.powf(
                    f64::from(super::audio_param::value_at(scope, detune, when).unwrap_or(0.0))
                        / 1_200.0,
                );
        if !rate.is_finite() || rate.abs() <= f64::EPSILON {
            return None;
        }
        if let Some(duration) = duration {
            return Some(when + duration / rate.abs());
        }
        if record.loop_enabled {
            return None;
        }
        let buffer_duration = record
            .buffer
            .as_ref()
            .and_then(|buffer| super::audio_buffer::duration(scope, v8::Local::new(scope, buffer)))
            .unwrap_or(0.0);
        Some(when + (buffer_duration - offset).max(0.0) / rate.abs())
    });
    match super::audio_scheduled_source_node::mark_started(scope, arguments.this(), when) {
        Ok(()) => {
            update(scope, arguments.this(), |record| {
                record.start_offset = Some(offset);
                record.start_duration = duration;
            });
            super::audio_scheduled_source_node::set_natural_end(
                scope,
                arguments.this(),
                natural_end,
            );
        }
        Err(super::audio_scheduled_source_node::StartSourceError::AlreadyStarted) => {
            throw_invalid_state(scope)
        }
        Err(super::audio_scheduled_source_node::StartSourceError::IllegalInvocation) => {
            crate::webidl::throw_type_error(scope, "Illegal invocation")
        }
    }
}

pub(crate) fn sample_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    channel: u32,
    time: f64,
) -> Option<f32> {
    let record = record(scope, object)?;
    if !super::audio_scheduled_source_node::is_active_at(scope, object, time) {
        return Some(0.0);
    }
    let started_at = super::audio_scheduled_source_node::started_at(scope, object)?;
    let buffer = v8::Local::new(scope, record.buffer.as_ref()?);
    let buffer_duration = super::audio_buffer::duration(scope, buffer)?;
    let playback_rate = v8::Local::new(scope, &record.playback_rate);
    let detune = v8::Local::new(scope, &record.detune);
    let playback_rate =
        f64::from(super::audio_param::value_at(scope, playback_rate, time).unwrap_or(1.0));
    let detune = f64::from(super::audio_param::value_at(scope, detune, time).unwrap_or(0.0));
    let rate = playback_rate * 2.0_f64.powf(detune / 1_200.0);
    let mut buffer_time = record.start_offset.unwrap_or(0.0) + (time - started_at).max(0.0) * rate;
    if record.loop_enabled {
        let loop_start = record.loop_start.clamp(0.0, buffer_duration);
        let loop_end = if record.loop_end > loop_start {
            record.loop_end.min(buffer_duration)
        } else {
            buffer_duration
        };
        let span = loop_end - loop_start;
        if span > 0.0 && buffer_time >= loop_end {
            buffer_time = loop_start + (buffer_time - loop_start).rem_euclid(span);
        }
    }
    if buffer_time < 0.0 || buffer_time >= buffer_duration {
        return Some(0.0);
    }
    let sample_rate =
        super::base_audio_context::sample_rate(scope, super::audio_node::context(scope, object)?)?;
    let sample_index = (buffer_time * sample_rate).floor() as u32;
    let channels = super::audio_buffer::number_of_channels(scope, buffer)?;
    super::audio_buffer::sample(
        scope,
        buffer,
        channel.min(channels.saturating_sub(1)),
        sample_index,
    )
}
