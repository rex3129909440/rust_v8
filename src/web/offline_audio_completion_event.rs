use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct OfflineAudioCompletionEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) rendered_buffers: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OfflineAudioCompletionEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "OfflineAudioCompletionEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<OfflineAudioCompletionEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "OfflineAudioCompletionEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::offline_audio_completion_event_rendered_buffer_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<OfflineAudioCompletionEventStore>()
        .ok_or_else(|| "OfflineAudioCompletionEvent state was not prepared".to_owned())?
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
            "Failed to construct 'OfflineAudioCompletionEvent': 2 arguments required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The event initializer must be an object");
        return;
    };
    let Some(rendered_buffer_key) = v8::String::new(scope, "renderedBuffer") else {
        return;
    };
    let Some(rendered_buffer_value) = init.get(scope, rendered_buffer_key.into()) else {
        return;
    };
    let Ok(rendered_buffer) = v8::Local::<v8::Object>::try_from(rendered_buffer_value) else {
        crate::webidl::throw_type_error(scope, "renderedBuffer is required");
        return;
    };
    if let Err(message) = attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        init,
        rendered_buffer,
    ) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    rendered_buffer: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create OfflineAudioCompletionEvent".to_owned());
    }
    let init = v8::Object::new(scope);
    attach(scope, event, event_type.to_owned(), init, rendered_buffer)?;
    Ok(event)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    event_type: String,
    init: v8::Local<'_, v8::Object>,
    rendered_buffer: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let bubbles = super::event::boolean_property(scope, init, "bubbles");
    let cancelable = super::event::boolean_property(scope, init, "cancelable");
    let composed = super::event::boolean_property(scope, init, "composed");
    super::event::attach(scope, event, event_type, bubbles, cancelable, composed);
    let rendered_buffer = v8::Global::new(scope, rendered_buffer);
    scope
        .get_slot_mut::<OfflineAudioCompletionEventStore>()
        .ok_or_else(|| "OfflineAudioCompletionEvent state was not prepared".to_owned())?
        .rendered_buffers
        .insert(event.get_identity_hash().get(), rendered_buffer);
    Ok(())
}

pub(crate) fn get_rendered_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(buffer) = scope
        .get_slot::<OfflineAudioCompletionEventStore>()
        .and_then(|store| {
            store
                .rendered_buffers
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, buffer).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
