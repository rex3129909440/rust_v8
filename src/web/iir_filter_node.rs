use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IirFilterNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IirFilterNodeRecord>,
}

#[derive(Clone)]
struct IirFilterNodeRecord {
    feedforward: Vec<f64>,
    feedback: Vec<f64>,
    sample_rate: f64,
}

pub(crate) fn coefficients(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(Vec<f64>, Vec<f64>)> {
    let record = scope
        .get_slot::<IirFilterNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())?;
    Some((record.feedforward.clone(), record.feedback.clone()))
}

pub(crate) struct IirError {
    pub name: &'static str,
    pub message: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IirFilterNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IIRFilterNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<IirFilterNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IIRFilterNode",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
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
        .get_slot_mut::<IirFilterNodeStore>()
        .ok_or_else(|| "IIRFilterNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'IIRFilterNode': 2 arguments required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'IIRFilterNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'IIRFilterNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "IIRFilterOptions must be an object");
        return;
    };
    let Some(feedforward) = sequence_property(scope, options, "feedforward") else {
        crate::webidl::throw_type_error(scope, "Required member feedforward is undefined");
        return;
    };
    let Some(feedback) = sequence_property(scope, options, "feedback") else {
        crate::webidl::throw_type_error(scope, "Required member feedback is undefined");
        return;
    };
    match attach(
        scope,
        arguments.this(),
        context,
        feedforward,
        feedback,
        Some(options),
    ) {
        Ok(()) => result.set(arguments.this().into()),
        Err(error) => throw_iir_error(scope, error),
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    feedforward: Vec<f64>,
    feedback: Vec<f64>,
) -> Result<v8::Local<'s, v8::Object>, IirError> {
    if !super::base_audio_context::is_context(scope, context) {
        return Err(IirError {
            name: "TypeError",
            message: "IIRFilterNode requires a BaseAudioContext".to_owned(),
        });
    }
    let constructor = ensure_constructor(scope).map_err(|message| IirError {
        name: "TypeError",
        message,
    })?;
    let prototype = crate::webidl::prototype(scope, constructor).map_err(|message| IirError {
        name: "TypeError",
        message,
    })?;
    let node = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, node, prototype.into()) != Some(true) {
        return Err(IirError {
            name: "TypeError",
            message: "cannot create IIRFilterNode".to_owned(),
        });
    }
    attach(scope, node, context, feedforward, feedback, None)?;
    Ok(node)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
    feedforward: Vec<f64>,
    feedback: Vec<f64>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), IirError> {
    validate_coefficients(&feedforward, &feedback)?;
    let sample_rate = super::base_audio_context::sample_rate(scope, context).unwrap_or(48000.0);
    super::audio_node::attach(scope, object, Some(context), 1, 1);
    let channel_count = options
        .map(|options| super::event::number_property(scope, options, "channelCount", 2.0) as u32)
        .unwrap_or(2)
        .max(1);
    let channel_count_mode = option_string(scope, options, "channelCountMode", "max");
    let channel_interpretation = option_string(scope, options, "channelInterpretation", "speakers");
    let _ = super::audio_node::set_channel_configuration(
        scope,
        object,
        channel_count,
        channel_count_mode,
        channel_interpretation,
    );
    scope
        .get_slot_mut::<IirFilterNodeStore>()
        .ok_or_else(|| IirError {
            name: "TypeError",
            message: "IIRFilterNode state was not prepared".to_owned(),
        })?
        .records
        .insert(
            object.get_identity_hash().get(),
            IirFilterNodeRecord {
                feedforward,
                feedback,
                sample_rate,
            },
        );
    Ok(())
}

fn validate_coefficients(feedforward: &[f64], feedback: &[f64]) -> Result<(), IirError> {
    if !(1..=20).contains(&feedforward.len()) {
        return Err(IirError {
            name: "NotSupportedError",
            message: format!(
                "The number of feedforward coefficients provided ({}) is outside the range [1, 20].",
                feedforward.len()
            ),
        });
    }
    if !(1..=20).contains(&feedback.len()) {
        return Err(IirError {
            name: "NotSupportedError",
            message: format!(
                "The number of feedback coefficients provided ({}) is outside the range [1, 20].",
                feedback.len()
            ),
        });
    }
    if feedback[0] == 0.0 {
        return Err(IirError {
            name: "InvalidStateError",
            message: "The first feedback coefficient must not be zero.".to_owned(),
        });
    }
    if feedforward.iter().all(|coefficient| *coefficient == 0.0) {
        return Err(IirError {
            name: "InvalidStateError",
            message: "At least one feedforward coefficient must be non-zero.".to_owned(),
        });
    }
    Ok(())
}

fn sequence_property(
    scope: &v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<Vec<f64>> {
    let key = v8::String::new(scope, name)?;
    let value = options.get(scope, key.into())?;
    if value.is_undefined() {
        return None;
    }
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let length_key = v8::String::new(scope, "length")?;
    let length = object.get(scope, length_key.into())?.uint32_value(scope)?;
    let mut coefficients = Vec::with_capacity(length as usize);
    for index in 0..length {
        coefficients.push(
            object
                .get_index(scope, index)
                .and_then(|value| value.number_value(scope))
                .unwrap_or(f64::NAN),
        );
    }
    Some(coefficients)
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
    options
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_else(|| default.to_owned())
}

fn throw_iir_error(scope: &mut v8::PinScope<'_, '_>, error: IirError) {
    if error.name == "TypeError" {
        crate::webidl::throw_type_error(scope, &error.message);
    } else if let Ok(exception) =
        super::dom_exception::create(scope, error.message, error.name.to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IirFilterNodeRecord> {
    scope
        .get_slot::<IirFilterNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
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
    let Ok(frequency_hz) = v8::Local::<v8::Float32Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "frequencyHz must be a Float32Array");
        return;
    };
    let Ok(mag_response) = v8::Local::<v8::Float32Array>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "magResponse must be a Float32Array");
        return;
    };
    let Ok(phase_response) = v8::Local::<v8::Float32Array>::try_from(arguments.get(2)) else {
        crate::webidl::throw_type_error(scope, "phaseResponse must be a Float32Array");
        return;
    };
    let length = frequency_hz.length();
    if mag_response.length() != length || phase_response.length() != length {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The frequency and response arrays must have equal lengths.".to_owned(),
            "InvalidAccessError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    for index in 0..length {
        let frequency = frequency_hz
            .get_index(scope, index as u32)
            .and_then(|value| value.number_value(scope))
            .unwrap_or(f64::NAN);
        let (magnitude, phase) = response_at(&record, frequency);
        let _ = mag_response.set_index(
            scope,
            index as u32,
            v8::Number::new(scope, magnitude).into(),
        );
        let _ = phase_response.set_index(scope, index as u32, v8::Number::new(scope, phase).into());
    }
}

fn response_at(record: &IirFilterNodeRecord, frequency: f64) -> (f64, f64) {
    if !frequency.is_finite() || frequency < 0.0 || frequency > record.sample_rate / 2.0 {
        return (f64::NAN, f64::NAN);
    }
    let omega = 2.0 * std::f64::consts::PI * frequency / record.sample_rate;
    let (numerator_real, numerator_imaginary) = polynomial(&record.feedforward, omega);
    let (denominator_real, denominator_imaginary) = polynomial(&record.feedback, omega);
    let divisor =
        denominator_real * denominator_real + denominator_imaginary * denominator_imaginary;
    let real =
        (numerator_real * denominator_real + numerator_imaginary * denominator_imaginary) / divisor;
    let imaginary =
        (numerator_imaginary * denominator_real - numerator_real * denominator_imaginary) / divisor;
    (
        (real * real + imaginary * imaginary).sqrt(),
        imaginary.atan2(real),
    )
}

fn polynomial(coefficients: &[f64], omega: f64) -> (f64, f64) {
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for (index, coefficient) in coefficients.iter().copied().enumerate() {
        let angle = omega * index as f64;
        real += coefficient * angle.cos();
        imaginary -= coefficient * angle.sin();
    }
    (real, imaginary)
}
