use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct SyncManagerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, HashSet<String>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SyncManagerStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SyncManager", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<SyncManagerStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "SyncManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_method(scope, p, "getTags", 0, get_tags)?;
    crate::webidl::define_method(scope, p, "register", 1, register)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SyncManagerStore>()
        .ok_or_else(|| "SyncManager state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create SyncManager".to_owned());
    }
    scope
        .get_slot_mut::<SyncManagerStore>()
        .ok_or_else(|| "SyncManager state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), HashSet::new());
    Ok(o)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SyncManager': Illegal constructor",
    );
}
fn get_tags(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(tags) = scope
        .get_slot::<SyncManagerStore>()
        .and_then(|s| s.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut tags: Vec<_> = tags.into_iter().collect();
    tags.sort();
    let array = v8::Array::new(scope, tags.len() as i32);
    for (i, tag) in tags.iter().enumerate() {
        if let Some(v) = v8::String::new(scope, tag) {
            let _ = array.set_index(scope, i as u32, v.into());
        }
    }
    if let Ok(p) = super::writable_stream::resolved_promise(scope, array.into()) {
        r.set(p.into())
    }
}
fn register(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if a.length() < 1 {
        crate::webidl::throw_type_error(scope, "register requires a tag");
        return;
    }
    let tag = crate::webidl::value_to_string(scope, a.get(0));
    if tag.is_empty() {
        crate::webidl::throw_type_error(scope, "The tag cannot be empty");
        return;
    }
    let Some(tags) = scope
        .get_slot_mut::<SyncManagerStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    tags.insert(tag);
    let value = v8::undefined(scope);
    if let Ok(p) = super::writable_stream::resolved_promise(scope, value.into()) {
        r.set(p.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SyncManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
