use std::collections::HashMap;

#[derive(Clone)]
struct AnalyserNodeRecord {
    fft_size: u32,
    min_decibels: f64,
    max_decibels: f64,
    smoothing_time_constant: f64,
    time_domain: Vec<f32>,
}

#[derive(Default)]
pub(crate) struct AnalyserNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AnalyserNodeRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnalyserNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AnalyserNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AnalyserNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AnalyserNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "fftSize", get_fft_size, set_fft_size)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "frequencyBinCount",
        get_frequency_bin_count,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "minDecibels",
        get_min_decibels,
        set_min_decibels,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "maxDecibels",
        get_max_decibels,
        set_max_decibels,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "smoothingTimeConstant",
        get_smoothing_time_constant,
        set_smoothing_time_constant,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getByteFrequencyData",
        1,
        get_byte_frequency_data,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getByteTimeDomainData",
        1,
        get_byte_time_domain_data,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getFloatFrequencyData",
        1,
        get_float_frequency_data,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getFloatTimeDomainData",
        1,
        get_float_time_domain_data,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AnalyserNodeStore>()
        .ok_or_else(|| "AnalyserNode state was not prepared".to_owned())?
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
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AnalyserNode': 1 argument required",
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
        return Err("AnalyserNode requires a BaseAudioContext".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let analyser = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, analyser, prototype.into()) != Some(true) {
        return Err("cannot create AnalyserNode".to_owned());
    }
    attach(scope, analyser, context, options)?;
    Ok(analyser)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let fft_size = option_number(scope, options, "fftSize", 2048.0) as u32;
    let min_decibels = option_number(scope, options, "minDecibels", -100.0);
    let max_decibels = option_number(scope, options, "maxDecibels", -30.0);
    let smoothing_time_constant = option_number(scope, options, "smoothingTimeConstant", 0.8);
    if !valid_fft_size(fft_size) {
        return Err("fftSize must be a power of two between 32 and 32768".to_owned());
    }
    if !min_decibels.is_finite() || !max_decibels.is_finite() || min_decibels >= max_decibels {
        return Err("minDecibels must be less than maxDecibels".to_owned());
    }
    if !(0.0..=1.0).contains(&smoothing_time_constant) {
        return Err("smoothingTimeConstant must be between zero and one".to_owned());
    }
    super::audio_node::attach(scope, object, Some(context), 1, 1);
    scope
        .get_slot_mut::<AnalyserNodeStore>()
        .ok_or_else(|| "AnalyserNode state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AnalyserNodeRecord {
                fft_size,
                min_decibels,
                max_decibels,
                smoothing_time_constant,
                time_domain: Vec::new(),
            },
        );
    Ok(())
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AnalyserNodeRecord> {
    scope
        .get_slot::<AnalyserNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut AnalyserNodeRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<AnalyserNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn valid_fft_size(value: u32) -> bool {
    (32..=32768).contains(&value) && value.is_power_of_two()
}

fn throw_index_size(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "IndexSizeError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn get_fft_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.fft_size).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_fft_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if !valid_fft_size(value) {
        throw_index_size(
            scope,
            "Failed to set 'fftSize': value must be a power of two between 32 and 32768.",
        );
        return;
    }
    update(scope, arguments.this(), |record| record.fft_size = value);
}
fn get_frequency_bin_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.fft_size / 2).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AnalyserNodeRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_min_decibels(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.min_decibels)
}
fn get_max_decibels(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.max_decibels)
}
fn get_smoothing_time_constant(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.smoothing_time_constant)
}

fn set_min_decibels(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !value.is_finite() || value >= record.max_decibels {
        throw_index_size(scope, "minDecibels must be less than maxDecibels.");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.min_decibels = value
    });
}
fn set_max_decibels(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !value.is_finite() || value <= record.min_decibels {
        throw_index_size(scope, "maxDecibels must be greater than minDecibels.");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.max_decibels = value
    });
}
fn set_smoothing_time_constant(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !(0.0..=1.0).contains(&value) {
        throw_index_size(scope, "smoothingTimeConstant must be between zero and one.");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.smoothing_time_constant = value
    });
}

fn ensure_brand(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    if record(scope, object).is_some() {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

fn fill_uint8(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    value: u32,
    amplitude: f32,
) {
    if !ensure_brand(scope, arguments.this()) {
        return;
    }
    let Ok(array) = v8::Local::<v8::Uint8Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "destination must be a Uint8Array");
        return;
    };
    for index in 0..array.length() as u32 {
        let noise = fingerprint_noise(scope, index, amplitude);
        let adjusted = (value as f32 + noise * 255.0).clamp(0.0, 255.0) as u32;
        let number = v8::Integer::new_from_unsigned(scope, adjusted);
        let _ = array.set_index(scope, index, number.into());
    }
}

fn fill_float32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    value: f64,
    amplitude: f32,
) {
    if !ensure_brand(scope, arguments.this()) {
        return;
    }
    let Ok(array) = v8::Local::<v8::Float32Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "destination must be a Float32Array");
        return;
    };
    for index in 0..array.length() as u32 {
        let noise = fingerprint_noise(scope, index, amplitude) as f64;
        let adjusted = if value.is_finite() {
            value + noise
        } else {
            value
        };
        let number = v8::Number::new(scope, adjusted);
        let _ = array.set_index(scope, index, number.into());
    }
}

fn get_byte_frequency_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if write_frequency_data(scope, arguments.this(), arguments.get(0), false) {
        return;
    }
    let amplitude = crate::fingerprint::edge(scope)
        .rendering
        .audio
        .frequency_noise_amplitude;
    fill_uint8(scope, arguments, 0, amplitude);
}
fn get_byte_time_domain_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if write_time_domain_data(scope, arguments.this(), arguments.get(0), false) {
        return;
    }
    let amplitude = crate::fingerprint::edge(scope)
        .rendering
        .audio
        .time_domain_noise_amplitude;
    fill_uint8(scope, arguments, 128, amplitude);
}
fn get_float_frequency_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if write_frequency_data(scope, arguments.this(), arguments.get(0), true) {
        return;
    }
    let amplitude = crate::fingerprint::edge(scope)
        .rendering
        .audio
        .frequency_noise_amplitude;
    fill_float32(scope, arguments, f64::NEG_INFINITY, amplitude);
}
fn get_float_time_domain_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if write_time_domain_data(scope, arguments.this(), arguments.get(0), true) {
        return;
    }
    let amplitude = crate::fingerprint::edge(scope)
        .rendering
        .audio
        .time_domain_noise_amplitude;
    fill_float32(scope, arguments, 0.0, amplitude);
}

pub(crate) fn capture_sample(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    sample: f32,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<AnalyserNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.time_domain.push(sample);
    if record.time_domain.len() > record.fft_size as usize {
        let excess = record.time_domain.len() - record.fft_size as usize;
        record.time_domain.drain(..excess);
    }
    true
}

fn write_time_domain_data(
    scope: &mut v8::PinScope<'_, '_>,
    analyser: v8::Local<'_, v8::Object>,
    destination: v8::Local<'_, v8::Value>,
    float_output: bool,
) -> bool {
    let Some(record) = record(scope, analyser) else {
        return false;
    };
    if record.time_domain.is_empty() {
        return false;
    }
    let Ok(destination) = v8::Local::<v8::Object>::try_from(destination) else {
        crate::webidl::throw_type_error(scope, "destination must be a typed array");
        return true;
    };
    let length_key = v8::String::new(scope, "length").expect("length key");
    let length = destination
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let available = record.time_domain.len();
    let offset = available.saturating_sub(length as usize);
    for index in 0..length {
        let sample = record
            .time_domain
            .get(offset + index as usize)
            .copied()
            .unwrap_or(0.0);
        let value = if float_output {
            f64::from(sample)
        } else {
            f64::from(((sample.clamp(-1.0, 1.0) + 1.0) * 128.0).clamp(0.0, 255.0))
        };
        let _ = destination.set_index(scope, index, v8::Number::new(scope, value).into());
    }
    true
}

fn write_frequency_data(
    scope: &mut v8::PinScope<'_, '_>,
    analyser: v8::Local<'_, v8::Object>,
    destination: v8::Local<'_, v8::Value>,
    float_output: bool,
) -> bool {
    let Some(record) = record(scope, analyser) else {
        return false;
    };
    if record.time_domain.is_empty() {
        return false;
    }
    let Ok(destination) = v8::Local::<v8::Object>::try_from(destination) else {
        crate::webidl::throw_type_error(scope, "destination must be a typed array");
        return true;
    };
    let spectrum = fft_decibels(&record);
    let length_key = v8::String::new(scope, "length").expect("length key");
    let length = destination
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0)
        .min(spectrum.len() as u32);
    for index in 0..length {
        let decibels = spectrum[index as usize];
        let value = if float_output {
            decibels
        } else {
            ((decibels - record.min_decibels) / (record.max_decibels - record.min_decibels) * 255.0)
                .clamp(0.0, 255.0)
        };
        let _ = destination.set_index(scope, index, v8::Number::new(scope, value).into());
    }
    true
}

fn fft_decibels(record: &AnalyserNodeRecord) -> Vec<f64> {
    let length = record.fft_size as usize;
    let mut values = vec![(0.0_f64, 0.0_f64); length];
    let source_offset = record.time_domain.len().saturating_sub(length);
    let target_offset = length.saturating_sub(record.time_domain.len());
    for (index, sample) in record.time_domain[source_offset..].iter().enumerate() {
        let target = target_offset + index;
        let window = 0.5
            * (1.0
                - (std::f64::consts::TAU * target as f64 / (length.saturating_sub(1)) as f64)
                    .cos());
        values[target].0 = f64::from(*sample) * window;
    }
    let mut j = 0;
    for index in 1..length {
        let mut bit = length >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if index < j {
            values.swap(index, j);
        }
    }
    let mut span = 2;
    while span <= length {
        let angle = -std::f64::consts::TAU / span as f64;
        let step = (angle.cos(), angle.sin());
        for start in (0..length).step_by(span) {
            let mut twiddle = (1.0, 0.0);
            for offset in 0..span / 2 {
                let even = values[start + offset];
                let odd = values[start + offset + span / 2];
                let transformed = (
                    odd.0 * twiddle.0 - odd.1 * twiddle.1,
                    odd.0 * twiddle.1 + odd.1 * twiddle.0,
                );
                values[start + offset] = (even.0 + transformed.0, even.1 + transformed.1);
                values[start + offset + span / 2] =
                    (even.0 - transformed.0, even.1 - transformed.1);
                twiddle = (
                    twiddle.0 * step.0 - twiddle.1 * step.1,
                    twiddle.0 * step.1 + twiddle.1 * step.0,
                );
            }
        }
        span *= 2;
    }
    values
        .into_iter()
        .take(length / 2)
        .map(|(real, imaginary)| {
            let magnitude = real.hypot(imaginary) / length as f64;
            if magnitude > 0.0 {
                20.0 * magnitude.log10()
            } else {
                f64::NEG_INFINITY
            }
        })
        .collect()
}

fn fingerprint_noise(scope: &v8::PinScope<'_, '_>, index: u32, amplitude: f32) -> f32 {
    if amplitude == 0.0 {
        return 0.0;
    }
    let seed = crate::fingerprint::edge(scope).rendering.audio.noise_seed;
    let mut value = seed ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    let unit = ((value >> 40) as f32) / ((1_u32 << 24) as f32);
    (unit * 2.0 - 1.0) * amplitude
}
