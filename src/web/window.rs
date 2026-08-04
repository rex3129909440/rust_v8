use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WindowStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WindowStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    super::object_get_prototype_of::install(scope)?;
    super::reflect_get_prototype_of::install(scope)?;
    super::object_get_own_property_descriptor::install(scope)?;
    super::reflect_get_own_property_descriptor::install(scope)?;
    super::object_get_own_property_descriptors::install(scope)?;
    super::object_get_own_property_names::install(scope)?;
    super::object_get_own_property_symbols::install(scope)?;
    super::reflect_own_keys::install(scope)?;
    super::object_keys::install(scope)?;
    super::object_values::install(scope)?;
    super::object_entries::install(scope)?;
    super::object_freeze::install(scope)?;
    super::object_seal::install(scope)?;
    super::object_set_prototype_of::install(scope)?;
    super::reflect_set_prototype_of::install(scope)?;
    super::object_prevent_extensions::install(scope)?;
    super::reflect_prevent_extensions::install(scope)?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let global = scope.get_current_context().global(scope);
    if crate::webidl::set_platform_prototype(scope, global, prototype.into()) != Some(true) {
        return Err("cannot attach Window prototype to global object".to_owned());
    }
    super::event_target::attach(scope, global);
    crate::webidl::define_global(scope, "Window", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<WindowStore>()
        .and_then(|store| store.constructors.get(&realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Window",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let properties = super::window_properties::create(scope, parent)?;
    if crate::webidl::set_platform_prototype(scope, prototype, properties.into()) != Some(true) {
        return Err("cannot set WindowProperties as Window prototype parent".to_owned());
    }
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_constant(scope, prototype, "TEMPORARY", 0)?;
    crate::webidl::define_constant(scope, prototype, "PERSISTENT", 1)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_constant(scope, constructor.into(), "TEMPORARY", 0)?;
    crate::webidl::define_constant(scope, constructor.into(), "PERSISTENT", 1)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WindowStore>()
        .ok_or_else(|| "Window state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Window': Illegal constructor");
}
