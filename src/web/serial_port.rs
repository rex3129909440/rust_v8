use std::collections::HashMap;
#[derive(Clone)]
struct PortData {
    connected: bool,
    opened: bool,
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
    on_connect: Option<v8::Global<v8::Value>>,
    on_disconnect: Option<v8::Global<v8::Value>>,
    vendor: u32,
    product: u32,
}
#[derive(Default)]
pub(crate) struct SerialPortStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PortData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SerialPortStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "SerialPort", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<SerialPortStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "SerialPort",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "onconnect", get_connect, set_connect)?;
    crate::webidl::define_accessor(s, p, "ondisconnect", get_disconnect, set_disconnect)?;
    crate::webidl::define_readonly_accessor(s, p, "readable", readable)?;
    crate::webidl::define_readonly_accessor(s, p, "writable", writable)?;
    crate::webidl::define_method(s, p, "close", 0, close)?;
    crate::webidl::define_method(s, p, "forget", 0, forget)?;
    crate::webidl::define_method(s, p, "getInfo", 0, get_info)?;
    crate::webidl::define_method(s, p, "getSignals", 0, get_signals)?;
    crate::webidl::define_method(s, p, "open", 1, open)?;
    crate::webidl::define_method(s, p, "setSignals", 0, set_signals)?;
    crate::webidl::define_readonly_accessor(s, p, "connected", connected)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SerialPortStore>()
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
    profile: crate::SerialPortFingerprint,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create SerialPort".to_owned());
    }
    super::event_target::attach(s, o);
    let readable_object = super::readable_stream::create_empty(s)?;
    let readable = v8::Global::new(s, readable_object);
    let writable_object = super::writable_stream::create_empty(s)?;
    let writable = v8::Global::new(s, writable_object);
    s.get_slot_mut::<SerialPortStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        PortData {
            connected: profile.connected,
            opened: false,
            readable,
            writable,
            on_connect: None,
            on_disconnect: None,
            vendor: profile.usb_vendor_id as u32,
            product: profile.usb_product_id as u32,
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<PortData> {
    s.get_slot::<SerialPortStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn get_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.on_connect),
        r,
    )
}
fn get_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.on_disconnect),
        r,
    )
}
fn set_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<SerialPortStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_connect = h
    }
}
fn set_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<SerialPortStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_disconnect = h
    }
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
fn connected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.connected).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    value: bool,
) {
    if let Some(v) = s
        .get_slot_mut::<SerialPortStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.opened = value;
        let x = v8::undefined(s);
        promise(s, x.into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_open(s, a, r, true)
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_open(s, a, r, false)
}
fn forget(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<SerialPortStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.connected = false;
        let x = v8::undefined(s);
        promise(s, x.into(), r)
    }
}
fn get_info(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let o = v8::Object::new(s);
    if let Some(k) = v8::String::new(s, "usbVendorId") {
        let _ = o.set(
            s,
            k.into(),
            v8::Integer::new_from_unsigned(s, v.vendor).into(),
        );
    }
    if let Some(k) = v8::String::new(s, "usbProductId") {
        let _ = o.set(
            s,
            k.into(),
            v8::Integer::new_from_unsigned(s, v.product).into(),
        );
    }
    r.set(o.into())
}
fn get_signals(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let o = v8::Object::new(s);
    promise(s, o.into(), r)
}
fn set_signals(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let x = v8::undefined(s);
    promise(s, x.into(), r)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SerialPortStore>() {
        store.constructor.remove(realm_id);
    }
}
