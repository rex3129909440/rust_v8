use std::collections::HashMap;
#[derive(Clone)]
struct ServerRecord {
    device: v8::Global<v8::Object>,
    connected: bool,
}
#[derive(Default)]
pub(crate) struct BluetoothRemoteGattServerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ServerRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(BluetoothRemoteGattServerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "BluetoothRemoteGATTServer", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<BluetoothRemoteGattServerStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "BluetoothRemoteGATTServer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "device", device)?;
    crate::webidl::define_readonly_accessor(s, p, "connected", connected)?;
    crate::webidl::define_method(s, p, "connect", 0, connect)?;
    crate::webidl::define_method(s, p, "disconnect", 0, disconnect)?;
    crate::webidl::define_method(s, p, "getPrimaryService", 1, get_primary_service)?;
    crate::webidl::define_method(s, p, "getPrimaryServices", 0, get_primary_services)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<BluetoothRemoteGattServerStore>()
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
    device: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create BluetoothRemoteGATTServer".to_owned());
    }
    let device = v8::Global::new(s, device);
    s.get_slot_mut::<BluetoothRemoteGattServerStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            ServerRecord {
                device,
                connected: false,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ServerRecord> {
    s.get_slot::<BluetoothRemoteGattServerStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn device(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.device).into())
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
fn connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<BluetoothRemoteGattServerStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.connected = true;
        resolve(s, a.this().into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTServer",
            "connect",
            r,
        )
    }
}
fn disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<BluetoothRemoteGattServerStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.connected = false
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn service<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let v = record(s, a.this()).ok_or_else(|| "Illegal invocation".to_owned())?;
    let d = v8::Local::new(s, &v.device);
    let uuid = if a.length() > 0 {
        crate::webidl::value_to_string(s, a.get(0))
    } else {
        "generic_access".to_owned()
    };
    super::bluetooth_remote_gatt_service::create(s, d, uuid, true)
}
fn get_primary_service<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTServer",
            "getPrimaryService",
            r,
        );
        return;
    }
    match service(s, a) {
        Ok(v) => resolve(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn get_primary_services<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTServer",
            "getPrimaryServices",
            r,
        );
        return;
    }
    match service(s, a) {
        Ok(v) => {
            let array = v8::Array::new(s, 1);
            let _ = array.set_index(s, 0, v.into());
            resolve(s, array.into(), r)
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
