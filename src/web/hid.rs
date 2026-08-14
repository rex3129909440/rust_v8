use std::collections::HashMap;
#[derive(Clone, Default)]
struct HidRecord {
    on_connect: Option<v8::Global<v8::Value>>,
    on_disconnect: Option<v8::Global<v8::Value>>,
    devices: Vec<v8::Global<v8::Object>>,
}
#[derive(Default)]
pub(crate) struct HidStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, HidRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(HidStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "HID", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<HidStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(s, "HID", 0, v8::ConstructorBehavior::Allow, illegal)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "onconnect", get_connect, set_connect)?;
    crate::webidl::define_accessor(s, p, "ondisconnect", get_disconnect, set_disconnect)?;
    crate::webidl::define_method(s, p, "getDevices", 0, get_devices)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let tag = v8::Symbol::get_to_string_tag(s);
    let _ = p.delete(s, tag.into());
    crate::webidl::define_method(s, p, "requestDevice", 1, request_device)?;
    crate::webidl::define_to_string_tag(s, p, "HID")?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<HidStore>()
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
        return Err("cannot create HID".to_owned());
    }
    super::event_target::attach(s, o);
    let configured = crate::fingerprint::edge(s)
        .hardware_devices
        .hid_devices
        .clone();
    let mut devices = Vec::with_capacity(configured.len());
    for profile in configured {
        let device = super::hid_device::create(
            s,
            profile.vendor_id as u32,
            profile.product_id as u32,
            profile.product_name,
        )?;
        devices.push(v8::Global::new(s, device));
    }
    s.get_slot_mut::<HidStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        HidRecord {
            devices,
            ..HidRecord::default()
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<HidRecord> {
    s.get_slot::<HidStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
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
        .get_slot_mut::<HidStore>()
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
        .get_slot_mut::<HidStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_disconnect = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn devices_array<'s>(
    s: &mut v8::PinScope<'s, '_>,
    values: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(s, values.len() as i32);
    for (i, v) in values.iter().enumerate() {
        let x = v8::Local::new(s, v);
        let _ = array.set_index(s, i as u32, x.into());
    }
    array
}
fn get_devices(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let array = devices_array(s, &v.devices);
        resolve(s, array.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "HID", "getDevices", r)
    }
}
fn request_device(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "HID", "requestDevice", r);
        return;
    }
    let profile = crate::fingerprint::edge(s)
        .hardware_devices
        .hid_devices
        .first()
        .cloned()
        .unwrap_or_default();
    match super::hid_device::create(
        s,
        profile.vendor_id as u32,
        profile.product_id as u32,
        profile.product_name,
    ) {
        Ok(device) => {
            let stored = v8::Global::new(s, device);
            if let Some(v) = s
                .get_slot_mut::<HidStore>()
                .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
            {
                v.devices.push(stored);
            }
            let array = v8::Array::new(s, 1);
            let _ = array.set_index(s, 0, device.into());
            resolve(s, array.into(), r)
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<HidStore>() {
        store.constructor.remove(realm_id);
    }
}
