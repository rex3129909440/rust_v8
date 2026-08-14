use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigationPrecommitControllerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationPrecommitRecord>,
}
#[derive(Clone, Default)]
struct NavigationPrecommitRecord {
    redirect_url: Option<String>,
    handlers: Vec<v8::Global<v8::Function>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationPrecommitControllerStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigationPrecommitController", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigationPrecommitControllerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let c = crate::webidl::create_function(
        scope,
        "NavigationPrecommitController",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_method(scope, p, "redirect", 1, redirect)?;
    crate::webidl::define_method(scope, p, "addHandler", 1, add_handler)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NavigationPrecommitControllerStore>()
        .ok_or_else(|| "NavigationPrecommitController state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create NavigationPrecommitController".to_owned());
    }
    scope
        .get_slot_mut::<NavigationPrecommitControllerStore>()
        .ok_or_else(|| "NavigationPrecommitController state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            NavigationPrecommitRecord::default(),
        );
    Ok(object)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NavigationPrecommitController': Illegal constructor",
    )
}
fn redirect(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if scope
        .get_slot::<NavigationPrecommitControllerStore>()
        .and_then(|store| store.records.get(&a.this().get_identity_hash().get()))
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "redirect requires options");
        return;
    };
    let Some(key) = v8::String::new(scope, "url") else {
        return;
    };
    let url = options
        .get(scope, key.into())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    if url.is_empty() {
        crate::webidl::throw_type_error(scope, "redirect URL is required");
        return;
    }
    if let Some(v) = scope
        .get_slot_mut::<NavigationPrecommitControllerStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.redirect_url = Some(url)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn add_handler(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if scope
        .get_slot::<NavigationPrecommitControllerStore>()
        .and_then(|store| store.records.get(&a.this().get_identity_hash().get()))
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(handler) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "addHandler requires a function");
        return;
    };
    let handler = v8::Global::new(scope, handler);
    if let Some(v) = scope
        .get_slot_mut::<NavigationPrecommitControllerStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handlers.push(handler)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
