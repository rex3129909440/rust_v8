use std::collections::HashMap;

#[derive(Clone)]
struct BiquadFilterNodeRecord {
    filter_type: String,
    frequency: v8::Global<v8::Object>,
    detune: v8::Global<v8::Object>,
    q: v8::Global<v8::Object>,
    gain: v8::Global<v8::Object>,
    frequency_value: f64,
    detune_value: f64,
    q_value: f64,
    gain_value: f64,
    sample_rate: f64,
}

#[derive(Default)]
pub(crate) struct BiquadFilterNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BiquadFilterNodeRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BiquadFilterNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BiquadFilterNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<BiquadFilterNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BiquadFilterNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "type", get_type, set_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "frequency", get_frequency)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "detune", get_detune)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "Q", get_q)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "gain", get_gain)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getFrequencyResponse",
        3,
        get_frequency_response,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BiquadFilterNodeStore>()
        .ok_or_else(|| "BiquadFilterNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    constructor
        .new_instance(scope, &[context.into()])
        .ok_or_else(|| "cannot create BiquadFilterNode".to_owned())
}

fn option<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn number_option(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    fallback: f64,
) -> f64 {
    object
        .and_then(|object| option(scope, object, name))
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "BiquadFilterNode requires an audio context");
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'BiquadFilterNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'BiquadFilterNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let filter_type = options
        .and_then(|options| option(scope, options, "type"))
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .filter(|value| valid_type(value))
        .unwrap_or_else(|| "lowpass".to_owned());
    let frequency_value = number_option(scope, options, "frequency", 350.0);
    let detune_value = number_option(scope, options, "detune", 0.0);
    let q_value = number_option(scope, options, "Q", 1.0);
    let gain_value = number_option(scope, options, "gain", 0.0);
    let sample_rate = super::base_audio_context::sample_rate(scope, context)
        .unwrap_or_else(|| crate::fingerprint::edge(scope).rendering.audio.sample_rate);
    let frequency = match super::audio_param::create(
        scope,
        context,
        frequency_value as f32,
        0.0,
        (sample_rate / 2.0) as f32,
    ) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let detune = match super::audio_param::create(
        scope,
        context,
        detune_value as f32,
        -153600.0,
        153600.0,
    ) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let q = match super::audio_param::create(scope, context, q_value as f32, -1000.0, 1000.0) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let gain = match super::audio_param::create(scope, context, gain_value as f32, -40.0, 40.0) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    super::audio_node::attach(scope, arguments.this(), Some(context), 1, 1);
    let record = BiquadFilterNodeRecord {
        filter_type,
        frequency: v8::Global::new(scope, frequency),
        detune: v8::Global::new(scope, detune),
        q: v8::Global::new(scope, q),
        gain: v8::Global::new(scope, gain),
        frequency_value,
        detune_value,
        q_value,
        gain_value,
        sample_rate,
    };
    scope
        .get_slot_mut::<BiquadFilterNodeStore>()
        .expect("BiquadFilterNode state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn valid_type(value: &str) -> bool {
    matches!(
        value,
        "lowpass"
            | "highpass"
            | "bandpass"
            | "lowshelf"
            | "highshelf"
            | "peaking"
            | "notch"
            | "allpass"
    )
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BiquadFilterNodeRecord> {
    scope
        .get_slot::<BiquadFilterNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &record.filter_type)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !valid_type(&value) {
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<BiquadFilterNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.filter_type = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_param(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<v8::Global<v8::Object>>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_frequency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.frequency);
    return_param(s, value, r);
}
fn get_detune(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.detune);
    return_param(s, value, r);
}
fn get_q(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.q);
    return_param(s, value, r);
}
fn get_gain(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.gain);
    return_param(s, value, r);
}

fn coefficients(record: &BiquadFilterNodeRecord) -> (f64, f64, f64, f64, f64, f64) {
    let adjusted_frequency = record.frequency_value * 2_f64.powf(record.detune_value / 1200.0);
    let omega = 2.0 * std::f64::consts::PI * adjusted_frequency / record.sample_rate;
    let cosine = omega.cos();
    let sine = omega.sin();
    let alpha = sine / (2.0 * record.q_value.max(0.0001));
    match record.filter_type.as_str() {
        "highpass" => (
            (1.0 + cosine) / 2.0,
            -(1.0 + cosine),
            (1.0 + cosine) / 2.0,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
        "bandpass" => (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cosine, 1.0 - alpha),
        "notch" => (
            1.0,
            -2.0 * cosine,
            1.0,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
        "allpass" => (
            1.0 - alpha,
            -2.0 * cosine,
            1.0 + alpha,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
        _ => (
            (1.0 - cosine) / 2.0,
            1.0 - cosine,
            (1.0 - cosine) / 2.0,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
    }
}

pub(crate) fn normalized_coefficients_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<[f64; 5]> {
    let record = record(scope, object)?;
    let frequency = f64::from(super::audio_param::value_at(
        scope,
        v8::Local::new(scope, &record.frequency),
        time,
    )?);
    let detune = f64::from(super::audio_param::value_at(
        scope,
        v8::Local::new(scope, &record.detune),
        time,
    )?);
    let q = f64::from(super::audio_param::value_at(
        scope,
        v8::Local::new(scope, &record.q),
        time,
    )?);
    let gain = f64::from(super::audio_param::value_at(
        scope,
        v8::Local::new(scope, &record.gain),
        time,
    )?);
    let frequency =
        (frequency * 2.0_f64.powf(detune / 1_200.0)).clamp(0.0, record.sample_rate / 2.0);
    let omega = std::f64::consts::TAU * frequency / record.sample_rate;
    let cosine = omega.cos();
    let sine = omega.sin();
    let alpha = sine / (2.0 * q.abs().max(0.0001));
    let amplitude = 10.0_f64.powf(gain / 40.0);
    let (b0, b1, b2, a0, a1, a2) = match record.filter_type.as_str() {
        "highpass" => (
            (1.0 + cosine) / 2.0,
            -(1.0 + cosine),
            (1.0 + cosine) / 2.0,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
        "bandpass" => (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cosine, 1.0 - alpha),
        "notch" => (
            1.0,
            -2.0 * cosine,
            1.0,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
        "allpass" => (
            1.0 - alpha,
            -2.0 * cosine,
            1.0 + alpha,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
        "peaking" => (
            1.0 + alpha * amplitude,
            -2.0 * cosine,
            1.0 - alpha * amplitude,
            1.0 + alpha / amplitude,
            -2.0 * cosine,
            1.0 - alpha / amplitude,
        ),
        "lowshelf" => {
            let root = amplitude.sqrt();
            let shelf_alpha = sine / 2.0 * 2.0_f64.sqrt();
            (
                amplitude
                    * ((amplitude + 1.0) - (amplitude - 1.0) * cosine + 2.0 * root * shelf_alpha),
                2.0 * amplitude * ((amplitude - 1.0) - (amplitude + 1.0) * cosine),
                amplitude
                    * ((amplitude + 1.0) - (amplitude - 1.0) * cosine - 2.0 * root * shelf_alpha),
                (amplitude + 1.0) + (amplitude - 1.0) * cosine + 2.0 * root * shelf_alpha,
                -2.0 * ((amplitude - 1.0) + (amplitude + 1.0) * cosine),
                (amplitude + 1.0) + (amplitude - 1.0) * cosine - 2.0 * root * shelf_alpha,
            )
        }
        "highshelf" => {
            let root = amplitude.sqrt();
            let shelf_alpha = sine / 2.0 * 2.0_f64.sqrt();
            (
                amplitude
                    * ((amplitude + 1.0) + (amplitude - 1.0) * cosine + 2.0 * root * shelf_alpha),
                -2.0 * amplitude * ((amplitude - 1.0) + (amplitude + 1.0) * cosine),
                amplitude
                    * ((amplitude + 1.0) + (amplitude - 1.0) * cosine - 2.0 * root * shelf_alpha),
                (amplitude + 1.0) - (amplitude - 1.0) * cosine + 2.0 * root * shelf_alpha,
                2.0 * ((amplitude - 1.0) - (amplitude + 1.0) * cosine),
                (amplitude + 1.0) - (amplitude - 1.0) * cosine - 2.0 * root * shelf_alpha,
            )
        }
        _ => (
            (1.0 - cosine) / 2.0,
            1.0 - cosine,
            (1.0 - cosine) / 2.0,
            1.0 + alpha,
            -2.0 * cosine,
            1.0 - alpha,
        ),
    };
    Some([b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0])
}

fn response_at(record: &BiquadFilterNodeRecord, frequency: f64) -> (f64, f64) {
    let (b0, b1, b2, a0, a1, a2) = coefficients(record);
    let omega = 2.0 * std::f64::consts::PI * frequency / record.sample_rate;
    let cosine = omega.cos();
    let sine = omega.sin();
    let cosine2 = (2.0 * omega).cos();
    let sine2 = (2.0 * omega).sin();
    let numerator_real = b0 + b1 * cosine + b2 * cosine2;
    let numerator_imaginary = -b1 * sine - b2 * sine2;
    let denominator_real = a0 + a1 * cosine + a2 * cosine2;
    let denominator_imaginary = -a1 * sine - a2 * sine2;
    let denominator =
        denominator_real * denominator_real + denominator_imaginary * denominator_imaginary;
    let real = (numerator_real * denominator_real + numerator_imaginary * denominator_imaginary)
        / denominator;
    let imaginary = (numerator_imaginary * denominator_real
        - numerator_real * denominator_imaginary)
        / denominator;
    (
        (real * real + imaginary * imaginary).sqrt(),
        imaginary.atan2(real),
    )
}

fn get_frequency_response(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(frequencies) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "frequencyHz must be a float array");
        return;
    };
    let Ok(magnitudes) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "magResponse must be a float array");
        return;
    };
    let Ok(phases) = v8::Local::<v8::Object>::try_from(arguments.get(2)) else {
        crate::webidl::throw_type_error(scope, "phaseResponse must be a float array");
        return;
    };
    let Some(length_key) = v8::String::new(scope, "length") else {
        return;
    };
    let length = frequencies
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    for index in 0..length {
        let frequency = frequencies
            .get_index(scope, index)
            .and_then(|value| value.number_value(scope))
            .unwrap_or(f64::NAN);
        let (magnitude, phase) = if (0.0..=record.sample_rate / 2.0).contains(&frequency) {
            response_at(&record, frequency)
        } else {
            (f64::NAN, f64::NAN)
        };
        let _ = magnitudes.set_index(scope, index, v8::Number::new(scope, magnitude).into());
        let _ = phases.set_index(scope, index, v8::Number::new(scope, phase).into());
    }
}
