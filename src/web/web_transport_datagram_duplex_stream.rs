use std::collections::HashMap;
#[derive(Clone)]
struct Datagram {
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
    incoming_age: Option<f64>,
    outgoing_age: Option<f64>,
    incoming_water: f64,
    outgoing_water: f64,
}
#[derive(Default)]
pub(crate) struct WebTransportDatagramDuplexStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Datagram>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(WebTransportDatagramDuplexStreamStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "WebTransportDatagramDuplexStream", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<WebTransportDatagramDuplexStreamStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "WebTransportDatagramDuplexStream",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "readable", readable)?;
    crate::webidl::define_readonly_accessor(s, p, "writable", writable)?;
    crate::webidl::define_readonly_accessor(s, p, "maxDatagramSize", max_size)?;
    crate::webidl::define_accessor(s, p, "incomingMaxAge", incoming_age, set_incoming_age)?;
    crate::webidl::define_accessor(s, p, "outgoingMaxAge", outgoing_age, set_outgoing_age)?;
    crate::webidl::define_readonly_accessor(s, p, "incomingMaxBufferedDatagrams", max_buffered)?;
    crate::webidl::define_readonly_accessor(s, p, "outgoingMaxBufferedDatagrams", max_buffered)?;
    crate::webidl::define_accessor(
        s,
        p,
        "incomingHighWaterMark",
        incoming_water,
        set_incoming_water,
    )?;
    crate::webidl::define_accessor(
        s,
        p,
        "outgoingHighWaterMark",
        outgoing_water,
        set_outgoing_water,
    )?;
    crate::webidl::finish_constructor(s, p, c)?;
    let g = v8::Global::new(s, c);
    s.get_slot_mut::<WebTransportDatagramDuplexStreamStore>()
        .ok_or_else(|| "WebTransportDatagramDuplexStream state missing".to_owned())?
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
        return Err("cannot create WebTransportDatagramDuplexStream".to_owned());
    }
    let read = super::readable_stream::create_empty(s)?;
    let write = super::writable_stream::create_empty(s)?;
    let d = Datagram {
        readable: v8::Global::new(s, read),
        writable: v8::Global::new(s, write),
        incoming_age: None,
        outgoing_age: None,
        incoming_water: 1.0,
        outgoing_water: 1.0,
    };
    s.get_slot_mut::<WebTransportDatagramDuplexStreamStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), d);
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Datagram> {
    s.get_slot::<WebTransportDatagramDuplexStreamStore>()?
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
fn max_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Integer::new(s, 1200).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn max_buffered(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Integer::new(s, 1).into());
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}
fn option(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    pick: impl FnOnce(Datagram) -> Option<f64>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(v) = pick(v) {
            r.set(v8::Number::new(s, v).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    pick: impl FnOnce(Datagram) -> f64,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Number::new(s, pick(v)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn incoming_age(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    option(s, a, r, |v| v.incoming_age)
}
fn outgoing_age(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    option(s, a, r, |v| v.outgoing_age)
}
fn incoming_water(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |v| v.incoming_water)
}
fn outgoing_water(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |v| v.outgoing_water)
}
fn update(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Datagram),
) {
    if let Some(v) = s
        .get_slot_mut::<WebTransportDatagramDuplexStreamStore>()
        .and_then(|x| x.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_incoming_age(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = if a.get(0).is_null() {
        None
    } else {
        a.get(0).number_value(s)
    };
    update(s, a.this(), |x| x.incoming_age = v)
}
fn set_outgoing_age(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = if a.get(0).is_null() {
        None
    } else {
        a.get(0).number_value(s)
    };
    update(s, a.this(), |x| x.outgoing_age = v)
}
fn set_incoming_water(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(s).unwrap_or(1.0);
    update(s, a.this(), |x| x.incoming_water = v)
}
fn set_outgoing_water(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(s).unwrap_or(1.0);
    update(s, a.this(), |x| x.outgoing_water = v)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebTransportDatagramDuplexStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
