use std::collections::HashMap;
#[derive(Clone)]
struct InData {
    bytes: Vec<u8>,
    status: String,
}
#[derive(Default)]
pub(crate) struct UsbInTransferResultStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, InData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbInTransferResultStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBInTransferResult", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbInTransferResultStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBInTransferResult",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "data", data)?;
    crate::webidl::define_readonly_accessor(s, p, "status", status)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbInTransferResultStore>()
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
    let status = crate::webidl::value_to_string(s, a.get(0));
    s.get_slot_mut::<UsbInTransferResultStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            InData {
                bytes: Vec::new(),
                status,
            },
        );
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    bytes: Vec<u8>,
    status: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBInTransferResult".to_owned());
    }
    s.get_slot_mut::<UsbInTransferResultStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), InData { bytes, status });
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<InData> {
    s.get_slot::<UsbInTransferResultStore>()?
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
fn status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.status)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbInTransferResultStore>() {
        store.constructor.remove(realm_id);
    }
}
