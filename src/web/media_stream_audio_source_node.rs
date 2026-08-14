use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamAudioSourceNodeStore {
    constructor: crate::webidl::RealmConstructor,
    streams: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamAudioSourceNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamAudioSourceNode", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamAudioSourceNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamAudioSourceNode",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mediaStream", get_media_stream)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamAudioSourceNodeStore>()
        .ok_or_else(|| "MediaStreamAudioSourceNode state was not prepared".to_owned())?
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
            "Failed to construct 'MediaStreamAudioSourceNode': 2 arguments required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamAudioSourceNode': parameter 1 is not of type 'AudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context)
        || super::offline_audio_context::is_offline_context(scope, context)
    {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamAudioSourceNode': parameter 1 is not of type 'AudioContext'.",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The options argument must be an object");
        return;
    };
    let Some(key) = v8::String::new(scope, "mediaStream") else {
        return;
    };
    let Some(stream_value) = options.get(scope, key.into()) else {
        return;
    };
    let Ok(stream) = v8::Local::<v8::Object>::try_from(stream_value) else {
        crate::webidl::throw_type_error(scope, "mediaStream is required");
        return;
    };
    if !super::media_stream::is_stream(scope, stream) {
        crate::webidl::throw_type_error(scope, "mediaStream is not a MediaStream");
        return;
    }
    if !super::media_stream::has_audio_track(scope, stream) {
        throw_invalid_state(scope);
        return;
    }
    super::audio_node::attach(scope, arguments.this(), Some(context), 0, 1);
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<MediaStreamAudioSourceNodeStore>()
        .expect("MediaStreamAudioSourceNode state")
        .streams
        .insert(arguments.this().get_identity_hash().get(), stream);
    result.set(arguments.this().into());
}

fn get_media_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(stream) = scope
        .get_slot::<MediaStreamAudioSourceNodeStore>()
        .and_then(|store| {
            store
                .streams
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, stream).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>) {
    match super::dom_exception::create(
        scope,
        "Failed to construct 'MediaStreamAudioSourceNode': MediaStream has no audio track"
            .to_owned(),
        "InvalidStateError".to_owned(),
    ) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
