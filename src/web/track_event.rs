use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TrackEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) tracks: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TrackEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TrackEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TrackEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TrackEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::track_event_track_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TrackEventStore>()
        .ok_or_else(|| "TrackEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'TrackEvent': use the new operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "bubbles"));
    let cancelable =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "cancelable"));
    let composed =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "composed"));
    let track = init.and_then(|object| {
        let key = v8::String::new(scope, "track")?;
        let value = object.get(scope, key.into())?;
        if value.is_null() || value.is_undefined() {
            None
        } else {
            v8::Local::<v8::Object>::try_from(value).ok()
        }
    });
    let object = arguments.this();
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    let track = track.map(|track| v8::Global::new(scope, track));
    scope
        .get_slot_mut::<TrackEventStore>()
        .expect("TrackEvent state")
        .tracks
        .insert(object.get_identity_hash().get(), track);
    result.set(object.into());
}

pub(crate) fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let track = scope
        .get_slot::<TrackEventStore>()
        .and_then(|store| {
            store
                .tracks
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    let Some(track) = track else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(track) = track {
        result.set(v8::Local::new(scope, &track).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
