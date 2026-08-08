use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct NavigatorLoginStore {
    constructor: crate::webidl::RealmConstructor,
    status: HashMap<i32, String>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(NavigatorLoginStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "NavigatorLogin", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<NavigatorLoginStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "NavigatorLogin",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "setStatus", 1, set_status)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<NavigatorLoginStore>()
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
        return Err("cannot create NavigatorLogin".to_owned());
    }
    s.get_slot_mut::<NavigatorLoginStore>()
        .unwrap()
        .status
        .insert(o.get_identity_hash().get(), "logged-out".to_owned());
    Ok(o)
}
fn set_status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let status = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<NavigatorLoginStore>()
        .and_then(|x| x.status.get_mut(&a.this().get_identity_hash().get()))
    {
        *v = status;
        let x = v8::undefined(s);
        if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
            r.set(p.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
