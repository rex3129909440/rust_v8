use std::collections::HashMap;

#[derive(Clone)]
struct AudioWorkletNodeRecord {
    object: v8::Global<v8::Object>,
    parameters: v8::Global<v8::Object>,
    port: v8::Global<v8::Object>,
    processor: v8::Global<v8::Object>,
    worklet: v8::Global<v8::Object>,
    number_of_inputs: u32,
    output_channel_counts: Vec<u32>,
    active: bool,
    onprocessorerror: Option<v8::Global<v8::Function>>,
}

#[derive(Default)]
pub(crate) struct AudioWorkletNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioWorkletNodeRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioWorkletNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioWorkletNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioWorkletNodeStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioWorkletNode",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "parameters", get_parameters)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "port", get_port)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onprocessorerror",
        get_onprocessorerror,
        set_onprocessorerror,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioWorkletNodeStore>()
        .ok_or_else(|| "AudioWorkletNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "AudioWorkletNode requires a context and processor name",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "Audio context must be an object");
        return;
    };
    let name = crate::webidl::value_to_string(scope, a.get(1));
    if name.is_empty() {
        crate::webidl::throw_type_error(scope, "Processor name cannot be empty");
        return;
    }
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "AudioWorkletNode context is not a BaseAudioContext",
        );
        return;
    }
    let Some(worklet) = super::base_audio_context::audio_worklet(scope, context) else {
        crate::webidl::throw_type_error(scope, "AudioWorklet is unavailable for this context");
        return;
    };
    let Some(definition) = super::worklet::processor_definition(scope, worklet, &name) else {
        crate::webidl::throw_type_error(
            scope,
            &format!("The AudioWorklet processor '{name}' is not registered"),
        );
        return;
    };
    let options = v8::Local::<v8::Object>::try_from(a.get(2)).ok();
    let parameter_data =
        options.and_then(|options| object_property(scope, options, "parameterData"));
    let mut entries = Vec::with_capacity(definition.parameters.len());
    for descriptor in &definition.parameters {
        let parameter = match super::audio_param::create(
            scope,
            context,
            descriptor.default_value,
            descriptor.min_value,
            descriptor.max_value,
        ) {
            Ok(parameter) => parameter,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        let _ = super::audio_param::set_initial_automation_rate(
            scope,
            parameter,
            &descriptor.automation_rate,
        );
        if let Some(parameter_data) = parameter_data
            && let Some(value) = number_property(scope, parameter_data, &descriptor.name)
        {
            let _ = super::audio_param::set_current_value(scope, parameter, value as f32);
        }
        entries.push((descriptor.name.clone(), parameter));
    }
    let parameters = match super::audio_param_map::create(scope, entries) {
        Ok(v) => v,
        Err(e) => {
            crate::webidl::throw_type_error(scope, &e);
            return;
        }
    };
    let (port, peer) = match super::message_port::create_pair(scope) {
        Ok(v) => v,
        Err(e) => {
            crate::webidl::throw_type_error(scope, &e);
            return;
        }
    };
    let processor_options = options
        .and_then(|options| value_property(scope, options, "processorOptions"))
        .unwrap_or_else(|| v8::undefined(scope).into());
    let processor =
        match super::worklet::instantiate_processor(scope, worklet, &name, peer, processor_options)
        {
            Ok(processor) => processor,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
    let number_of_inputs = options
        .and_then(|options| integer_property(scope, options, "numberOfInputs"))
        .unwrap_or(1)
        .min(32);
    let number_of_outputs = options
        .and_then(|options| integer_property(scope, options, "numberOfOutputs"))
        .unwrap_or(1)
        .min(32);
    let output_channel_counts = options
        .and_then(|options| integer_sequence_property(scope, options, "outputChannelCount"))
        .filter(|counts| counts.len() == number_of_outputs as usize)
        .unwrap_or_else(|| vec![1; number_of_outputs as usize]);
    if output_channel_counts
        .iter()
        .any(|count| *count == 0 || *count > 32)
    {
        crate::webidl::throw_type_error(
            scope,
            "outputChannelCount entries must be between 1 and 32",
        );
        return;
    }
    super::audio_node::attach(
        scope,
        a.this(),
        Some(context),
        number_of_inputs,
        number_of_outputs,
    );
    let record = AudioWorkletNodeRecord {
        object: v8::Global::new(scope, a.this()),
        parameters: v8::Global::new(scope, parameters),
        port: v8::Global::new(scope, port),
        processor,
        worklet: v8::Global::new(scope, worklet),
        number_of_inputs,
        output_channel_counts,
        active: true,
        onprocessorerror: None,
    };
    scope
        .get_slot_mut::<AudioWorkletNodeStore>()
        .expect("AudioWorkletNode state")
        .records
        .insert(a.this().get_identity_hash().get(), record);
    r.set(a.this().into())
}

pub(crate) fn run_pending(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let pending: Vec<(
        i32,
        v8::Global<v8::Object>,
        v8::Global<v8::Object>,
        u32,
        Vec<u32>,
        v8::Global<v8::Object>,
    )> = scope
        .get_slot::<AudioWorkletNodeStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| record.active)
                .map(|(id, record)| {
                    (
                        *id,
                        record.worklet.clone(),
                        record.processor.clone(),
                        record.number_of_inputs,
                        record.output_channel_counts.clone(),
                        record.parameters.clone(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let processed = !pending.is_empty();
    for (id, worklet, processor, number_of_inputs, output_channel_counts, parameters) in pending {
        let worklet = v8::Local::new(scope, &worklet);
        let parameters = v8::Local::new(scope, &parameters);
        let parameter_values: Vec<(String, f32, bool)> =
            super::audio_param_map::entry_snapshot(scope, parameters)
                .into_iter()
                .map(|(name, parameter)| {
                    let parameter = v8::Local::new(scope, &parameter);
                    let value = super::audio_param::value_at(scope, parameter, 0.0).unwrap_or(0.0);
                    let a_rate = parameter
                        .get(
                            scope,
                            v8::String::new(scope, "automationRate").unwrap().into(),
                        )
                        .map(|value| crate::webidl::value_to_string(scope, value) == "a-rate")
                        .unwrap_or(true);
                    (name, value, a_rate)
                })
                .collect();
        match super::worklet::process_quantum(
            scope,
            worklet,
            &processor,
            number_of_inputs,
            &output_channel_counts,
            &parameter_values,
        ) {
            Ok(keep_alive) => {
                if let Some(record) = scope
                    .get_slot_mut::<AudioWorkletNodeStore>()
                    .and_then(|store| store.records.get_mut(&id))
                {
                    record.active = keep_alive;
                }
            }
            Err(_) => {
                dispatch_processor_error(scope, id);
                if let Some(record) = scope
                    .get_slot_mut::<AudioWorkletNodeStore>()
                    .and_then(|store| store.records.get_mut(&id))
                {
                    record.active = false;
                }
            }
        }
    }
    processed
}

fn dispatch_processor_error(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    let snapshot = scope
        .get_slot::<AudioWorkletNodeStore>()
        .and_then(|store| store.records.get(&id))
        .cloned();
    let Some(snapshot) = snapshot else {
        return;
    };
    let target = v8::Local::new(scope, &snapshot.object);
    let event = super::event_target::create_event(scope, "processorerror");
    super::event_target::dispatch(scope, target, event);
    if let Some(handler) = snapshot.onprocessorerror {
        let handler = v8::Local::new(scope, &handler);
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
}

fn value_property<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn object_property<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    value_property(scope, object, name)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
}

fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    value_property(scope, object, name)?.number_value(scope)
}

fn integer_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u32> {
    let number = number_property(scope, object, name)?;
    if !number.is_finite() || number < 0.0 {
        return None;
    }
    Some(number.trunc() as u32)
}

fn integer_sequence_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<Vec<u32>> {
    let value = value_property(scope, object, name)?;
    let sequence = v8::Local::<v8::Object>::try_from(value).ok()?;
    let length_key = v8::String::new(scope, "length")?;
    let length = sequence
        .get(scope, length_key.into())?
        .uint32_value(scope)?;
    let mut output = Vec::with_capacity(length as usize);
    for index in 0..length {
        output.push(sequence.get_index(scope, index)?.uint32_value(scope)?);
    }
    Some(output)
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<AudioWorkletNodeRecord> {
    scope
        .get_slot::<AudioWorkletNodeStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_parameters(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.parameters).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_port(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.port).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_onprocessorerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(s, a.this()) {
        Some(v) => match v.onprocessorerror {
            Some(h) => r.set(v8::Local::new(s, &h).into()),
            None => r.set(v8::null(s).into()),
        },
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
fn set_onprocessorerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = v8::Local::<v8::Function>::try_from(a.get(0))
        .ok()
        .map(|h| v8::Global::new(s, h));
    if let Some(v) = s
        .get_slot_mut::<AudioWorkletNodeStore>()
        .and_then(|v| v.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.onprocessorerror = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
