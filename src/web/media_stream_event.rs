use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) streams: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::media_stream_event_stream_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamEventStore>()
        .ok_or_else(|| "MediaStreamEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let stream = init.and_then(|init| {
        let key = v8::String::new(scope, "stream")?;
        let value = init.get(scope, key.into())?;
        if value.is_null() || value.is_undefined() {
            None
        } else {
            v8::Local::<v8::Object>::try_from(value).ok()
        }
    });
    if let Some(stream) = stream {
        if !super::media_stream::is_stream(scope, stream) {
            crate::webidl::throw_type_error(scope, "stream is not a MediaStream");
            return;
        }
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = init.is_some_and(|init| super::event::boolean_property(scope, init, "bubbles"));
    let cancelable =
        init.is_some_and(|init| super::event::boolean_property(scope, init, "cancelable"));
    let composed = init.is_some_and(|init| super::event::boolean_property(scope, init, "composed"));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let stream = stream.map(|stream| v8::Global::new(scope, stream));
    scope
        .get_slot_mut::<MediaStreamEventStore>()
        .expect("MediaStreamEvent state")
        .streams
        .insert(arguments.this().get_identity_hash().get(), stream);
    result.set(arguments.this().into());
}

pub(crate) fn get_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let stream = scope
        .get_slot::<MediaStreamEventStore>()
        .and_then(|store| {
            store
                .streams
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    let Some(stream) = stream else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(stream) = stream {
        result.set(v8::Local::new(scope, &stream).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
