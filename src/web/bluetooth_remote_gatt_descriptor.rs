use std::collections::HashMap;
#[derive(Clone)]
struct DescriptorRecord {
    characteristic: v8::Global<v8::Object>,
    uuid: String,
    bytes: Vec<u8>,
}
#[derive(Default)]
pub(crate) struct BluetoothRemoteGattDescriptorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DescriptorRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(BluetoothRemoteGattDescriptorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "BluetoothRemoteGATTDescriptor", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<BluetoothRemoteGattDescriptorStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "BluetoothRemoteGATTDescriptor",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "characteristic", characteristic)?;
    crate::webidl::define_readonly_accessor(s, p, "uuid", uuid)?;
    crate::webidl::define_readonly_accessor(s, p, "value", value)?;
    crate::webidl::define_method(s, p, "readValue", 0, read_value)?;
    crate::webidl::define_method(s, p, "writeValue", 1, write_value)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<BluetoothRemoteGattDescriptorStore>()
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
    characteristic: v8::Local<'s, v8::Object>,
    uuid: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create BluetoothRemoteGATTDescriptor".to_owned());
    }
    let characteristic = v8::Global::new(s, characteristic);
    s.get_slot_mut::<BluetoothRemoteGattDescriptorStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            DescriptorRecord {
                characteristic,
                uuid,
                bytes: Vec::new(),
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<DescriptorRecord> {
    s.get_slot::<BluetoothRemoteGattDescriptorStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn data_view<'s>(s: &mut v8::PinScope<'s, '_>, bytes: Vec<u8>) -> v8::Local<'s, v8::DataView> {
    let len = bytes.len();
    let b = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(s, &b);
    v8::DataView::new(s, buffer, 0, len)
}
fn characteristic(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.characteristic).into())
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
            "BluetoothRemoteGATTDescriptor",
            "readValue",
            r,
        )
    }
}
fn write_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTDescriptor",
            "writeValue",
            r,
        );
        return;
    }
    let bytes = crate::webidl::value_to_string(s, a.get(0)).into_bytes();
    if let Some(v) = s
        .get_slot_mut::<BluetoothRemoteGattDescriptorStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.bytes = bytes;
        let x = v8::undefined(s);
        resolve(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "BluetoothRemoteGATTDescriptor",
            "writeValue",
            r,
        )
    }
}
