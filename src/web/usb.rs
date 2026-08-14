use std::collections::HashMap;
#[derive(Clone, Default)]
struct UsbData {
    on_connect: Option<v8::Global<v8::Value>>,
    on_disconnect: Option<v8::Global<v8::Value>>,
    devices: Vec<v8::Global<v8::Object>>,
}
#[derive(Default)]
pub(crate) struct UsbStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, UsbData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USB", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(s, "USB", 0, v8::ConstructorBehavior::Allow, illegal)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "onconnect", get_connect, set_connect)?;
    crate::webidl::define_accessor(s, p, "ondisconnect", get_disconnect, set_disconnect)?;
    crate::webidl::define_method(s, p, "getDevices", 0, get_devices)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let tag = v8::Symbol::get_to_string_tag(s);
    let _ = p.delete(s, tag.into());
    crate::webidl::define_method(s, p, "requestDevice", 1, request_device)?;
    crate::webidl::define_to_string_tag(s, p, "USB")?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbStore>()
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
        return Err("cannot create USB".to_owned());
    }
    super::event_target::attach(s, o);
    let configured = crate::fingerprint::edge(s)
        .hardware_devices
        .usb_devices
        .clone();
    let mut devices = Vec::with_capacity(configured.len());
    for profile in configured {
        let device = super::usb_device::create(s, profile)?;
        devices.push(v8::Global::new(s, device));
    }
    s.get_slot_mut::<UsbStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        UsbData {
            devices,
            ..UsbData::default()
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<UsbData> {
    s.get_slot::<UsbStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn reject_illegal_invocation(
    s: &mut v8::PinScope<'_, '_>,
    method: &str,
    mut r: v8::ReturnValue<'_>,
) {
    let message = format!("Failed to execute '{method}' on 'USB': Illegal invocation");
    if let Some(promise) = crate::webidl::rejected_type_error_promise(s, &message) {
        r.set(promise.into());
    }
}
fn get_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.on_connect, r)
}
fn get_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.on_disconnect, r)
}
fn set_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<UsbStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_connect = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<UsbStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_disconnect = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_devices(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        reject_illegal_invocation(s, "getDevices", r);
        return;
    };
    let array = v8::Array::new(s, v.devices.len() as i32);
    for (i, item) in v.devices.iter().enumerate() {
        let x = v8::Local::new(s, item);
        let _ = array.set_index(s, i as u32, x.into());
    }
    promise(s, array.into(), r)
}
fn request_device(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        reject_illegal_invocation(s, "requestDevice", r);
        return;
    }
    let profile = crate::fingerprint::edge(s)
        .hardware_devices
        .usb_devices
        .first()
        .cloned()
        .unwrap_or_default();
    match super::usb_device::create(s, profile) {
        Ok(device) => {
            let stored = v8::Global::new(s, device);
            if let Some(v) = s
                .get_slot_mut::<UsbStore>()
                .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
            {
                v.devices.push(stored)
            }
            promise(s, device.into(), r)
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbStore>() {
        store.constructor.remove(realm_id);
    }
}
