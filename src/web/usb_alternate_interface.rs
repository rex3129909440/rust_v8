use std::collections::HashMap;
#[derive(Clone)]
struct AlternateData {
    setting: u32,
    class: u32,
    subclass: u32,
    protocol: u32,
    name: Option<String>,
    endpoints: v8::Global<v8::Array>,
}
#[derive(Default)]
pub(crate) struct UsbAlternateInterfaceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AlternateData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbAlternateInterfaceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBAlternateInterface", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbAlternateInterfaceStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBAlternateInterface",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "alternateSetting", setting)?;
    crate::webidl::define_readonly_accessor(s, p, "interfaceClass", class)?;
    crate::webidl::define_readonly_accessor(s, p, "interfaceSubclass", subclass)?;
    crate::webidl::define_readonly_accessor(s, p, "interfaceProtocol", protocol)?;
    crate::webidl::define_readonly_accessor(s, p, "interfaceName", name)?;
    crate::webidl::define_readonly_accessor(s, p, "endpoints", endpoints)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbAlternateInterfaceStore>()
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
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(s, "2 arguments required");
        return;
    }
    if a.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'USBAlternateInterface': parameter 1 is not of type 'USBInterface'.",
        );
        return;
    }
    let valid_interface = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .is_some_and(|object| {
            super::structured_clone::inherits_platform_interface(s, object, "USBInterface")
        });
    if !valid_interface {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'USBAlternateInterface': parameter 1 is not of type 'USBInterface'.",
        );
        return;
    }
    let setting = a.get(1).uint32_value(s).unwrap_or(0);
    let endpoints = v8::Array::new(s, 0);
    let endpoints = v8::Global::new(s, endpoints);
    s.get_slot_mut::<UsbAlternateInterfaceStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            AlternateData {
                setting,
                class: 255,
                subclass: 0,
                protocol: 0,
                name: None,
                endpoints,
            },
        );
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    setting: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBAlternateInterface".to_owned());
    }
    let endpoint = super::usb_endpoint::create(s, 1, "in".to_owned())?;
    let endpoints = v8::Array::new(s, 1);
    let _ = endpoints.set_index(s, 0, endpoint.into());
    let endpoints = v8::Global::new(s, endpoints);
    s.get_slot_mut::<UsbAlternateInterfaceStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            AlternateData {
                setting,
                class: 255,
                subclass: 0,
                protocol: 0,
                name: Some("Edge Sandbox USB".to_owned()),
                endpoints,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<AlternateData> {
    s.get_slot::<UsbAlternateInterfaceStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn uint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(AlternateData) -> u32,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, f(v)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn setting(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    uint(s, a, r, |v| v.setting)
}
fn class(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    uint(s, a, r, |v| v.class)
}
fn subclass(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    uint(s, a, r, |v| v.subclass)
}
fn protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    uint(s, a, r, |v| v.protocol)
}
fn name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(x) = v.name.and_then(|x| v8::String::new(s, &x)) {
            r.set(x.into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn endpoints(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.endpoints).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbAlternateInterfaceStore>() {
        store.constructor.remove(realm_id);
    }
}
