use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct ExternalStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ExternalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "External", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ExternalStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "External",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "AddSearchProvider",
        0,
        add_search_provider,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "IsSearchProviderInstalled",
        0,
        is_search_provider_installed,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ExternalStore>()
        .ok_or_else(|| "External state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create External".to_owned());
    }
    scope
        .get_slot_mut::<ExternalStore>()
        .ok_or_else(|| "External state was not prepared".to_owned())?
        .objects
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<ExternalStore>()
        .is_some_and(|store| store.objects.contains(&object.get_identity_hash().get()))
}

fn add_search_provider(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn is_search_provider_installed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
