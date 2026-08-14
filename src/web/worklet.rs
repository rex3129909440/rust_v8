use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct AudioParamDescriptor {
    pub(crate) name: String,
    pub(crate) default_value: f32,
    pub(crate) min_value: f32,
    pub(crate) max_value: f32,
    pub(crate) automation_rate: String,
}

#[derive(Clone)]
pub(crate) struct ProcessorDefinition {
    pub(crate) constructor: v8::Global<v8::Function>,
    pub(crate) parameters: Vec<AudioParamDescriptor>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkletKind {
    Audio,
    Paint,
}

#[derive(Clone)]
pub(crate) struct PaintDefinition {
    pub(crate) constructor: v8::Global<v8::Function>,
    pub(crate) input_properties: Vec<String>,
    pub(crate) input_arguments: Vec<String>,
    pub(crate) alpha: bool,
}

#[derive(Clone)]
struct WorkletRecord {
    kind: WorkletKind,
    modules: HashSet<String>,
    compiled_modules: HashMap<String, v8::Global<v8::Module>>,
    module_urls: HashMap<i32, String>,
    current_url: Option<String>,
    context: Option<v8::Global<v8::Context>>,
    global: Option<v8::Global<v8::Object>>,
    processors: HashMap<String, ProcessorDefinition>,
    paint_definitions: HashMap<String, PaintDefinition>,
    sample_rate: f64,
    device_pixel_ratio: f64,
    current_frame: u64,
}

impl WorkletRecord {
    fn audio(sample_rate: f64) -> Self {
        Self {
            kind: WorkletKind::Audio,
            modules: HashSet::new(),
            compiled_modules: HashMap::new(),
            module_urls: HashMap::new(),
            current_url: None,
            context: None,
            global: None,
            processors: HashMap::new(),
            paint_definitions: HashMap::new(),
            sample_rate,
            device_pixel_ratio: 1.0,
            current_frame: 0,
        }
    }

    fn paint(device_pixel_ratio: f64) -> Self {
        Self {
            kind: WorkletKind::Paint,
            modules: HashSet::new(),
            compiled_modules: HashMap::new(),
            module_urls: HashMap::new(),
            current_url: None,
            context: None,
            global: None,
            processors: HashMap::new(),
            paint_definitions: HashMap::new(),
            sample_rate: 0.0,
            device_pixel_ratio,
            current_frame: 0,
        }
    }
}

#[derive(Default)]
pub(crate) struct WorkletStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WorkletRecord>,
    worklets_by_global: HashMap<i32, i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WorkletStore::default());
}

pub(crate) fn enable_native_trace_for_existing_realms(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let realms = scope
        .get_slot::<WorkletStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter_map(|(id, record)| {
                    Some((
                        *id,
                        record.kind,
                        record.context.clone()?,
                        record.global.clone()?,
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for (id, kind, context, global) in realms {
        let context = v8::Local::new(scope, &context);
        let child_scope = &mut v8::ContextScope::new(scope, context);
        let global = v8::Local::new(child_scope, &global);
        crate::trace::relabel_json_intrinsic_trace(child_scope, &worklet_trace_label(kind, id))?;
        crate::trace::label_native_value(
            child_scope,
            global.into(),
            &worklet_trace_label(kind, id),
        );
    }
    Ok(())
}

pub(crate) fn disable_native_trace_for_existing_realms(_: &mut v8::OwnedIsolate) {}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Worklet", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Worklet",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "addModule", 1, add_module)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WorkletStore>()
        .ok_or_else(|| "Worklet state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Worklet': Illegal constructor");
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    sample_rate: f64,
) {
    scope
        .get_slot_mut::<WorkletStore>()
        .expect("Worklet state")
        .records
        .insert(
            object.get_identity_hash().get(),
            WorkletRecord::audio(sample_rate),
        );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Worklet".to_owned());
    }
    let device_pixel_ratio = crate::fingerprint::edge(scope).screen.device_pixel_ratio;
    scope
        .get_slot_mut::<WorkletStore>()
        .ok_or_else(|| "Worklet state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            WorkletRecord::paint(device_pixel_ratio),
        );
    Ok(object)
}

pub(crate) fn current_worklet_id(scope: &mut v8::PinScope<'_, '_>) -> Option<i32> {
    let global_id = scope
        .get_current_context()
        .global(scope)
        .get_identity_hash()
        .get();
    scope
        .get_slot::<WorkletStore>()?
        .worklets_by_global
        .get(&global_id)
        .copied()
}

pub(crate) fn current_sample_rate(scope: &mut v8::PinScope<'_, '_>) -> Option<f64> {
    let worklet_id = current_worklet_id(scope)?;
    scope
        .get_slot::<WorkletStore>()?
        .records
        .get(&worklet_id)
        .map(|record| record.sample_rate)
}

pub(crate) fn current_device_pixel_ratio(scope: &mut v8::PinScope<'_, '_>) -> Option<f64> {
    let worklet_id = current_worklet_id(scope)?;
    scope
        .get_slot::<WorkletStore>()?
        .records
        .get(&worklet_id)
        .filter(|record| record.kind == WorkletKind::Paint)
        .map(|record| record.device_pixel_ratio)
}

pub(crate) fn current_frame(scope: &mut v8::PinScope<'_, '_>) -> Option<u64> {
    let worklet_id = current_worklet_id(scope)?;
    scope
        .get_slot::<WorkletStore>()?
        .records
        .get(&worklet_id)
        .map(|record| record.current_frame)
}

pub(crate) fn register_processor(
    scope: &mut v8::PinScope<'_, '_>,
    name: String,
    constructor: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Processor name cannot be empty".to_owned());
    }
    let worklet_id = current_worklet_id(scope)
        .ok_or_else(|| "registerProcessor called outside a Worklet".to_owned())?;
    if scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .is_none_or(|record| record.kind != WorkletKind::Audio)
    {
        return Err("registerProcessor is only available in an AudioWorklet".to_owned());
    }
    if scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .is_some_and(|record| record.processors.contains_key(&name))
    {
        return Err(format!("A processor named '{name}' is already registered"));
    }
    let parameters = read_parameter_descriptors(scope, constructor)?;
    let definition = ProcessorDefinition {
        constructor: v8::Global::new(scope, constructor),
        parameters,
    };
    scope
        .get_slot_mut::<WorkletStore>()
        .and_then(|store| store.records.get_mut(&worklet_id))
        .ok_or_else(|| "Worklet state disappeared".to_owned())?
        .processors
        .insert(name, definition);
    Ok(())
}

pub(crate) fn register_paint(
    scope: &mut v8::PinScope<'_, '_>,
    name: String,
    constructor: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Paint worklet name cannot be empty".to_owned());
    }
    let worklet_id = current_worklet_id(scope)
        .ok_or_else(|| "registerPaint called outside a Worklet".to_owned())?;
    let record = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .ok_or_else(|| "PaintWorklet is unavailable".to_owned())?;
    if record.kind != WorkletKind::Paint {
        return Err("registerPaint is only available in a PaintWorklet".to_owned());
    }
    if record.paint_definitions.contains_key(&name) {
        return Err(format!(
            "A paint worklet named '{name}' is already registered"
        ));
    }
    let prototype_key = v8::String::new(scope, "prototype")
        .ok_or_else(|| "cannot create prototype key".to_owned())?;
    let prototype = constructor
        .get(scope, prototype_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "Paint worklet constructor must have a prototype".to_owned())?;
    let paint_key =
        v8::String::new(scope, "paint").ok_or_else(|| "cannot create paint key".to_owned())?;
    if prototype
        .get(scope, paint_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .is_none()
    {
        return Err("Paint worklet prototype.paint must be callable".to_owned());
    }
    let input_properties = read_string_array_static(scope, constructor, "inputProperties")?;
    let input_arguments = read_string_array_static(scope, constructor, "inputArguments")?;
    let alpha = read_paint_alpha(scope, constructor)?;
    let definition = PaintDefinition {
        constructor: v8::Global::new(scope, constructor),
        input_properties,
        input_arguments,
        alpha,
    };
    scope
        .get_slot_mut::<WorkletStore>()
        .and_then(|store| store.records.get_mut(&worklet_id))
        .ok_or_else(|| "PaintWorklet state disappeared".to_owned())?
        .paint_definitions
        .insert(name, definition);
    Ok(())
}

fn read_string_array_static(
    scope: &mut v8::PinScope<'_, '_>,
    constructor: v8::Local<'_, v8::Function>,
    name: &str,
) -> Result<Vec<String>, String> {
    let key = v8::String::new(scope, name).ok_or_else(|| format!("cannot create {name} key"))?;
    let Some(value) = constructor.get(scope, key.into()) else {
        return Ok(Vec::new());
    };
    if value.is_undefined() {
        return Ok(Vec::new());
    }
    let array = v8::Local::<v8::Array>::try_from(value)
        .map_err(|_| format!("{name} must return an array"))?;
    let mut output = Vec::with_capacity(array.length() as usize);
    for index in 0..array.length() {
        let value = array
            .get_index(scope, index)
            .ok_or_else(|| format!("cannot read {name} entry"))?;
        output.push(crate::webidl::value_to_string(scope, value));
    }
    Ok(output)
}

fn read_paint_alpha(
    scope: &mut v8::PinScope<'_, '_>,
    constructor: v8::Local<'_, v8::Function>,
) -> Result<bool, String> {
    let key = v8::String::new(scope, "contextOptions")
        .ok_or_else(|| "cannot create contextOptions key".to_owned())?;
    let Some(value) = constructor.get(scope, key.into()) else {
        return Ok(true);
    };
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        return Ok(true);
    };
    let alpha_key =
        v8::String::new(scope, "alpha").ok_or_else(|| "cannot create alpha key".to_owned())?;
    Ok(options
        .get(scope, alpha_key.into())
        .filter(|value| !value.is_undefined())
        .is_none_or(|value| value.boolean_value(scope)))
}

pub(crate) fn processor_definition(
    scope: &v8::PinScope<'_, '_>,
    worklet: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<ProcessorDefinition> {
    scope
        .get_slot::<WorkletStore>()?
        .records
        .get(&worklet.get_identity_hash().get())?
        .processors
        .get(name)
        .cloned()
}

pub(crate) fn instantiate_processor(
    scope: &mut v8::PinScope<'_, '_>,
    worklet: v8::Local<'_, v8::Object>,
    name: &str,
    port: v8::Local<'_, v8::Object>,
    processor_options: v8::Local<'_, v8::Value>,
) -> Result<v8::Global<v8::Object>, String> {
    let worklet_id = worklet.get_identity_hash().get();
    ensure_realm(scope, worklet_id)?;
    let record = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .cloned()
        .ok_or_else(|| "AudioWorklet is unavailable".to_owned())?;
    let definition = record
        .processors
        .get(name)
        .cloned()
        .ok_or_else(|| format!("The AudioWorklet processor '{name}' is not registered"))?;
    let context = v8::Local::new(
        scope,
        record
            .context
            .as_ref()
            .ok_or_else(|| "AudioWorklet realm is unavailable".to_owned())?,
    );
    let child_scope = &mut v8::ContextScope::new(scope, context);
    let constructor = v8::Local::new(child_scope, &definition.constructor);
    let port = v8::Local::new(child_scope, &v8::Global::new(child_scope, port));
    super::audio_worklet_processor::set_pending_port(child_scope, port);
    let processor_options = v8::Local::new(
        child_scope,
        &v8::Global::new(child_scope, processor_options),
    );
    let instance = constructor.new_instance(child_scope, &[processor_options]);
    if instance.is_none() {
        super::audio_worklet_processor::clear_pending_port(child_scope);
    }
    let instance =
        instance.ok_or_else(|| format!("Cannot construct AudioWorklet processor '{name}'"))?;
    if !super::audio_worklet_processor::is_instance(child_scope, instance) {
        return Err(format!(
            "AudioWorklet processor '{name}' must extend AudioWorkletProcessor"
        ));
    }
    Ok(v8::Global::new(child_scope, instance))
}

pub(crate) fn process_quantum(
    scope: &mut v8::PinScope<'_, '_>,
    worklet: v8::Local<'_, v8::Object>,
    processor: &v8::Global<v8::Object>,
    number_of_inputs: u32,
    output_channel_counts: &[u32],
    parameter_values: &[(String, f32, bool)],
) -> Result<bool, String> {
    let worklet_id = worklet.get_identity_hash().get();
    let record = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .cloned()
        .ok_or_else(|| "AudioWorklet is unavailable".to_owned())?;
    let context = v8::Local::new(
        scope,
        record
            .context
            .as_ref()
            .ok_or_else(|| "AudioWorklet realm is unavailable".to_owned())?,
    );
    let child_scope = &mut v8::ContextScope::new(scope, context);
    let processor = v8::Local::new(child_scope, processor);
    let process_key = v8::String::new(child_scope, "process")
        .ok_or_else(|| "cannot create process key".to_owned())?;
    let process = processor
        .get(child_scope, process_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "AudioWorkletProcessor.process is not callable".to_owned())?;
    let inputs = v8::Array::new(child_scope, number_of_inputs as i32);
    for input_index in 0..number_of_inputs {
        let channels = v8::Array::new(child_scope, 1);
        let samples = quantum_array(child_scope)?;
        let _ = channels.set_index(child_scope, 0, samples.into());
        let _ = inputs.set_index(child_scope, input_index, channels.into());
    }
    let outputs = v8::Array::new(child_scope, output_channel_counts.len() as i32);
    for (output_index, channel_count) in output_channel_counts.iter().enumerate() {
        let channels = v8::Array::new(child_scope, *channel_count as i32);
        for channel_index in 0..*channel_count {
            let samples = quantum_array(child_scope)?;
            let _ = channels.set_index(child_scope, channel_index, samples.into());
        }
        let _ = outputs.set_index(child_scope, output_index as u32, channels.into());
    }
    let parameters = v8::Object::new(child_scope);
    for (name, value, a_rate) in parameter_values {
        let length = if *a_rate { 128 } else { 1 };
        let bytes = vec![0_u8; length * std::mem::size_of::<f32>()];
        let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
        let buffer = v8::ArrayBuffer::with_backing_store(child_scope, &backing);
        let array = v8::Float32Array::new(child_scope, buffer, 0, length)
            .ok_or_else(|| "cannot create AudioWorklet parameter array".to_owned())?;
        for index in 0..length as u32 {
            let _ = array.set_index(
                child_scope,
                index,
                v8::Number::new(child_scope, f64::from(*value)).into(),
            );
        }
        let key = v8::String::new(child_scope, name)
            .ok_or_else(|| "cannot create AudioWorklet parameter name".to_owned())?;
        let _ = parameters.create_data_property(child_scope, key.into(), array.into());
    }
    let returned = process
        .call(
            child_scope,
            processor.into(),
            &[inputs.into(), outputs.into(), parameters.into()],
        )
        .ok_or_else(|| "AudioWorkletProcessor.process threw an exception".to_owned())?;
    let keep_alive = returned.boolean_value(child_scope);
    if let Some(record) = child_scope
        .get_slot_mut::<WorkletStore>()
        .and_then(|store| store.records.get_mut(&worklet_id))
    {
        record.current_frame = record.current_frame.saturating_add(128);
    }
    Ok(keep_alive)
}

fn quantum_array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Float32Array>, String> {
    let bytes = vec![0_u8; 128 * std::mem::size_of::<f32>()];
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    v8::Float32Array::new(scope, buffer, 0, 128)
        .ok_or_else(|| "cannot create AudioWorklet render quantum".to_owned())
}

fn read_parameter_descriptors(
    scope: &mut v8::PinScope<'_, '_>,
    constructor: v8::Local<'_, v8::Function>,
) -> Result<Vec<AudioParamDescriptor>, String> {
    let key = v8::String::new(scope, "parameterDescriptors")
        .ok_or_else(|| "cannot create parameterDescriptors key".to_owned())?;
    let Some(value) = constructor.get(scope, key.into()) else {
        return Ok(Vec::new());
    };
    if value.is_undefined() {
        return Ok(Vec::new());
    }
    let array = v8::Local::<v8::Array>::try_from(value)
        .map_err(|_| "parameterDescriptors must return an array".to_owned())?;
    let mut parameters = Vec::with_capacity(array.length() as usize);
    for index in 0..array.length() {
        let value = array
            .get_index(scope, index)
            .ok_or_else(|| "Cannot read AudioParam descriptor".to_owned())?;
        let descriptor = v8::Local::<v8::Object>::try_from(value)
            .map_err(|_| "Each AudioParam descriptor must be an object".to_owned())?;
        let name = string_property(scope, descriptor, "name").unwrap_or_default();
        if name.is_empty() {
            return Err("AudioParam descriptor name cannot be empty".to_owned());
        }
        if parameters
            .iter()
            .any(|parameter: &AudioParamDescriptor| parameter.name == name)
        {
            return Err(format!("Duplicate AudioParam descriptor '{name}'"));
        }
        let default_value = number_property(scope, descriptor, "defaultValue").unwrap_or(0.0);
        let min_value = number_property(scope, descriptor, "minValue").unwrap_or(-3.402_823_5e38);
        let max_value = number_property(scope, descriptor, "maxValue").unwrap_or(3.402_823_5e38);
        if !default_value.is_finite()
            || !min_value.is_finite()
            || !max_value.is_finite()
            || min_value > max_value
            || default_value < min_value
            || default_value > max_value
        {
            return Err(format!("Invalid AudioParam descriptor '{name}'"));
        }
        let automation_rate = string_property(scope, descriptor, "automationRate")
            .unwrap_or_else(|| "a-rate".to_owned());
        if automation_rate != "a-rate" && automation_rate != "k-rate" {
            return Err(format!("Invalid automationRate for AudioParam '{name}'"));
        }
        parameters.push(AudioParamDescriptor {
            name,
            default_value,
            min_value,
            max_value,
            automation_rate,
        });
    }
    parameters.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(parameters)
}

fn string_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        return None;
    }
    Some(crate::webidl::value_to_string(scope, value))
}

fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f32> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        return None;
    }
    value.number_value(scope).map(|value| value as f32)
}

fn ensure_realm(scope: &mut v8::PinScope<'_, '_>, worklet_id: i32) -> Result<(), String> {
    if scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .is_some_and(|record| record.context.is_some())
    {
        return Ok(());
    }
    let sample_rate = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .map(|record| record.sample_rate)
        .ok_or_else(|| "Illegal invocation".to_owned())?;
    let context = v8::Context::new(scope, Default::default());
    let global = context.global(scope);
    let context_global = v8::Global::new(scope, context);
    let global_global = v8::Global::new(scope, global);
    {
        let store = scope
            .get_slot_mut::<WorkletStore>()
            .ok_or_else(|| "Worklet state was not prepared".to_owned())?;
        let record = store
            .records
            .get_mut(&worklet_id)
            .ok_or_else(|| "Illegal invocation".to_owned())?;
        record.context = Some(context_global);
        record.global = Some(global_global);
        record.sample_rate = sample_rate;
        store
            .worklets_by_global
            .insert(global.get_identity_hash().get(), worklet_id);
    }
    let kind = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .map(|record| record.kind)
        .ok_or_else(|| "Illegal invocation".to_owned())?;
    {
        let child_scope = &mut v8::ContextScope::new(scope, context);
        let trace_label = worklet_trace_label(kind, worklet_id);
        crate::trace::install_json_intrinsic_trace(child_scope, &trace_label)?;
        let prototype = match kind {
            WorkletKind::Audio => super::audio_worklet_global_scope::install(child_scope)?,
            WorkletKind::Paint => super::paint_worklet_global_scope::install(child_scope)?,
        };
        if crate::webidl::set_platform_prototype(child_scope, global, prototype.into())
            != Some(true)
        {
            return Err("cannot attach WorkletGlobalScope prototype".to_owned());
        }
        if kind == WorkletKind::Audio {
            super::audio_worklet_processor::install(child_scope)?;
        }
        crate::locale_runtime::install(child_scope)?;
        crate::determinism::install(child_scope)?;
        if crate::trace::is_enabled(child_scope) {
            crate::trace::label_native_value(child_scope, global.into(), &trace_label);
        }
    }
    Ok(())
}

fn worklet_trace_label(kind: WorkletKind, worklet_id: i32) -> String {
    match kind {
        WorkletKind::Audio => format!("audioWorklet[{worklet_id}]"),
        WorkletKind::Paint => format!("paintWorklet[{worklet_id}]"),
    }
}

fn add_module(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let worklet_id = arguments.this().get_identity_hash().get();
    if !scope
        .get_slot::<WorkletStore>()
        .is_some_and(|store| store.records.contains_key(&worklet_id))
    {
        crate::webidl::reject_illegal_invocation_promise(scope, "Worklet", "addModule", result);
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let script = match super::worker_script_source::load(scope, &input, None) {
        Ok(script) => script,
        Err(message) => {
            return_rejected_promise(scope, &mut result, &message);
            return;
        }
    };
    if scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .is_some_and(|record| record.modules.contains(&script.url))
    {
        return_resolved_promise(scope, &mut result);
        return;
    }
    if let Err(message) = ensure_realm(scope, worklet_id).and_then(|_| {
        if let Some(record) = scope
            .get_slot_mut::<WorkletStore>()
            .and_then(|store| store.records.get_mut(&worklet_id))
        {
            record.current_url = Some(script.url.clone());
        }
        evaluate_module(scope, worklet_id, &script.url, &script.source)
    }) {
        return_rejected_promise(scope, &mut result, &message);
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<WorkletStore>()
        .and_then(|store| store.records.get_mut(&worklet_id))
    {
        record.modules.insert(script.url);
    }
    return_resolved_promise(scope, &mut result);
}

fn evaluate_module(
    scope: &mut v8::PinScope<'_, '_>,
    worklet_id: i32,
    url: &str,
    source: &str,
) -> Result<(), String> {
    let record = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .cloned()
        .ok_or_else(|| "AudioWorklet is unavailable".to_owned())?;
    let context = v8::Local::new(
        scope,
        record
            .context
            .as_ref()
            .ok_or_else(|| "AudioWorklet realm is unavailable".to_owned())?,
    );
    let child_scope = &mut v8::ContextScope::new(scope, context);
    v8::tc_scope!(let try_catch, child_scope);
    let _user_execution = crate::trace::enter_user_execution(try_catch);
    let source = v8::String::new(try_catch, source)
        .ok_or_else(|| "AudioWorklet module exceeds V8 limits".to_owned())?;
    let evaluated = compile_module(try_catch, worklet_id, url, source).and_then(|module| {
        module
            .instantiate_module(try_catch, resolve_module)
            .and_then(|instantiated| instantiated.then(|| module))
            .and_then(|module| module.evaluate(try_catch))
    });
    if evaluated.is_none() {
        let message = try_catch
            .exception()
            .and_then(|exception| exception.to_string(try_catch))
            .map(|text| text.to_rust_string_lossy(try_catch))
            .unwrap_or_else(|| "AudioWorklet module execution failed".to_owned());
        return Err(message);
    }
    try_catch.perform_microtask_checkpoint();
    Ok(())
}

fn compile_module<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    worklet_id: i32,
    url: &str,
    source: v8::Local<'s, v8::String>,
) -> Option<v8::Local<'s, v8::Module>> {
    let resource_name = v8::String::new(scope, url)?;
    let origin = v8::ScriptOrigin::new(
        scope,
        resource_name.into(),
        0,
        0,
        false,
        -1,
        None,
        false,
        false,
        true,
        None,
    );
    let mut source = v8::script_compiler::Source::new(source, Some(&origin));
    let module = v8::script_compiler::compile_module(scope, &mut source)?;
    let saved_module = v8::Global::new(scope, module);
    if let Some(record) = scope
        .get_slot_mut::<WorkletStore>()
        .and_then(|store| store.records.get_mut(&worklet_id))
    {
        if let Some(script_id) = module.script_id() {
            record.module_urls.insert(script_id, url.to_owned());
        }
        record.compiled_modules.insert(url.to_owned(), saved_module);
    }
    Some(module)
}

fn resolve_module<'s>(
    context: v8::Local<'s, v8::Context>,
    specifier: v8::Local<'s, v8::String>,
    _import_attributes: v8::Local<'s, v8::FixedArray>,
    referrer: v8::Local<'s, v8::Module>,
) -> Option<v8::Local<'s, v8::Module>> {
    v8::callback_scope!(unsafe scope, context);
    let worklet_id = current_worklet_id(scope)?;
    let base = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .and_then(|record| {
            referrer
                .script_id()
                .and_then(|script_id| record.module_urls.get(&script_id).cloned())
        })?;
    let input = specifier.to_rust_string_lossy(scope);
    let script = match super::worker_script_source::load(scope, &input, Some(&base)) {
        Ok(script) => script,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    if let Some(module) = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .and_then(|record| record.compiled_modules.get(&script.url))
        .cloned()
    {
        return Some(v8::Local::new(scope, &module));
    }
    let source = v8::String::new(scope, &script.source)?;
    compile_module(scope, worklet_id, &script.url, source)
}

pub(crate) fn dynamic_import<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    specifier: v8::Local<'s, v8::String>,
) -> Option<v8::Local<'s, v8::Promise>> {
    let worklet_id = current_worklet_id(scope)?;
    let base = scope
        .get_slot::<WorkletStore>()
        .and_then(|store| store.records.get(&worklet_id))
        .and_then(|record| record.current_url.clone())
        .unwrap_or_else(|| "https://sandbox.test/".to_owned());
    let input = specifier.to_rust_string_lossy(scope);
    let script = match super::worker_script_source::load(scope, &input, Some(&base)) {
        Ok(script) => script,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    let source = v8::String::new(scope, &script.source)?;
    let module = compile_module(scope, worklet_id, &script.url, source)?;
    if module.instantiate_module(scope, resolve_module) != Some(true) {
        return None;
    }
    module.evaluate(scope)?;
    let resolver = v8::PromiseResolver::new(scope)?;
    if resolver.resolve(scope, module.get_module_namespace()) != Some(true) {
        return None;
    }
    Some(resolver.get_promise(scope))
}

fn return_resolved_promise(scope: &mut v8::PinScope<'_, '_>, result: &mut v8::ReturnValue<'_>) {
    let undefined = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
        result.set(promise.into());
    }
}

fn return_rejected_promise(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    message: &str,
) {
    let message = v8::String::new(scope, message).expect("AudioWorklet error");
    let error = v8::Exception::error(scope, message);
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, error) {
        result.set(promise.into());
    }
}
