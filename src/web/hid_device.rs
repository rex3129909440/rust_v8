use std::collections::HashMap;
#[derive(Clone)]
struct DeviceRecord {
    opened: bool,
    vendor: u32,
    product: u32,
    name: String,
    collections: v8::Global<v8::Array>,
    handler: Option<v8::Global<v8::Value>>,
    reports: HashMap<u8, Vec<u8>>,
}
#[derive(Default)]
pub(crate) struct HidDeviceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DeviceRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(HidDeviceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "HIDDevice", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<HidDeviceStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c =
        crate::webidl::create_function(s, "HIDDevice", 0, v8::ConstructorBehavior::Allow, illegal)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "oninputreport", get_handler, set_handler)?;
    crate::webidl::define_readonly_accessor(s, p, "opened", opened)?;
    crate::webidl::define_readonly_accessor(s, p, "vendorId", vendor_id)?;
    crate::webidl::define_readonly_accessor(s, p, "productId", product_id)?;
    crate::webidl::define_readonly_accessor(s, p, "productName", product_name)?;
    crate::webidl::define_readonly_accessor(s, p, "collections", collections)?;
    crate::webidl::define_method(s, p, "close", 0, close)?;
    crate::webidl::define_method(s, p, "forget", 0, forget)?;
    crate::webidl::define_method(s, p, "open", 0, open)?;
    crate::webidl::define_method(s, p, "receiveFeatureReport", 1, receive_feature_report)?;
    crate::webidl::define_method(s, p, "sendFeatureReport", 2, send_feature_report)?;
    crate::webidl::define_method(s, p, "sendReport", 2, send_report)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<HidDeviceStore>()
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
    vendor: u32,
    product: u32,
    name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create HIDDevice".to_owned());
    }
    super::event_target::attach(s, o);
    let collections = v8::Array::new(s, 0);
    let collections = v8::Global::new(s, collections);
    s.get_slot_mut::<HidDeviceStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        DeviceRecord {
            opened: false,
            vendor,
            product,
            name,
            collections,
            handler: None,
            reports: HashMap::new(),
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<DeviceRecord> {
    s.get_slot::<HidDeviceStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
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
        .get_slot_mut::<HidDeviceStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn opened(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.opened).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn vendor_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.vendor).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn product_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.product).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn product_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.name)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn collections(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.collections).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_opened(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    method: &str,
    value: bool,
) {
    if let Some(v) = s
        .get_slot_mut::<HidDeviceStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.opened = value;
        let x = v8::undefined(s);
        resolve(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "HIDDevice", method, r)
    }
}
fn open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_opened(s, a, r, "open", true)
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_opened(s, a, r, "close", false)
}
fn forget(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_opened(s, a, r, "forget", false)
}
fn data_view<'s>(s: &mut v8::PinScope<'s, '_>, bytes: Vec<u8>) -> v8::Local<'s, v8::DataView> {
    let len = bytes.len();
    let b = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(s, &b);
    v8::DataView::new(s, buffer, 0, len)
}
fn receive_feature_report(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "HIDDevice", "receiveFeatureReport", r);
        return;
    }
    let id = a.get(0).uint32_value(s).unwrap_or(0) as u8;
    if let Some(v) = record(s, a.this()) {
        let view = data_view(s, v.reports.get(&id).cloned().unwrap_or_default());
        resolve(s, view.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "HIDDevice", "receiveFeatureReport", r)
    }
}
fn send(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    method: &str,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "HIDDevice", method, r);
        return;
    }
    let id = a.get(0).uint32_value(s).unwrap_or(0) as u8;
    let bytes = crate::webidl::value_to_string(s, a.get(1)).into_bytes();
    if let Some(v) = s
        .get_slot_mut::<HidDeviceStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.reports.insert(id, bytes);
        let x = v8::undefined(s);
        resolve(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "HIDDevice", method, r)
    }
}
fn send_feature_report(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    send(s, a, r, "sendFeatureReport")
}
fn send_report(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    send(s, a, r, "sendReport")
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<HidDeviceStore>() {
        store.constructor.remove(realm_id);
    }
}
