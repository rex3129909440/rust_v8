use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct OscillatorNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, OscillatorRecord>,
}

#[derive(Clone)]
struct OscillatorRecord {
    oscillator_type: String,
    frequency: v8::Global<v8::Object>,
    detune: v8::Global<v8::Object>,
    periodic_wave: Option<v8::Global<v8::Object>>,
    sample_rate: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OscillatorNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "OscillatorNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<OscillatorNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "OscillatorNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "type", get_type, set_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "frequency", get_frequency)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "detune", get_detune)?;
    crate::webidl::define_method(scope, prototype, "setPeriodicWave", 1, set_periodic_wave)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_scheduled_source_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<OscillatorNodeStore>()
        .ok_or_else(|| "OscillatorNode state was not prepared".to_owned())?
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
            "Failed to construct 'OscillatorNode': 1 argument required",
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
    match attach(scope, arguments.this(), context, options) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if !super::base_audio_context::is_context(scope, context) {
        return Err("OscillatorNode requires a BaseAudioContext".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let oscillator = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, oscillator, prototype.into()) != Some(true) {
        return Err("cannot create OscillatorNode".to_owned());
    }
    attach(scope, oscillator, context, options)?;
    Ok(oscillator)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let oscillator_type = option_string(scope, options, "type", "sine");
    if !matches!(
        oscillator_type.as_str(),
        "sine" | "square" | "sawtooth" | "triangle"
    ) {
        return Err("The provided OscillatorType is invalid".to_owned());
    }
    let frequency_value = option_number(scope, options, "frequency", 440.0) as f32;
    let detune_value = option_number(scope, options, "detune", 0.0) as f32;
    let sample_rate = super::base_audio_context::sample_rate(scope, context)
        .unwrap_or_else(|| crate::fingerprint::edge(scope).rendering.audio.sample_rate);
    let nyquist = (sample_rate / 2.0) as f32;
    let frequency = super::audio_param::create(scope, context, 440.0, -nyquist, nyquist)?;
    let detune = super::audio_param::create(scope, context, 0.0, -153600.0, 153600.0)?;
    set_param_value(scope, frequency, frequency_value);
    set_param_value(scope, detune, detune_value);
    super::audio_node::attach(scope, object, Some(context), 0, 1);
    super::audio_scheduled_source_node::attach(scope, object);
    let channel_count = option_number(scope, options, "channelCount", 2.0) as u32;
    let _ = super::audio_node::set_channel_configuration(
        scope,
        object,
        channel_count.max(1),
        "max".to_owned(),
        "speakers".to_owned(),
    );
    let frequency = v8::Global::new(scope, frequency);
    let detune = v8::Global::new(scope, detune);
    scope
        .get_slot_mut::<OscillatorNodeStore>()
        .ok_or_else(|| "OscillatorNode state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            OscillatorRecord {
                oscillator_type,
                frequency,
                detune,
                periodic_wave: None,
                sample_rate,
            },
        );
    Ok(())
}

fn set_param_value(scope: &v8::PinScope<'_, '_>, parameter: v8::Local<'_, v8::Object>, value: f32) {
    if let Some(key) = v8::String::new(scope, "value") {
        let _ = parameter.set(
            scope,
            key.into(),
            v8::Number::new(scope, value as f64).into(),
        );
    }
}

fn option_number(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: f64,
) -> f64 {
    options
        .map(|options| super::event::number_property(scope, options, name, default))
        .unwrap_or(default)
}

fn option_string(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: &str,
) -> String {
    let Some(options) = options else {
        return default.to_owned();
    };
    let Some(key) = v8::String::new(scope, name) else {
        return default.to_owned();
    };
    let Some(value) = options.get(scope, key.into()) else {
        return default.to_owned();
    };
    if value.is_undefined() {
        default.to_owned()
    } else {
        crate::webidl::value_to_string(scope, value)
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<OscillatorRecord> {
    scope
        .get_slot::<OscillatorNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.oscillator_type) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value == "custom" {
        let custom_allowed =
            record(scope, arguments.this()).is_some_and(|record| record.periodic_wave.is_some());
        if !custom_allowed {
            if let Ok(exception) = super::dom_exception::create(
                scope,
                "The oscillator has no custom periodic wave".to_owned(),
                "InvalidStateError".to_owned(),
            ) {
                scope.throw_exception(exception.into());
            }
            return;
        }
    } else if !matches!(value.as_str(), "sine" | "square" | "sawtooth" | "triangle") {
        crate::webidl::throw_type_error(scope, "The provided OscillatorType is invalid");
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<OscillatorNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.oscillator_type = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_param(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&OscillatorRecord) -> v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_frequency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |record| record.frequency.clone());
}
fn get_detune(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |record| record.detune.clone());
}

fn set_periodic_wave(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(wave) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'PeriodicWave'");
        return;
    };
    if !super::periodic_wave::is_periodic_wave(scope, wave) {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'PeriodicWave'");
        return;
    }
    let wave = v8::Global::new(scope, wave);
    if let Some(record) = scope
        .get_slot_mut::<OscillatorNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.periodic_wave = Some(wave);
        record.oscillator_type = "custom".to_owned();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn sample_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<f32> {
    let record = record(scope, object)?;
    if !super::audio_scheduled_source_node::is_active_at(scope, object, time) {
        return Some(0.0);
    }
    let started_at = super::audio_scheduled_source_node::started_at(scope, object)?;
    let frequency = v8::Local::new(scope, &record.frequency);
    let detune = v8::Local::new(scope, &record.detune);
    let frequency =
        f64::from(super::audio_param::value_at(scope, frequency, time).unwrap_or(440.0));
    let detune = f64::from(super::audio_param::value_at(scope, detune, time).unwrap_or(0.0));
    let computed_frequency = frequency * 2.0_f64.powf(detune / 1_200.0);
    let phase = std::f64::consts::TAU * computed_frequency * (time - started_at);
    let frame = ((time - started_at) * record.sample_rate).round();
    let rate_scale = 4_096.0_f32 / record.sample_rate as f32;
    let phase_increment = computed_frequency as f32 * rate_scale;
    let virtual_read_index = frame * f64::from(phase_increment);
    let sample = match record.oscillator_type.as_str() {
        "square" => band_limited_square(virtual_read_index, computed_frequency, record.sample_rate),
        "sawtooth" => {
            band_limited_sawtooth(virtual_read_index, computed_frequency, record.sample_rate)
        }
        "triangle" => {
            band_limited_triangle(virtual_read_index, computed_frequency, record.sample_rate)
        }
        "custom" => record
            .periodic_wave
            .as_ref()
            .and_then(|wave| {
                super::periodic_wave::sample(scope, v8::Local::new(scope, wave), phase)
            })
            .unwrap_or(0.0),
        _ => phase.sin() as f32,
    };
    Some(sample)
}

fn harmonic_limit(frequency: f64, sample_rate: f64) -> usize {
    let frequency = frequency.abs();
    if !frequency.is_finite() || frequency <= f64::EPSILON {
        return 0;
    }
    ((sample_rate * 0.5 / frequency).floor() as usize).min(4_096)
}

fn periodic_wave_interpolate(virtual_read_index: f64, sample: impl Fn(f64) -> f32) -> f32 {
    const TABLE_SIZE: usize = 4_096;
    let wrapped = virtual_read_index.rem_euclid(TABLE_SIZE as f64);
    let first_index = wrapped as usize;
    let second_index = (first_index + 1) & (TABLE_SIZE - 1);
    let factor = wrapped as f32 - first_index as f32;
    let first = sample(std::f64::consts::TAU * first_index as f64 / TABLE_SIZE as f64);
    let second = sample(std::f64::consts::TAU * second_index as f64 / TABLE_SIZE as f64);
    (1.0 - factor) * first + factor * second
}

fn band_limited_square(virtual_read_index: f64, frequency: f64, sample_rate: f64) -> f32 {
    let limit = harmonic_limit(frequency, sample_rate);
    periodic_wave_interpolate(virtual_read_index, |phase| {
        let mut value = 0.0;
        for harmonic in (1..=limit).step_by(2) {
            value += (harmonic as f64 * phase).sin() / harmonic as f64;
        }
        (4.0 / std::f64::consts::PI * value) as f32
    })
}

fn band_limited_sawtooth(virtual_read_index: f64, frequency: f64, sample_rate: f64) -> f32 {
    let limit = harmonic_limit(frequency, sample_rate);
    periodic_wave_interpolate(virtual_read_index, |phase| {
        let mut value = 0.0;
        for harmonic in 1..=limit {
            let sign = if harmonic % 2 == 0 { -1.0 } else { 1.0 };
            value += sign * (harmonic as f64 * phase).sin() / harmonic as f64;
        }
        (2.0 / std::f64::consts::PI * value) as f32
    })
}

fn band_limited_triangle(virtual_read_index: f64, frequency: f64, sample_rate: f64) -> f32 {
    let limit = harmonic_limit(frequency, sample_rate);
    periodic_wave_interpolate(virtual_read_index, |phase| {
        let mut value = 0.0;
        for harmonic in (1..=limit).step_by(2) {
            let sign = if harmonic % 4 == 1 { 1.0 } else { -1.0 };
            let harmonic = harmonic as f64;
            value += sign * (harmonic * phase).sin() / (harmonic * harmonic);
        }
        // Blink normalizes all band-limited tables with the peak measured
        // from the full 4096-sample triangle table. Retain that normalization
        // when a high pitch culls the upper partials.
        const TRIANGLE_TABLE_NORMALIZATION: f64 = 1.000_197_787_918_47;
        (TRIANGLE_TABLE_NORMALIZATION * 8.0 / (std::f64::consts::PI * std::f64::consts::PI) * value)
            as f32
    })
}
