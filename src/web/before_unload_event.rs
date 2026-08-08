use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct BeforeUnloadEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) values: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BeforeUnloadEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BeforeUnloadEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<BeforeUnloadEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BeforeUnloadEvent",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::before_unload_event_return_value_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BeforeUnloadEventStore>()
        .ok_or_else(|| "BeforeUnloadEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create BeforeUnloadEvent".to_owned());
    }
    super::event::attach(scope, event, String::new(), false, false, false);
    scope
        .get_slot_mut::<BeforeUnloadEventStore>()
        .ok_or_else(|| "BeforeUnloadEvent state was not prepared".to_owned())?
        .values
        .insert(event.get_identity_hash().get(), String::new());
    Ok(event)
}

pub(crate) fn get_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<BeforeUnloadEventStore>()
        .and_then(|store| {
            store
                .values
                .get(&arguments.this().get_identity_hash().get())
        })
        && let Some(value) = v8::String::new(scope, value)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(current) = scope
        .get_slot_mut::<BeforeUnloadEventStore>()
        .and_then(|store| {
            store
                .values
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
