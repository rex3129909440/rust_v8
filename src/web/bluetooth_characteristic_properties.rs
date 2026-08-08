use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct CharacteristicFlags {
    pub broadcast: bool,
    pub read: bool,
    pub write_without_response: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
    pub authenticated_signed_writes: bool,
    pub reliable_write: bool,
    pub writable_auxiliaries: bool,
}

#[derive(Default)]
pub(crate) struct BluetoothCharacteristicPropertiesStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CharacteristicFlags>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BluetoothCharacteristicPropertiesStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BluetoothCharacteristicProperties", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<BluetoothCharacteristicPropertiesStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "BluetoothCharacteristicProperties",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "broadcast", broadcast)?;
    crate::webidl::define_readonly_accessor(s, p, "read", read)?;
    crate::webidl::define_readonly_accessor(s, p, "writeWithoutResponse", write_without_response)?;
    crate::webidl::define_readonly_accessor(s, p, "write", write)?;
    crate::webidl::define_readonly_accessor(s, p, "notify", notify)?;
    crate::webidl::define_readonly_accessor(s, p, "indicate", indicate)?;
    crate::webidl::define_readonly_accessor(
        s,
        p,
        "authenticatedSignedWrites",
        authenticated_signed_writes,
    )?;
    crate::webidl::define_readonly_accessor(s, p, "reliableWrite", reliable_write)?;
    crate::webidl::define_readonly_accessor(s, p, "writableAuxiliaries", writable_auxiliaries)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<BluetoothCharacteristicPropertiesStore>()
        .ok_or_else(|| "BluetoothCharacteristicProperties state missing".to_owned())?
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
    flags: CharacteristicFlags,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create BluetoothCharacteristicProperties".to_owned());
    }
    s.get_slot_mut::<BluetoothCharacteristicPropertiesStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), flags);
    Ok(o)
}
fn get(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<CharacteristicFlags> {
    s.get_slot::<BluetoothCharacteristicPropertiesStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(CharacteristicFlags) -> bool,
) {
    if let Some(v) = get(s, a.this()) {
        r.set(v8::Boolean::new(s, f(v)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn broadcast(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.broadcast)
}
fn read(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.read)
}
fn write_without_response(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.write_without_response)
}
fn write(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.write)
}
fn notify(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.notify)
}
fn indicate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.indicate)
}
fn authenticated_signed_writes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.authenticated_signed_writes)
}
fn reliable_write(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.reliable_write)
}
fn writable_auxiliaries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    value(s, a, r, |v| v.writable_auxiliaries)
}
