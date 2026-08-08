use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigationActivationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationActivationRecord>,
}
#[derive(Clone)]
struct NavigationActivationRecord {
    entry: v8::Global<v8::Object>,
    from: Option<v8::Global<v8::Object>>,
    navigation_type: Option<String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationActivationStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigationActivation", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigationActivationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let c = crate::webidl::create_function(
        scope,
        "NavigationActivation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "entry", get_entry)?;
    crate::webidl::define_readonly_accessor(scope, p, "from", get_from)?;
    crate::webidl::define_readonly_accessor(scope, p, "navigationType", get_navigation_type)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NavigationActivationStore>()
        .ok_or_else(|| "NavigationActivation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    entry: v8::Local<'_, v8::Object>,
    from: Option<v8::Local<'_, v8::Object>>,
    navigation_type: Option<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create NavigationActivation".to_owned());
    }
    let entry = v8::Global::new(scope, entry);
    let from = from.map(|value| v8::Global::new(scope, value));
    scope
        .get_slot_mut::<NavigationActivationStore>()
        .ok_or_else(|| "NavigationActivation state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            NavigationActivationRecord {
                entry,
                from,
                navigation_type,
            },
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
        "Failed to construct 'NavigationActivation': Illegal constructor",
    )
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationActivationRecord> {
    scope
        .get_slot::<NavigationActivationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_entry(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.entry).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_from(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(scope, a.this()) {
        Some(v) => match v.from {
            Some(value) => r.set(v8::Local::new(scope, &value).into()),
            None => r.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
fn get_navigation_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(scope, a.this()) {
        Some(v) => match v.navigation_type {
            Some(value) => {
                if let Some(s) = v8::String::new(scope, &value) {
                    r.set(s.into())
                }
            }
            None => r.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
