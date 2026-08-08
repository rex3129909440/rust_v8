use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigationDestinationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationDestinationRecord>,
}
#[derive(Clone)]
struct NavigationDestinationRecord {
    key: String,
    id: String,
    url: String,
    index: i32,
    same_document: bool,
    state: v8::Global<v8::Value>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationDestinationStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigationDestination", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigationDestinationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let c = crate::webidl::create_function(
        scope,
        "NavigationDestination",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "key", get_key)?;
    crate::webidl::define_readonly_accessor(scope, p, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, p, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, p, "index", get_index)?;
    crate::webidl::define_readonly_accessor(scope, p, "sameDocument", get_same_document)?;
    crate::webidl::define_method(scope, p, "getState", 0, get_state)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NavigationDestinationStore>()
        .ok_or_else(|| "NavigationDestination state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    key: String,
    id: String,
    url: String,
    index: i32,
    same_document: bool,
    state: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create NavigationDestination".to_owned());
    }
    let state = super::performance_mark::clone_value(scope, state);
    let state = v8::Global::new(scope, state);
    scope
        .get_slot_mut::<NavigationDestinationStore>()
        .ok_or_else(|| "NavigationDestination state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            NavigationDestinationRecord {
                key,
                id,
                url,
                index,
                same_document,
                state,
            },
        );
    Ok(object)
}
pub(crate) fn is_destination(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<NavigationDestinationStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NavigationDestination': Illegal constructor",
    )
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationDestinationRecord> {
    scope
        .get_slot::<NavigationDestinationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigationDestinationRecord) -> &str,
) {
    if let Some(v) = record(scope, a.this()) {
        if let Some(s) = v8::String::new(scope, select(&v)) {
            r.set(s.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.key)
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.id)
}
fn get_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.url)
}
fn get_index(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new(scope, v.index).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_same_document(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.same_document).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        let value = v8::Local::new(scope, &v.state);
        r.set(super::performance_mark::clone_value(scope, value))
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
