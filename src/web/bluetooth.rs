use std::collections::HashMap;

#[derive(Clone, Default)]
struct BluetoothRecord {
    on_availability_changed: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct BluetoothStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BluetoothRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BluetoothStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Bluetooth", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<BluetoothStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Bluetooth",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getAvailability", 0, get_availability)?;
    crate::webidl::define_method(scope, prototype, "requestDevice", 0, request_device)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BluetoothStore>()
        .ok_or_else(|| "Bluetooth state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Bluetooth".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<BluetoothStore>()
        .ok_or_else(|| "Bluetooth state missing".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), BluetoothRecord::default());
    Ok(object)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<BluetoothStore>()
        .is_some_and(|v| v.records.contains_key(&o.get_identity_hash().get()))
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn get_availability(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "Bluetooth", "getAvailability", r);
        return;
    }
    let available = crate::fingerprint::edge(s)
        .hardware_devices
        .bluetooth_available;
    let v = v8::Boolean::new(s, available);
    resolve(s, v.into(), r)
}
fn request_device(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "Bluetooth", "requestDevice", r);
        return;
    }
    let Some(profile) = crate::fingerprint::edge(s)
        .hardware_devices
        .bluetooth_devices
        .first()
        .cloned()
    else {
        let null = v8::null(s);
        resolve(s, null.into(), r);
        return;
    };
    match super::bluetooth_device::create(s, profile.id, profile.name) {
        Ok(v) => resolve(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
