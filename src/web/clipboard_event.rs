use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ClipboardEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) data: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ClipboardEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ClipboardEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ClipboardEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ClipboardEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::clipboard_event_clipboard_data_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ClipboardEventStore>()
        .ok_or_else(|| "ClipboardEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn option<'s>(
    scope: &v8::PinScope<'s, '_>,
    options: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = v8::Local::<v8::Object>::try_from(options).ok()?;
    object.get(scope, v8::String::new(scope, name)?.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "ClipboardEvent requires an event type");
        return;
    }
    let options = arguments.get(1);
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = option(scope, options, "bubbles").is_some_and(|value| value.boolean_value(scope));
    let cancelable =
        option(scope, options, "cancelable").is_some_and(|value| value.boolean_value(scope));
    let composed =
        option(scope, options, "composed").is_some_and(|value| value.boolean_value(scope));
    let data = option(scope, options, "clipboardData")
        .filter(|value| !value.is_null_or_undefined())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .map(|value| v8::Global::new(scope, value));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<ClipboardEventStore>()
        .expect("ClipboardEvent state")
        .data
        .insert(arguments.this().get_identity_hash().get(), data);
    result.set(arguments.this().into());
}

pub(crate) fn get_clipboard_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<ClipboardEventStore>()
        .and_then(|store| store.data.get(&arguments.this().get_identity_hash().get()))
        .cloned();
    match value {
        Some(Some(value)) => result.set(v8::Local::new(scope, &value).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
