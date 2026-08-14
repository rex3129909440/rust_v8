use std::collections::{HashMap, HashSet};
#[derive(Default)]
pub(crate) struct StorageBucketManagerStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
    buckets: HashMap<String, v8::Global<v8::Object>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(StorageBucketManagerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "StorageBucketManager", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<StorageBucketManagerStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "StorageBucketManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "delete", 1, delete)?;
    crate::webidl::define_method(s, p, "keys", 0, keys)?;
    crate::webidl::define_method(s, p, "open", 1, open)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<StorageBucketManagerStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
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
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create StorageBucketManager".to_owned());
    }
    s.get_slot_mut::<StorageBucketManagerStore>()
        .unwrap()
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<StorageBucketManagerStore>()
        .is_some_and(|x| x.instances.contains(&o.get_identity_hash().get()))
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn delete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucketManager", "delete", r);
        return;
    }
    let name = crate::webidl::value_to_string(s, a.get(0));
    s.get_slot_mut::<StorageBucketManagerStore>()
        .unwrap()
        .buckets
        .remove(&name);
    let x = v8::undefined(s);
    promise(s, x.into(), r)
}
fn keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucketManager", "keys", r);
        return;
    }
    let names = s
        .get_slot::<StorageBucketManagerStore>()
        .unwrap()
        .buckets
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    let array = v8::Array::new(s, names.len() as i32);
    for (i, n) in names.iter().enumerate() {
        if let Some(x) = v8::String::new(s, n) {
            let _ = array.set_index(s, i as u32, x.into());
        }
    }
    promise(s, array.into(), r)
}
fn open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucketManager", "open", r);
        return;
    }
    let name = crate::webidl::value_to_string(s, a.get(0));
    let existing = s
        .get_slot::<StorageBucketManagerStore>()
        .and_then(|x| x.buckets.get(&name))
        .cloned();
    let value = if let Some(v) = existing {
        v8::Local::new(s, &v)
    } else {
        match super::storage_bucket::create(s, name.clone()) {
            Ok(v) => {
                let stored = v8::Global::new(s, v);
                s.get_slot_mut::<StorageBucketManagerStore>()
                    .unwrap()
                    .buckets
                    .insert(name, stored);
                v
            }
            Err(e) => {
                crate::webidl::throw_type_error(s, &e);
                return;
            }
        }
    };
    promise(s, value.into(), r)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<StorageBucketManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
