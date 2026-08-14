use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DocumentPictureInPictureEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) windows: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentPictureInPictureEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DocumentPictureInPictureEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DocumentPictureInPictureEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DocumentPictureInPictureEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::document_picture_in_picture_event_window_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DocumentPictureInPictureEventStore>()
        .ok_or_else(|| "DocumentPictureInPictureEvent state was not prepared".to_owned())?
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
            "Failed to construct 'DocumentPictureInPictureEvent': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to construct 'DocumentPictureInPictureEvent': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'DocumentPictureInPictureEvent': The provided value is not of type 'DocumentPictureInPictureEventInit'.",
        );
        return;
    };
    let Some(key) = v8::String::new(scope, "window") else {
        return;
    };
    let Some(window) = init.get(scope, key.into()) else {
        return;
    };
    let Ok(window) = v8::Local::<v8::Object>::try_from(window) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'DocumentPictureInPictureEvent': Failed to read the 'window' property from 'DocumentPictureInPictureEventInit': Required member is undefined.",
        );
        return;
    };
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let window = v8::Global::new(scope, window);
    scope
        .get_slot_mut::<DocumentPictureInPictureEventStore>()
        .expect("DocumentPictureInPictureEvent state")
        .windows
        .insert(arguments.this().get_identity_hash().get(), window);
    result.set(arguments.this().into());
}

pub(crate) fn get_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let window = scope
        .get_slot::<DocumentPictureInPictureEventStore>()
        .and_then(|store| {
            store
                .windows
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(window) = window {
        result.set(v8::Local::new(scope, &window).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
