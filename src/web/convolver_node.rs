use std::collections::HashMap;

#[derive(Clone)]
struct ConvolverRecord {
    buffer: Option<v8::Global<v8::Object>>,
    normalize: bool,
}

#[derive(Default)]
pub(crate) struct ConvolverNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ConvolverRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ConvolverNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ConvolverNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ConvolverNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ConvolverNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "buffer", get_buffer, set_buffer)?;
    crate::webidl::define_accessor(scope, prototype, "normalize", get_normalize, set_normalize)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ConvolverNodeStore>()
        .ok_or_else(|| "ConvolverNode state was not prepared".to_owned())?
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
        .ok_or_else(|| "cannot create ConvolverNode".to_owned())
}

fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "ConvolverNode requires a BaseAudioContext");
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ConvolverNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ConvolverNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let normalize = options
        .and_then(|options| object_property(scope, options, "disableNormalization"))
        .map(|value| !value.boolean_value(scope))
        .unwrap_or(true);
    let buffer_value = options.and_then(|options| object_property(scope, options, "buffer"));
    let buffer = match buffer_value {
        Some(value) if !value.is_null_or_undefined() => {
            let Ok(buffer) = v8::Local::<v8::Object>::try_from(value) else {
                crate::webidl::throw_type_error(scope, "buffer is not an AudioBuffer");
                return;
            };
            if !super::audio_buffer::is_buffer(scope, buffer) {
                crate::webidl::throw_type_error(scope, "buffer is not an AudioBuffer");
                return;
            }
            Some(v8::Global::new(scope, buffer))
        }
        _ => None,
    };
    super::audio_node::attach(scope, arguments.this(), Some(context), 1, 1);
    scope
        .get_slot_mut::<ConvolverNodeStore>()
        .expect("ConvolverNode state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ConvolverRecord { buffer, normalize },
        );
    result.set(arguments.this().into());
}

fn get_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let buffer = scope
        .get_slot::<ConvolverNodeStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.buffer.clone());
    match buffer {
        Some(Some(buffer)) => result.set(v8::Local::new(scope, &buffer).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let buffer = if arguments.get(0).is_null() {
        None
    } else {
        let Ok(buffer) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "buffer is not an AudioBuffer");
            return;
        };
        if !super::audio_buffer::is_buffer(scope, buffer) {
            crate::webidl::throw_type_error(scope, "buffer is not an AudioBuffer");
            return;
        }
        Some(v8::Global::new(scope, buffer))
    };
    if let Some(record) = scope
        .get_slot_mut::<ConvolverNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.buffer = buffer;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_normalize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot::<ConvolverNodeStore>().and_then(|store| {
        store
            .records
            .get(&arguments.this().get_identity_hash().get())
    }) {
        result.set(v8::Boolean::new(scope, record.normalize).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_normalize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let normalize = arguments.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<ConvolverNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.normalize = normalize;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn impulse_response(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    channel: u32,
) -> Option<Vec<f32>> {
    let record = scope
        .get_slot::<ConvolverNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())?;
    let Some(buffer) = record.buffer.as_ref() else {
        return Some(Vec::new());
    };
    let buffer = v8::Local::new(scope, buffer);
    let length = super::audio_buffer::length(scope, buffer)?;
    let channels = super::audio_buffer::number_of_channels(scope, buffer)?;
    let channel = channel.min(channels.saturating_sub(1));
    let mut impulse = (0..length)
        .map(|index| super::audio_buffer::sample(scope, buffer, channel, index).unwrap_or(0.0))
        .collect::<Vec<_>>();
    if record.normalize {
        let energy = impulse
            .iter()
            .map(|sample| f64::from(*sample) * f64::from(*sample))
            .sum::<f64>()
            .sqrt();
        if energy > f64::EPSILON {
            let scale = (1.0 / energy).min(1.0) as f32;
            for sample in &mut impulse {
                *sample *= scale;
            }
        }
    }
    Some(impulse)
}
