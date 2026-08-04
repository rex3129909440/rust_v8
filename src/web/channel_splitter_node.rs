use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct ChannelSplitterNodeStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ChannelSplitterNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ChannelSplitterNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ChannelSplitterNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ChannelSplitterNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ChannelSplitterNodeStore>()
        .ok_or_else(|| "ChannelSplitterNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    number_of_outputs: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let options = v8::Object::new(scope);
    let key = v8::String::new(scope, "numberOfOutputs")
        .ok_or_else(|| "cannot create ChannelSplitterNode options".to_owned())?;
    let value = v8::Integer::new_from_unsigned(scope, number_of_outputs);
    if options.create_data_property(scope, key.into(), value.into()) != Some(true) {
        return Err("cannot set ChannelSplitterNode options".to_owned());
    }
    constructor
        .new_instance(scope, &[context.into(), options.into()])
        .ok_or_else(|| "cannot create ChannelSplitterNode".to_owned())
}

fn output_count(scope: &v8::PinScope<'_, '_>, options: v8::Local<'_, v8::Value>) -> u32 {
    let Ok(options) = v8::Local::<v8::Object>::try_from(options) else {
        return 6;
    };
    let Some(key) = v8::String::new(scope, "numberOfOutputs") else {
        return 6;
    };
    options
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(6)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "ChannelSplitterNode requires a BaseAudioContext");
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
    let outputs = output_count(scope, arguments.get(1));
    if outputs == 0 || outputs > 32 {
        crate::webidl::throw_type_error(scope, "numberOfOutputs is outside the supported range");
        return;
    }
    super::audio_node::attach(scope, arguments.this(), Some(context), 1, outputs);
    super::audio_node::set_channel_configuration(
        scope,
        arguments.this(),
        outputs,
        "explicit".to_owned(),
        "discrete".to_owned(),
    );
    scope
        .get_slot_mut::<ChannelSplitterNodeStore>()
        .expect("ChannelSplitterNode state")
        .instances
        .insert(arguments.this().get_identity_hash().get());
    result.set(arguments.this().into());
}

pub(crate) fn is_channel_splitter(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<ChannelSplitterNodeStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}
