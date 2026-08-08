use std::collections::HashMap;
#[derive(Clone)]
struct ResultData {
    bytes: Vec<u8>,
    packets: v8::Global<v8::Array>,
}
#[derive(Default)]
pub(crate) struct UsbIsochronousInTransferResultStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ResultData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbIsochronousInTransferResultStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBIsochronousInTransferResult", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbIsochronousInTransferResultStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBIsochronousInTransferResult",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "data", data)?;
    crate::webidl::define_readonly_accessor(s, p, "packets", packets)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbIsochronousInTransferResultStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "use new");
        return;
    }
    let array = v8::Array::new(s, 0);
    let packets = v8::Global::new(s, array);
    s.get_slot_mut::<UsbIsochronousInTransferResultStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            ResultData {
                bytes: Vec::new(),
                packets,
            },
        );
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    bytes: Vec<u8>,
    packet: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBIsochronousInTransferResult".to_owned());
    }
    let array = v8::Array::new(s, 1);
    let _ = array.set_index(s, 0, packet.into());
    let packets = v8::Global::new(s, array);
    s.get_slot_mut::<UsbIsochronousInTransferResultStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), ResultData { bytes, packets });
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ResultData> {
    s.get_slot::<UsbIsochronousInTransferResultStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let len = v.bytes.len();
        let b = v8::ArrayBuffer::new_backing_store_from_vec(v.bytes).make_shared();
        let buffer = v8::ArrayBuffer::with_backing_store(s, &b);
        r.set(v8::DataView::new(s, buffer, 0, len).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn packets(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.packets).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbIsochronousInTransferResultStore>() {
        store.constructor.remove(realm_id);
    }
}
