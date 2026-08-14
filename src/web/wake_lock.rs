use std::collections::HashSet;
#[derive(Default)]
pub(crate) struct WakeLockStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(WakeLockStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "WakeLock", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<WakeLockStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c =
        crate::webidl::create_function(s, "WakeLock", 0, v8::ConstructorBehavior::Allow, illegal)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "request", 0, request)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<WakeLockStore>()
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
        return Err("cannot create WakeLock".to_owned());
    }
    s.get_slot_mut::<WakeLockStore>()
        .unwrap()
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn request(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !s
        .get_slot::<WakeLockStore>()
        .is_some_and(|x| x.instances.contains(&a.this().get_identity_hash().get()))
    {
        crate::webidl::reject_illegal_invocation_promise(s, "WakeLock", "request", r);
        return;
    }
    let kind = if a.get(0).is_undefined() {
        "screen".to_owned()
    } else {
        crate::webidl::value_to_string(s, a.get(0))
    };
    match super::wake_lock_sentinel::create(s, kind) {
        Ok(v) => {
            if let Ok(p) = super::writable_stream::resolved_promise(s, v.into()) {
                r.set(p.into())
            }
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
