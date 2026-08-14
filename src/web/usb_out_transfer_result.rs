use std::collections::HashMap;
#[derive(Clone)]
struct OutData {
    written: u32,
    status: String,
}
#[derive(Default)]
pub(crate) struct UsbOutTransferResultStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, OutData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbOutTransferResultStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBOutTransferResult", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbOutTransferResultStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBOutTransferResult",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "bytesWritten", written)?;
    crate::webidl::define_readonly_accessor(s, p, "status", status)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbOutTransferResultStore>()
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
    if a.get(0).is_null_or_undefined() {
        let status = crate::webidl::value_to_string(s, a.get(0));
        crate::webidl::throw_type_error(
            s,
            &format!(
                "Failed to construct 'USBOutTransferResult': The provided value '{status}' is not a valid enum value of type USBTransferStatus."
            ),
        );
        return;
    }
    let Some(status) = crate::webidl::dom_string(s, a.get(0)) else {
        return;
    };
    if !matches!(status.as_str(), "ok" | "stall" | "babble") {
        crate::webidl::throw_type_error(
            s,
            &format!(
                "Failed to construct 'USBOutTransferResult': The provided value '{status}' is not a valid enum value of type USBTransferStatus."
            ),
        );
        return;
    }
    s.get_slot_mut::<UsbOutTransferResultStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            OutData { written: 0, status },
        );
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    written: u32,
    status: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBOutTransferResult".to_owned());
    }
    s.get_slot_mut::<UsbOutTransferResultStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), OutData { written, status });
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<OutData> {
    s.get_slot::<UsbOutTransferResultStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn written(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.written).into())
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
    if let Some(store) = scope.get_slot_mut::<UsbOutTransferResultStore>() {
        store.constructor.remove(realm_id);
    }
}
