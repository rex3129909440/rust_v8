use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct PresentationReceiverStore {
    constructor: crate::webidl::RealmConstructor,
    lists: HashMap<i32, v8::Global<v8::Object>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PresentationReceiverStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PresentationReceiver", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PresentationReceiverStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PresentationReceiver",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "connectionList", connection_list)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PresentationReceiverStore>()
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
        return Err("cannot create PresentationReceiver".to_owned());
    }
    let list = super::presentation_connection_list::create(s, &[])?;
    let list = v8::Global::new(s, list);
    s.get_slot_mut::<PresentationReceiverStore>()
        .unwrap()
        .lists
        .insert(o.get_identity_hash().get(), list);
    Ok(o)
}
fn connection_list(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot::<PresentationReceiverStore>()
        .and_then(|x| x.lists.get(&a.this().get_identity_hash().get()))
        .cloned()
    {
        let x = v8::Local::new(s, &v);
        if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
            r.set(p.into())
        }
    } else {
        if let Some(promise) = crate::webidl::rejected_type_error_promise(
            s,
            "Failed to read the 'connectionList' property from 'PresentationReceiver': Illegal invocation",
        ) {
            r.set(promise.into())
        }
    }
}
