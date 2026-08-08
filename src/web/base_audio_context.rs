use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct BaseAudioContextStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BaseAudioContextRecord>,
}

#[derive(Clone)]
struct BaseAudioContextRecord {
    destination: v8::Global<v8::Object>,
    listener: v8::Global<v8::Object>,
    audio_worklet: v8::Global<v8::Object>,
    sample_rate: f64,
    current_frame: u64,
    running_started_ms: Option<f64>,
    offline: bool,
    state: String,
    onstatechange: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BaseAudioContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BaseAudioContext", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<BaseAudioContextStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BaseAudioContext",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "destination", get_destination)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sampleRate", get_sample_rate)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "currentTime", get_current_time)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "listener", get_listener)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onstatechange",
        get_onstatechange,
        set_onstatechange,
    )?;
    crate::webidl::define_method(scope, prototype, "createAnalyser", 0, create_analyser)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createBiquadFilter",
        0,
        create_biquad_filter,
    )?;
    crate::webidl::define_method(scope, prototype, "createBuffer", 3, create_buffer)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createBufferSource",
        0,
        create_buffer_source,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createChannelMerger",
        0,
        create_channel_merger,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createChannelSplitter",
        0,
        create_channel_splitter,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createConstantSource",
        0,
        create_constant_source,
    )?;
    crate::webidl::define_method(scope, prototype, "createConvolver", 0, create_convolver)?;
    crate::webidl::define_method(scope, prototype, "createDelay", 0, create_delay)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createDynamicsCompressor",
        0,
        create_dynamics_compressor,
    )?;
    crate::webidl::define_method(scope, prototype, "createGain", 0, create_gain)?;
    crate::webidl::define_method(scope, prototype, "createIIRFilter", 2, create_iir_filter)?;
    crate::webidl::define_method(scope, prototype, "createOscillator", 0, create_oscillator)?;
    crate::webidl::define_method(scope, prototype, "createPanner", 0, create_panner)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createPeriodicWave",
        2,
        create_periodic_wave,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createScriptProcessor",
        0,
        create_script_processor,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createStereoPanner",
        0,
        create_stereo_panner,
    )?;
    crate::webidl::define_method(scope, prototype, "createWaveShaper", 0, create_wave_shaper)?;
    crate::webidl::define_method(scope, prototype, "decodeAudioData", 1, decode_audio_data)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "audioWorklet", get_audio_worklet)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BaseAudioContextStore>()
        .ok_or_else(|| "BaseAudioContext state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'BaseAudioContext': Illegal constructor",
    );
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    sample_rate: f64,
    state: &str,
    offline: bool,
) -> Result<(), String> {
    super::event_target::attach(scope, object);
    let max_channel_count = crate::fingerprint::edge(scope)
        .rendering
        .audio
        .max_channel_count;
    let destination = super::audio_destination_node::create(scope, object, max_channel_count)?;
    let listener = super::audio_listener::create(scope, object)?;
    let audio_worklet = super::audio_worklet::create(scope, sample_rate)?;
    super::periodic_wave::register_context(scope, object);
    super::panner_node::register_context(scope, object);
    let destination = v8::Global::new(scope, destination);
    let listener = v8::Global::new(scope, listener);
    let audio_worklet = v8::Global::new(scope, audio_worklet);
    let running_started_ms =
        (!offline && state == "running").then(|| crate::determinism::elapsed_milliseconds(scope));
    scope
        .get_slot_mut::<BaseAudioContextStore>()
        .ok_or_else(|| "BaseAudioContext state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            BaseAudioContextRecord {
                destination,
                listener,
                audio_worklet,
                sample_rate,
                current_frame: 0,
                running_started_ms,
                offline,
                state: state.to_owned(),
                onstatechange: None,
            },
        );
    if state == "running" {
        queue_state_change(scope, object);
    }
    Ok(())
}

pub(crate) fn is_context(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<BaseAudioContextStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

pub(crate) fn sample_rate(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<f64> {
    scope
        .get_slot::<BaseAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .map(|record| record.sample_rate)
}

pub(crate) fn audio_worklet<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let worklet = scope
        .get_slot::<BaseAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())?
        .audio_worklet
        .clone();
    Some(v8::Local::new(scope, &worklet))
}

pub(crate) fn destination<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let destination = scope
        .get_slot::<BaseAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())?
        .destination
        .clone();
    Some(v8::Local::new(scope, &destination))
}

pub(crate) fn listener<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let listener = scope
        .get_slot::<BaseAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())?
        .listener
        .clone();
    Some(v8::Local::new(scope, &listener))
}

pub(crate) fn set_state(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    state: &str,
) -> bool {
    let now_ms = crate::determinism::elapsed_milliseconds(scope);
    let mut changed = false;
    if let Some(record) = scope
        .get_slot_mut::<BaseAudioContextStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        if record.state == state {
            return false;
        }
        if !record.offline && record.state == "running" {
            record.current_frame = current_frame_at(record, now_ms);
            record.running_started_ms = None;
        }
        if !record.offline && state == "running" {
            record.running_started_ms = Some(now_ms);
        }
        record.state = state.to_owned();
        changed = true;
    }
    if changed {
        queue_state_change(scope, object);
    }
    changed
}

pub(crate) fn state(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    scope
        .get_slot::<BaseAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .map(|record| record.state.clone())
}

pub(crate) fn current_time(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<f64> {
    let record = scope
        .get_slot::<BaseAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())?;
    let frame = current_frame_at(record, crate::determinism::elapsed_milliseconds(scope));
    Some(frame as f64 / record.sample_rate)
}

pub(crate) fn advance_offline_to_frame(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    frame: u64,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<BaseAudioContextStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    if !record.offline {
        return false;
    }
    record.current_frame = record.current_frame.max(frame);
    true
}

fn current_frame_at(record: &BaseAudioContextRecord, now_ms: f64) -> u64 {
    if record.offline || record.state != "running" {
        return record.current_frame;
    }
    let Some(started_ms) = record.running_started_ms else {
        return record.current_frame;
    };
    let elapsed_seconds = ((now_ms - started_ms).max(0.0)) / 1_000.0;
    let elapsed_frames = (elapsed_seconds * record.sample_rate).floor() as u64;
    let quantum_frames = elapsed_frames / 128 * 128;
    record.current_frame.saturating_add(quantum_frames)
}

fn queue_state_change(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let task = v8::Function::builder(dispatch_state_change)
        .data(object.into())
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope);
    if let Some(task) = task {
        scope.enqueue_microtask(task);
    }
}

fn dispatch_state_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.data()) else {
        return;
    };
    let Some(_) = record(scope, context) else {
        return;
    };
    let Ok(event) = super::event::create(scope, "statechange") else {
        return;
    };
    super::event_target::dispatch(scope, context, event);
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BaseAudioContextRecord> {
    scope
        .get_slot::<BaseAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&BaseAudioContextRecord) -> v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_destination(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |record| record.destination.clone());
}
fn get_listener(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |record| record.listener.clone());
}
fn get_audio_worklet(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |record| record.audio_worklet.clone());
}
fn get_sample_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.sample_rate).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(current_time) = current_time(scope, arguments.this()) {
        result.set(v8::Number::new(scope, current_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.state) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_onstatechange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.onstatechange {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
fn set_onstatechange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    let present = value.is_some();
    if let Some(record) = scope
        .get_slot_mut::<BaseAudioContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.onstatechange = value;
        super::event_target::set_attribute_handler(scope, arguments.this(), "statechange", present);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "statechange" {
        return;
    }
    let handler = scope
        .get_slot::<BaseAudioContextStore>()
        .and_then(|store| store.records.get(&target.get_identity_hash().get()))
        .and_then(|record| record.onstatechange.clone());
    let Some(handler) = handler else {
        return;
    };
    let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) else {
        return;
    };
    let _ = handler.call(scope, target.into(), &[event.into()]);
}

fn context_or_throw(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    if is_context(scope, object) {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

fn create_analyser(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::analyser_node::create(scope, context, None) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_biquad_filter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::biquad_filter_node::create(scope, context) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_buffer_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::audio_buffer_source_node::create(scope, context, None) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_channel_merger(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    let number_of_inputs = if arguments.get(0).is_undefined() {
        6
    } else {
        arguments.get(0).uint32_value(scope).unwrap_or(0)
    };
    match super::channel_merger_node::create(scope, context, number_of_inputs) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_channel_splitter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    let number_of_outputs = if arguments.get(0).is_undefined() {
        6
    } else {
        arguments.get(0).uint32_value(scope).unwrap_or(0)
    };
    match super::channel_splitter_node::create(scope, context, number_of_outputs) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_constant_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::constant_source_node::create(scope, context) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_convolver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::convolver_node::create(scope, context) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_delay(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    let max_delay_time = if arguments.get(0).is_undefined() {
        1.0
    } else {
        arguments.get(0).number_value(scope).unwrap_or(f64::NAN)
    };
    match super::delay_node::create(scope, context, max_delay_time) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_dynamics_compressor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::dynamics_compressor_node::create(scope, context) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_gain(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::gain_node::create(scope, context) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_iir_filter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    let feedforward = sequence(scope, arguments.get(0))
        .into_iter()
        .map(f64::from)
        .collect();
    let feedback = sequence(scope, arguments.get(1))
        .into_iter()
        .map(f64::from)
        .collect();
    match super::iir_filter_node::create(scope, context, feedforward, feedback) {
        Ok(node) => result.set(node.into()),
        Err(error) if error.name == "TypeError" => {
            crate::webidl::throw_type_error(scope, &error.message)
        }
        Err(error) => {
            if let Ok(exception) =
                super::dom_exception::create(scope, error.message, error.name.to_owned())
            {
                scope.throw_exception(exception.into());
            }
        }
    }
}
fn create_stereo_panner(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::stereo_panner_node::create(scope, context) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn create_wave_shaper(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::wave_shaper_node::create(scope, context) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !context_or_throw(scope, arguments.this()) {
        return;
    }
    let channels = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let length = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let sample_rate = arguments.get(2).number_value(scope).unwrap_or(0.0);
    if !(1..=32).contains(&channels)
        || length == 0
        || !sample_rate.is_finite()
        || !(3_000.0..=768_000.0).contains(&sample_rate)
    {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The AudioBuffer dimensions are outside the supported range".to_owned(),
            "NotSupportedError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    match super::audio_buffer::create(scope, channels, length, sample_rate) {
        Ok(buffer) => result.set(buffer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_oscillator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::oscillator_node::create(scope, context, None) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_panner(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    match super::panner_node::create(scope, context, None) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn sequence(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Vec<f32> {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return Vec::new();
    };
    let Some(length_key) = v8::String::new(scope, "length") else {
        return Vec::new();
    };
    let length = object
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let mut output = Vec::with_capacity(length as usize);
    for index in 0..length {
        output.push(
            object
                .get_index(scope, index)
                .and_then(|value| value.number_value(scope))
                .unwrap_or(0.0) as f32,
        );
    }
    output
}

fn create_periodic_wave(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !context_or_throw(scope, arguments.this()) {
        return;
    }
    let real = sequence(scope, arguments.get(0));
    let imaginary = sequence(scope, arguments.get(1));
    let disable = v8::Local::<v8::Object>::try_from(arguments.get(2))
        .ok()
        .map(|options| super::event::boolean_property(scope, options, "disableNormalization"))
        .unwrap_or(false);
    match super::periodic_wave::create(scope, real, imaginary, disable) {
        Ok(wave) => result.set(wave.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_script_processor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    let buffer_size = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let inputs = arguments.get(1).uint32_value(scope).unwrap_or(2);
    let outputs = arguments.get(2).uint32_value(scope).unwrap_or(2);
    match super::script_processor_node::create(scope, context, buffer_size, inputs, outputs) {
        Ok(node) => result.set(node.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn decode_audio_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let context = arguments.this();
    if !context_or_throw(scope, context) {
        return;
    }
    let sample_rate = record(scope, context)
        .map(|record| record.sample_rate)
        .unwrap_or_else(|| crate::fingerprint::edge(scope).rendering.audio.sample_rate);
    let Ok(input) = v8::Local::<v8::ArrayBuffer>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'decodeAudioData' on 'BaseAudioContext': parameter 1 is not of type 'ArrayBuffer'.",
        );
        return;
    };
    let backing = input.get_backing_store();
    let Some(data) = backing.data() else {
        reject_audio_decode(scope, arguments, result, "Unable to decode audio data");
        return;
    };
    let bytes =
        unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), backing.byte_length()) };
    let Some(decoded) = decode_wave(bytes, sample_rate) else {
        reject_audio_decode(scope, arguments, result, "Unable to decode audio data");
        return;
    };
    let buffer = match super::audio_buffer::create(
        scope,
        decoded.channels.len() as u32,
        decoded.length,
        sample_rate,
    ) {
        Ok(buffer) => buffer,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    for (channel_index, channel) in decoded.channels.iter().enumerate() {
        for (sample_index, sample) in channel.iter().enumerate() {
            super::audio_buffer::set_sample(
                scope,
                buffer,
                channel_index as u32,
                sample_index as u32,
                *sample,
            );
        }
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, buffer.into()) {
        attach_decode_callback(scope, promise, arguments.get(1));
        result.set(promise.into());
    }
}

struct DecodedWave {
    channels: Vec<Vec<f32>>,
    length: u32,
}

fn decode_wave(bytes: &[u8], target_sample_rate: f64) -> Option<DecodedWave> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let mut format = None;
    let mut audio_data = None;
    let mut offset = 12_usize;
    while offset.checked_add(8)? <= bytes.len() {
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_length =
            u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?) as usize;
        let start = offset + 8;
        let end = start.checked_add(chunk_length)?.min(bytes.len());
        if chunk_id == b"fmt " && end >= start + 16 {
            format = Some((
                u16::from_le_bytes(bytes[start..start + 2].try_into().ok()?),
                u16::from_le_bytes(bytes[start + 2..start + 4].try_into().ok()?),
                u32::from_le_bytes(bytes[start + 4..start + 8].try_into().ok()?),
                u16::from_le_bytes(bytes[start + 12..start + 14].try_into().ok()?),
                u16::from_le_bytes(bytes[start + 14..start + 16].try_into().ok()?),
            ));
        } else if chunk_id == b"data" {
            audio_data = Some(&bytes[start..end]);
        }
        offset = start
            .checked_add(chunk_length)?
            .checked_add(chunk_length & 1)?;
    }
    let (format, channel_count, source_sample_rate, block_align, bits_per_sample) = format?;
    if !matches!(format, 1 | 3)
        || channel_count == 0
        || source_sample_rate == 0
        || block_align == 0
        || !matches!(bits_per_sample, 8 | 16 | 24 | 32)
    {
        return None;
    }
    let data = audio_data?;
    let source_length = data.len() / block_align as usize;
    if source_length == 0 {
        return None;
    }
    let mut source = vec![vec![0.0_f32; source_length]; channel_count as usize];
    let bytes_per_sample = bits_per_sample as usize / 8;
    for frame in 0..source_length {
        for channel in 0..channel_count as usize {
            let start = frame * block_align as usize + channel * bytes_per_sample;
            let sample = match (format, bits_per_sample) {
                (1, 8) => (f32::from(data[start]) - 128.0) / 128.0,
                (1, 16) => {
                    f32::from(i16::from_le_bytes(data[start..start + 2].try_into().ok()?))
                        / 32_768.0
                }
                (1, 24) => {
                    let raw = i32::from(data[start])
                        | (i32::from(data[start + 1]) << 8)
                        | (i32::from(data[start + 2]) << 16);
                    let signed = if raw & 0x80_0000 != 0 {
                        raw | !0xff_ffff
                    } else {
                        raw
                    };
                    signed as f32 / 8_388_608.0
                }
                (1, 32) => {
                    i32::from_le_bytes(data[start..start + 4].try_into().ok()?) as f32
                        / 2_147_483_648.0
                }
                (3, 32) => f32::from_le_bytes(data[start..start + 4].try_into().ok()?),
                _ => return None,
            };
            source[channel][frame] = if sample.is_finite() {
                sample.clamp(-1.0, 1.0)
            } else {
                0.0
            };
        }
    }
    let target_length =
        ((source_length as f64 * target_sample_rate / f64::from(source_sample_rate)).round()
            as usize)
            .max(1);
    let mut channels = vec![vec![0.0_f32; target_length]; channel_count as usize];
    for (target_channel, source_channel) in channels.iter_mut().zip(&source) {
        for (index, target) in target_channel.iter_mut().enumerate() {
            let position = index as f64 * f64::from(source_sample_rate) / target_sample_rate;
            let lower = position.floor() as usize;
            let upper = (lower + 1).min(source_length - 1);
            let amount = (position - lower as f64) as f32;
            *target =
                source_channel[lower] + (source_channel[upper] - source_channel[lower]) * amount;
        }
    }
    Some(DecodedWave {
        channels,
        length: target_length as u32,
    })
}

fn attach_decode_callback(
    scope: &mut v8::PinScope<'_, '_>,
    promise: v8::Local<'_, v8::Promise>,
    callback: v8::Local<'_, v8::Value>,
) {
    if !callback.is_function() {
        return;
    }
    let Some(key) = v8::String::new(scope, "then") else {
        return;
    };
    let Some(then) = promise
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    let _ = then.call(scope, promise.into(), &[callback]);
}

fn reject_audio_decode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    message: &str,
) {
    let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "EncodingError".to_owned())
    else {
        return;
    };
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception.into()) {
        let callback = arguments.get(2);
        if callback.is_function()
            && let Some(key) = v8::String::new(scope, "catch")
            && let Some(catch) = promise
                .get(scope, key.into())
                .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        {
            let _ = catch.call(scope, promise.into(), &[callback]);
        }
        result.set(promise.into());
    }
}
