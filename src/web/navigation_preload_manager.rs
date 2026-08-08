use std::collections::HashMap;
#[derive(Clone)]
struct PreloadRecord {
    enabled: bool,
    header: String,
}
#[derive(Default)]
pub(crate) struct NavigationPreloadManagerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PreloadRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationPreloadManagerStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigationPreloadManager", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<NavigationPreloadManagerStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NavigationPreloadManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "disable", 0, disable)?;
    crate::webidl::define_method(scope, prototype, "enable", 0, enable)?;
    crate::webidl::define_method(scope, prototype, "getState", 0, get_state)?;
    crate::webidl::define_method(scope, prototype, "setHeaderValue", 1, set_header_value)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NavigationPreloadManagerStore>()
        .ok_or_else(|| "NavigationPreloadManager state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(s)?;
    let prototype = crate::webidl::prototype(s, constructor)?;
    let object = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, object, prototype.into()) != Some(true) {
        return Err("cannot create NavigationPreloadManager".to_owned());
    }
    s.get_slot_mut::<NavigationPreloadManagerStore>()
        .ok_or_else(|| "NavigationPreloadManager state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            PreloadRecord {
                enabled: false,
                header: "true".to_owned(),
            },
        );
    Ok(object)
}
fn resolve(
    s: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(s, value) {
        r.set(promise.into())
    }
}
fn set_enabled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    enabled: bool,
) {
    if let Some(v) = s
        .get_slot_mut::<NavigationPreloadManagerStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.enabled = enabled;
        resolve(s, v8::undefined(s).into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn enable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_enabled(s, a, r, true)
}
fn disable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_enabled(s, a, r, false)
}
fn set_header_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let header = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<NavigationPreloadManagerStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.header = header;
        resolve(s, v8::undefined(s).into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = s
        .get_slot::<NavigationPreloadManagerStore>()
        .and_then(|store| store.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(s);
    let enabled = v8::String::new(s, "enabled").unwrap();
    let header = v8::String::new(s, "headerValue").unwrap();
    let header_value = v8::String::new(s, &v.header).unwrap();
    let _ = object.set(s, enabled.into(), v8::Boolean::new(s, v.enabled).into());
    let _ = object.set(s, header.into(), header_value.into());
    resolve(s, object.into(), r)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<NavigationPreloadManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
