use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PictureInPictureEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PictureInPictureEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PictureInPictureEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PictureInPictureEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PictureInPictureEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::picture_in_picture_event_picture_in_picture_window_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PictureInPictureEventStore>()
        .ok_or_else(|| "PictureInPictureEvent state was not prepared".to_owned())?
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
            "Failed to construct 'PictureInPictureEvent': 2 arguments required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "PictureInPictureEventInit must be an object");
        return;
    };
    let Some(key) = v8::String::new(scope, "pictureInPictureWindow") else {
        return;
    };
    let Some(value) = init.get(scope, key.into()) else {
        crate::webidl::throw_type_error(
            scope,
            "Required member 'pictureInPictureWindow' is undefined",
        );
        return;
    };
    if value.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Required member 'pictureInPictureWindow' is undefined",
        );
        return;
    }
    let Ok(window) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "value is not a PictureInPictureWindow");
        return;
    };
    if !super::picture_in_picture_window::is_instance(scope, window) {
        crate::webidl::throw_type_error(scope, "value is not a PictureInPictureWindow");
        return;
    }
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        super::event::boolean_property(scope, init, "bubbles"),
        super::event::boolean_property(scope, init, "cancelable"),
        super::event::boolean_property(scope, init, "composed"),
    );
    let window = v8::Global::new(scope, window);
    scope
        .get_slot_mut::<PictureInPictureEventStore>()
        .expect("PictureInPictureEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), window);
    result.set(arguments.this().into());
}

pub(crate) fn get_picture_in_picture_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(window) = scope
        .get_slot::<PictureInPictureEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, window).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
