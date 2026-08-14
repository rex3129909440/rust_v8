use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct GainNodeStore {
    constructor: crate::webidl::RealmConstructor,
    gains: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GainNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GainNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<GainNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GainNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "gain", get_gain)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GainNodeStore>()
        .ok_or_else(|| "GainNode state was not prepared".to_owned())?
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
        .ok_or_else(|| "cannot create GainNode".to_owned())
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'GainNode': 1 argument required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'GainNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'GainNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let gain_value = options
        .map(|object| super::event::number_property(scope, object, "gain", 1.0))
        .unwrap_or(1.0);
    let channel_count = options
        .map(|object| super::event::number_property(scope, object, "channelCount", 2.0) as u32)
        .unwrap_or(2)
        .max(1);
    let channel_count_mode = option_string(scope, options, "channelCountMode", "max");
    let channel_interpretation = option_string(scope, options, "channelInterpretation", "speakers");
    match attach(
        scope,
        arguments.this(),
        context,
        gain_value,
        channel_count,
        channel_count_mode,
        channel_interpretation,
    ) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
    gain_value: f64,
    channel_count: u32,
    channel_count_mode: String,
    channel_interpretation: String,
) -> Result<(), String> {
    super::audio_node::attach(scope, object, Some(context), 1, 1);
    let _ = super::audio_node::set_channel_configuration(
        scope,
        object,
        channel_count,
        channel_count_mode,
        channel_interpretation,
    );
    let gain =
        super::audio_param::create(scope, context, 1.0, -3.402_823_5e38_f32, 3.402_823_5e38_f32)?;
    let _ = super::audio_param::set_current_value(scope, gain, gain_value as f32);
    let gain = v8::Global::new(scope, gain);
    scope
        .get_slot_mut::<GainNodeStore>()
        .ok_or_else(|| "GainNode state was not prepared".to_owned())?
        .gains
        .insert(object.get_identity_hash().get(), gain);
    Ok(())
}

fn option_string(
    scope: &mut v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    fallback: &str,
) -> String {
    let Some(options) = options else {
        return fallback.to_owned();
    };
    let Some(key) = v8::String::new(scope, name) else {
        return fallback.to_owned();
    };
    let Some(value) = options.get(scope, key.into()) else {
        return fallback.to_owned();
    };
    if value.is_undefined() {
        fallback.to_owned()
    } else {
        crate::webidl::value_to_string(scope, value)
    }
}

fn get_gain(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(gain) = scope
        .get_slot::<GainNodeStore>()
        .and_then(|store| store.gains.get(&arguments.this().get_identity_hash().get()))
        .cloned()
    {
        result.set(v8::Local::new(scope, &gain).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn gain_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<f32> {
    let gain = scope
        .get_slot::<GainNodeStore>()?
        .gains
        .get(&object.get_identity_hash().get())?;
    super::audio_param::value_at(scope, v8::Local::new(scope, gain), time)
}
