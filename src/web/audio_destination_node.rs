use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AudioDestinationNodeStore {
    constructor: crate::webidl::RealmConstructor,
    max_channel_counts: HashMap<i32, u32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioDestinationNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioDestinationNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioDestinationNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioDestinationNode",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxChannelCount",
        get_max_channel_count,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioDestinationNodeStore>()
        .ok_or_else(|| "AudioDestinationNode state was not prepared".to_owned())?
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
        "Failed to construct 'AudioDestinationNode': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    max_channel_count: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let destination = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, destination, prototype.into()) != Some(true) {
        return Err("cannot create AudioDestinationNode".to_owned());
    }
    super::audio_node::attach(scope, destination, Some(context), 1, 0);
    super::audio_node::set_channel_configuration(
        scope,
        destination,
        max_channel_count.min(2).max(1),
        "explicit".to_owned(),
        "speakers".to_owned(),
    );
    scope
        .get_slot_mut::<AudioDestinationNodeStore>()
        .ok_or_else(|| "AudioDestinationNode state was not prepared".to_owned())?
        .max_channel_counts
        .insert(destination.get_identity_hash().get(), max_channel_count);
    Ok(destination)
}

fn get_max_channel_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<AudioDestinationNodeStore>()
        .and_then(|store| {
            store
                .max_channel_counts
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Integer::new_from_unsigned(scope, *value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
