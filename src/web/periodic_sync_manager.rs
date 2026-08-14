use std::collections::{BTreeSet, HashSet};
#[derive(Default)]
pub(crate) struct PeriodicSyncManagerStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
    tags: BTreeSet<String>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PeriodicSyncManagerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PeriodicSyncManager", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(c) = s
        .get_slot::<PeriodicSyncManagerStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "PeriodicSyncManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "getTags", 0, get_tags)?;
    crate::webidl::define_method(s, p, "register", 1, register)?;
    crate::webidl::define_method(s, p, "unregister", 1, unregister)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PeriodicSyncManagerStore>()
        .ok_or_else(|| "PeriodicSyncManager state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
#[allow(dead_code)]
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create PeriodicSyncManager".to_owned());
    }
    s.get_slot_mut::<PeriodicSyncManagerStore>()
        .ok_or_else(|| "state missing".to_owned())?
        .objects
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<PeriodicSyncManagerStore>()
        .is_some_and(|x| x.objects.contains(&o.get_identity_hash().get()))
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'PeriodicSyncManager': Illegal constructor",
    )
}
fn get_tags(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "PeriodicSyncManager", "getTags", r);
        return;
    }
    let tags = s
        .get_slot::<PeriodicSyncManagerStore>()
        .map(|x| x.tags.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let arr = v8::Array::new(s, tags.len() as i32);
    for (i, t) in tags.iter().enumerate() {
        if let Some(v) = v8::String::new(s, t) {
            let _ = arr.set_index(s, i as u32, v.into());
        }
    }
    if let Ok(p) = super::writable_stream::resolved_promise(s, arr.into()) {
        r.set(p.into())
    }
}
fn register(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "PeriodicSyncManager", "register", r);
        return;
    }
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'register' on 'PeriodicSyncManager': 1 argument required, but only 0 present.",
        );
        return;
    }
    let tag = crate::webidl::value_to_string(s, a.get(0));
    if let Some(x) = s.get_slot_mut::<PeriodicSyncManagerStore>() {
        x.tags.insert(tag);
    }
    resolved(s, &mut r)
}
fn unregister(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "PeriodicSyncManager", "unregister", r);
        return;
    }
    let tag = crate::webidl::value_to_string(s, a.get(0));
    if let Some(x) = s.get_slot_mut::<PeriodicSyncManagerStore>() {
        x.tags.remove(&tag);
    }
    resolved(s, &mut r)
}
fn resolved(s: &mut v8::PinScope<'_, '_>, r: &mut v8::ReturnValue<'_>) {
    let u = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, u.into()) {
        r.set(p.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PeriodicSyncManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
