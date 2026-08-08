use std::collections::HashMap;
#[derive(Clone)]
struct Pair {
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
}
#[derive(Default)]
pub(crate) struct WebTransportBidirectionalStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Pair>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(WebTransportBidirectionalStreamStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "WebTransportBidirectionalStream", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<WebTransportBidirectionalStreamStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "WebTransportBidirectionalStream",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "readable", readable)?;
    crate::webidl::define_readonly_accessor(s, p, "writable", writable)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let g = v8::Global::new(s, c);
    s.get_slot_mut::<WebTransportBidirectionalStreamStore>()
        .ok_or_else(|| "WebTransportBidirectionalStream state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
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
        return Err("cannot create WebTransportBidirectionalStream".to_owned());
    }
    let read = super::readable_stream::create_empty(s)?;
    let write = super::writable_stream::create_empty(s)?;
    let pair = Pair {
        readable: v8::Global::new(s, read),
        writable: v8::Global::new(s, write),
    };
    s.get_slot_mut::<WebTransportBidirectionalStreamStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), pair);
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Pair> {
    s.get_slot::<WebTransportBidirectionalStreamStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn readable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.readable).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn writable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.writable).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebTransportBidirectionalStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
