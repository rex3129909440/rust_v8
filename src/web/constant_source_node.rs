use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ConstantSourceNodeStore {
    constructor: crate::webidl::RealmConstructor,
    offsets: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ConstantSourceNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ConstantSourceNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ConstantSourceNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ConstantSourceNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "offset", get_offset)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_scheduled_source_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ConstantSourceNodeStore>()
        .ok_or_else(|| "ConstantSourceNode state was not prepared".to_owned())?
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
        .ok_or_else(|| "cannot create ConstantSourceNode".to_owned())
}

fn option_number(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
    default_value: f64,
) -> f64 {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return default_value;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return default_value;
    };
    object
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(default_value)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "ConstantSourceNode requires a BaseAudioContext");
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ConstantSourceNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ConstantSourceNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let initial = option_number(scope, arguments.get(1), "offset", 1.0);
    let offset =
        match super::audio_param::create(scope, context, initial as f32, f32::MIN, f32::MAX) {
            Ok(offset) => offset,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
    super::audio_node::attach(scope, arguments.this(), Some(context), 0, 1);
    super::audio_scheduled_source_node::attach(scope, arguments.this());
    let offset = v8::Global::new(scope, offset);
    scope
        .get_slot_mut::<ConstantSourceNodeStore>()
        .expect("ConstantSourceNode state")
        .offsets
        .insert(arguments.this().get_identity_hash().get(), offset);
    result.set(arguments.this().into());
}

fn get_offset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(offset) = scope
        .get_slot::<ConstantSourceNodeStore>()
        .and_then(|store| {
            store
                .offsets
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    {
        result.set(v8::Local::new(scope, &offset).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn sample_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<f32> {
    let offset = scope
        .get_slot::<ConstantSourceNodeStore>()?
        .offsets
        .get(&object.get_identity_hash().get())?;
    if !super::audio_scheduled_source_node::is_active_at(scope, object, time) {
        return Some(0.0);
    }
    super::audio_param::value_at(scope, v8::Local::new(scope, offset), time)
}
