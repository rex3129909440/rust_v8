use std::collections::HashMap;
#[derive(Clone)]
struct ConfigurationData {
    value: u32,
    name: Option<String>,
    interfaces: v8::Global<v8::Array>,
}
#[derive(Default)]
pub(crate) struct UsbConfigurationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ConfigurationData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbConfigurationStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBConfiguration", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbConfigurationStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBConfiguration",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "configurationValue", value)?;
    crate::webidl::define_readonly_accessor(s, p, "configurationName", name)?;
    crate::webidl::define_readonly_accessor(s, p, "interfaces", interfaces)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbConfigurationStore>()
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
    match create_data(s, a.get(1).uint32_value(s).unwrap_or(1)) {
        Ok(v) => {
            s.get_slot_mut::<UsbConfigurationStore>()
                .unwrap()
                .records
                .insert(a.this().get_identity_hash().get(), v);
            r.set(a.this().into())
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn create_data(s: &mut v8::PinScope<'_, '_>, value: u32) -> Result<ConfigurationData, String> {
    let interface = super::usb_interface::create(s, 0)?;
    let array = v8::Array::new(s, 1);
    let _ = array.set_index(s, 0, interface.into());
    Ok(ConfigurationData {
        value,
        name: Some("Default".to_owned()),
        interfaces: v8::Global::new(s, array),
    })
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    value: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBConfiguration".to_owned());
    }
    let data = create_data(s, value)?;
    s.get_slot_mut::<UsbConfigurationStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), data);
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ConfigurationData> {
    s.get_slot::<UsbConfigurationStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.value).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
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
fn interfaces(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.interfaces).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbConfigurationStore>() {
        store.constructor.remove(realm_id);
    }
}
