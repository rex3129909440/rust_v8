use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamTrackEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) tracks: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamTrackEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamTrackEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamTrackEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamTrackEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::media_stream_track_event_track_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamTrackEventStore>()
        .ok_or_else(|| "MediaStreamTrackEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamTrackEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamTrackEvent': The provided value is not of type 'MediaStreamTrackEventInit'.",
        );
        return;
    };
    let Some(key) = v8::String::new(scope, "track") else {
        return;
    };
    let Some(track_value) = init.get(scope, key.into()) else {
        return;
    };
    if track_value.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamTrackEvent': Failed to read the 'track' property from 'MediaStreamTrackEventInit': Required member is undefined.",
        );
        return;
    }
    let Ok(track) = v8::Local::<v8::Object>::try_from(track_value) else {
        crate::webidl::throw_type_error(scope, "track is not a MediaStreamTrack");
        return;
    };
    if !super::media_stream_track::is_track(scope, track) {
        crate::webidl::throw_type_error(scope, "track is not a MediaStreamTrack");
        return;
    }
    let bubbles = super::event::boolean_property(scope, init, "bubbles");
    let cancelable = super::event::boolean_property(scope, init, "cancelable");
    let composed = super::event::boolean_property(scope, init, "composed");
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let track = v8::Global::new(scope, track);
    scope
        .get_slot_mut::<MediaStreamTrackEventStore>()
        .expect("MediaStreamTrackEvent state")
        .tracks
        .insert(arguments.this().get_identity_hash().get(), track);
    result.set(arguments.this().into());
}

pub(crate) fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(track) = scope
        .get_slot::<MediaStreamTrackEventStore>()
        .and_then(|store| {
            store
                .tracks
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, track).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
