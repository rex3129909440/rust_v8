use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamAudioDestinationNodeStore {
    constructor: crate::webidl::RealmConstructor,
    streams: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamAudioDestinationNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamAudioDestinationNode", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamAudioDestinationNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamAudioDestinationNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "stream", get_stream)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamAudioDestinationNodeStore>()
        .ok_or_else(|| "MediaStreamAudioDestinationNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamAudioDestinationNode': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamAudioDestinationNode': parameter 1 is not of type 'AudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context)
        || super::offline_audio_context::is_offline_context(scope, context)
    {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamAudioDestinationNode': parameter 1 is not of type 'AudioContext'.",
        );
        return;
    }
    let track = match super::media_stream_track::create(
        scope,
        "audio",
        Some("MediaStreamAudioDestinationNode".to_owned()),
    ) {
        Ok(track) => track,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let stream = match super::media_stream::create_with_tracks(scope, &[track]) {
        Ok(stream) => stream,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    super::audio_node::attach(scope, arguments.this(), Some(context), 1, 0);
    let _ = super::audio_node::set_channel_configuration(
        scope,
        arguments.this(),
        2,
        "explicit".to_owned(),
        "speakers".to_owned(),
    );
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<MediaStreamAudioDestinationNodeStore>()
        .expect("MediaStreamAudioDestinationNode state")
        .streams
        .insert(arguments.this().get_identity_hash().get(), stream);
    result.set(arguments.this().into());
}

fn get_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(stream) = scope
        .get_slot::<MediaStreamAudioDestinationNodeStore>()
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
