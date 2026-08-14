use std::collections::HashMap;
#[derive(Clone)]
struct CharacteristicRecord {
    service: v8::Global<v8::Object>,
    uuid: String,
    properties: v8::Global<v8::Object>,
    bytes: Vec<u8>,
    handler: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct BluetoothRemoteGattCharacteristicStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CharacteristicRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(BluetoothRemoteGattCharacteristicStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "BluetoothRemoteGATTCharacteristic", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<BluetoothRemoteGattCharacteristicStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "BluetoothRemoteGATTCharacteristic",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "service", service)?;
    crate::webidl::define_readonly_accessor(s, p, "uuid", uuid)?;
    crate::webidl::define_readonly_accessor(s, p, "properties", properties)?;
    crate::webidl::define_readonly_accessor(s, p, "value", value)?;
    crate::webidl::define_accessor(
        s,
        p,
        "oncharacteristicvaluechanged",
        get_handler,
        set_handler,
    )?;
    crate::webidl::define_method(s, p, "getDescriptor", 1, get_descriptor)?;
    crate::webidl::define_method(s, p, "getDescriptors", 0, get_descriptors)?;
    crate::webidl::define_method(s, p, "readValue", 0, read_value)?;
    crate::webidl::define_method(s, p, "startNotifications", 0, start_notifications)?;
    crate::webidl::define_method(s, p, "stopNotifications", 0, stop_notifications)?;
    crate::webidl::define_method(s, p, "writeValue", 1, write_value)?;
    crate::webidl::define_method(s, p, "writeValueWithResponse", 1, write_value_with_response)?;
    crate::webidl::define_method(
        s,
        p,
        "writeValueWithoutResponse",
        1,
        write_value_without_response,
    )?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<BluetoothRemoteGattCharacteristicStore>()
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
    service: v8::Local<'s, v8::Object>,
    uuid: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create BluetoothRemoteGATTCharacteristic".to_owned());
    }
    super::event_target::attach(s, o);
    let flags = super::bluetooth_characteristic_properties::CharacteristicFlags {
        read: true,
        write: true,
        notify: true,
        ..Default::default()
    };
    let properties = super::bluetooth_characteristic_properties::create(s, flags)?;
    let service = v8::Global::new(s, service);
    let properties = v8::Global::new(s, properties);
    s.get_slot_mut::<BluetoothRemoteGattCharacteristicStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            CharacteristicRecord {
                service,
                uuid,
                properties,
                bytes: Vec::new(),
                handler: None,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<CharacteristicRecord> {
    s.get_slot::<BluetoothRemoteGattCharacteristicStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn service(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.service).into())
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
fn properties(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.properties).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn data_view<'s>(s: &mut v8::PinScope<'s, '_>, bytes: Vec<u8>) -> v8::Local<'s, v8::DataView> {
    let len = bytes.len();
    let b = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(s, &b);
    v8::DataView::new(s, buffer, 0, len)
}
fn value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let x = data_view(s, v.bytes);
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.handler, r)
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<BluetoothRemoteGattCharacteristicStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn descriptor<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if record(s, a.this()).is_none() {
        return Err("Illegal invocation".to_owned());
    }
    let uuid = if a.length() > 0 {
        crate::webidl::value_to_string(s, a.get(0))
    } else {
        "2901".to_owned()
    };
    super::bluetooth_remote_gatt_descriptor::create(s, a.this(), uuid)
}
fn get_descriptor<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTCharacteristic",
            "getDescriptor",
            r,
        );
        return;
    }
    match descriptor(s, a) {
        Ok(v) => resolve(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn get_descriptors<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTCharacteristic",
            "getDescriptors",
            r,
        );
        return;
    }
    match descriptor(s, a) {
        Ok(v) => {
            let array = v8::Array::new(s, 1);
            let _ = array.set_index(s, 0, v.into());
            resolve(s, array.into(), r)
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn read_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let x = data_view(s, v.bytes);
        resolve(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTCharacteristic",
            "readValue",
            r,
        )
    }
}
fn self_promise(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    method: &str,
) {
    if record(s, a.this()).is_some() {
        resolve(s, a.this().into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTCharacteristic",
            method,
            r,
        )
    }
}
fn start_notifications(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    self_promise(s, a, r, "startNotifications")
}
fn stop_notifications(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    self_promise(s, a, r, "stopNotifications")
}
fn write_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    write_value_operation(s, a, r, "writeValue")
}
fn write_value_operation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    method: &str,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTCharacteristic",
            method,
            r,
        );
        return;
    }
    let bytes = crate::webidl::value_to_string(s, a.get(0)).into_bytes();
    if let Some(v) = s
        .get_slot_mut::<BluetoothRemoteGattCharacteristicStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.bytes = bytes;
        let x = v8::undefined(s);
        resolve(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTCharacteristic",
            method,
            r,
        )
    }
}
fn write_value_with_response(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    write_value_operation(s, a, r, "writeValueWithResponse")
}
fn write_value_without_response(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    write_value_operation(s, a, r, "writeValueWithoutResponse")
}
