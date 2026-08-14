use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct PaymentManagerStore {
    constructor: crate::webidl::RealmConstructor,
    hints: HashMap<i32, String>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PaymentManagerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PaymentManager", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PaymentManagerStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PaymentManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "userHint", get_hint, set_hint)?;
    crate::webidl::define_method(s, p, "enableDelegations", 1, enable)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PaymentManagerStore>()
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
        return Err("cannot create PaymentManager".to_owned());
    }
    s.get_slot_mut::<PaymentManagerStore>()
        .unwrap()
        .hints
        .insert(o.get_identity_hash().get(), String::new());
    Ok(o)
}
fn get_hint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot::<PaymentManagerStore>()
        .and_then(|x| x.hints.get(&a.this().get_identity_hash().get()))
        && let Some(x) = v8::String::new(s, v)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_hint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let hint = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<PaymentManagerStore>()
        .and_then(|x| x.hints.get_mut(&a.this().get_identity_hash().get()))
    {
        *v = hint
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn enable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if s.get_slot::<PaymentManagerStore>()
        .is_some_and(|x| x.hints.contains_key(&a.this().get_identity_hash().get()))
    {
        let x = v8::undefined(s);
        if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
            r.set(p.into())
        }
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "PaymentManager",
            "enableDelegations",
            r,
        )
    }
}
