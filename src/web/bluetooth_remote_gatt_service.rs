use std::collections::HashMap;
#[derive(Clone)]
struct ServiceRecord {
    device: v8::Global<v8::Object>,
    uuid: String,
    primary: bool,
}
#[derive(Default)]
pub(crate) struct BluetoothRemoteGattServiceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ServiceRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(BluetoothRemoteGattServiceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "BluetoothRemoteGATTService", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<BluetoothRemoteGattServiceStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "BluetoothRemoteGATTService",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "device", device)?;
    crate::webidl::define_readonly_accessor(s, p, "uuid", uuid)?;
    crate::webidl::define_readonly_accessor(s, p, "isPrimary", primary)?;
    crate::webidl::define_method(s, p, "getCharacteristic", 1, get_characteristic)?;
    crate::webidl::define_method(s, p, "getCharacteristics", 0, get_characteristics)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<BluetoothRemoteGattServiceStore>()
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
    uuid: String,
    primary: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create BluetoothRemoteGATTService".to_owned());
    }
    let device = v8::Global::new(s, device);
    s.get_slot_mut::<BluetoothRemoteGattServiceStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            ServiceRecord {
                device,
                uuid,
                primary,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ServiceRecord> {
    s.get_slot::<BluetoothRemoteGattServiceStore>()?
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
fn uuid(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.uuid)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn primary(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.primary).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn characteristic<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if record(s, a.this()).is_none() {
        return Err("Illegal invocation".to_owned());
    }
    let uuid = if a.length() > 0 {
        crate::webidl::value_to_string(s, a.get(0))
    } else {
        "2a00".to_owned()
    };
    super::bluetooth_remote_gatt_characteristic::create(s, a.this(), uuid)
}
fn get_characteristic<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    match characteristic(s, a) {
        Ok(v) => resolve(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn get_characteristics<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    match characteristic(s, a) {
        Ok(v) => {
            let array = v8::Array::new(s, 1);
            let _ = array.set_index(s, 0, v.into());
            resolve(s, array.into(), r)
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
