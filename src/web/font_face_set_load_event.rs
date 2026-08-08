use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct FontFaceSetLoadEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, v8::Global<v8::Array>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FontFaceSetLoadEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FontFaceSetLoadEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FontFaceSetLoadEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FontFaceSetLoadEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::font_face_set_load_event_fontfaces_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FontFaceSetLoadEventStore>()
        .ok_or_else(|| "FontFaceSetLoadEvent state was not prepared".to_owned())?
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
            "Failed to construct 'FontFaceSetLoadEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles = init.is_some_and(|value| super::event::boolean_property(scope, value, "bubbles"));
    let cancelable =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "cancelable"));
    let composed =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "composed"));
    let fontfaces = init
        .and_then(|value| {
            let key = v8::String::new(scope, "fontfaces")?;
            value.get(scope, key.into())
        })
        .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
        .unwrap_or_else(|| v8::Array::new(scope, 0));
    let copied = v8::Array::new(scope, fontfaces.length() as i32);
    for index in 0..fontfaces.length() {
        if let Some(value) = fontfaces.get_index(scope, index) {
            let _ = copied.set_index(scope, index, value);
        }
    }
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let copied = v8::Global::new(scope, copied);
    scope
        .get_slot_mut::<FontFaceSetLoadEventStore>()
        .expect("FontFaceSetLoadEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), copied);
    result.set(arguments.this().into());
}

pub(crate) fn get_fontfaces(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(fontfaces) = scope
        .get_slot::<FontFaceSetLoadEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    {
        result.set(v8::Local::new(scope, &fontfaces).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
