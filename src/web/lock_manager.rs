use std::collections::HashSet;
#[derive(Default)]
pub(crate) struct LockManagerStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(LockManagerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "LockManager", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<LockManagerStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "LockManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "query", 0, query)?;
    crate::webidl::define_method(s, p, "request", 2, request)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<LockManagerStore>()
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
        return Err("cannot create LockManager".to_owned());
    }
    s.get_slot_mut::<LockManagerStore>()
        .unwrap()
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<LockManagerStore>()
        .is_some_and(|x| x.instances.contains(&o.get_identity_hash().get()))
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn query(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let o = v8::Object::new(s);
    if let Some(k) = v8::String::new(s, "held") {
        let v = v8::Array::new(s, 0);
        let _ = o.set(s, k.into(), v.into());
    }
    if let Some(k) = v8::String::new(s, "pending") {
        let v = v8::Array::new(s, 0);
        let _ = o.set(s, k.into(), v.into());
    }
    promise(s, o.into(), r)
}
fn request(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let name = crate::webidl::value_to_string(s, a.get(0));
    let callback_index = if a.get(1).is_function() { 1 } else { 2 };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(callback_index)) else {
        crate::webidl::throw_type_error(s, "callback is required");
        return;
    };
    match super::lock::create(s, name, "exclusive".to_owned()) {
        Ok(lock) => {
            let value = callback
                .call(s, v8::undefined(s).into(), &[lock.into()])
                .unwrap_or_else(|| v8::undefined(s).into());
            promise(s, value, r)
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<LockManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
