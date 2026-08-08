use std::collections::HashMap;
#[derive(Clone)]
struct InterfaceData {
    number: u32,
    alternate: v8::Global<v8::Object>,
    alternates: v8::Global<v8::Array>,
    claimed: bool,
}
#[derive(Default)]
pub(crate) struct UsbInterfaceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, InterfaceData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbInterfaceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBInterface", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbInterfaceStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBInterface",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "interfaceNumber", number)?;
    crate::webidl::define_readonly_accessor(s, p, "alternate", alternate)?;
    crate::webidl::define_readonly_accessor(s, p, "alternates", alternates)?;
    crate::webidl::define_readonly_accessor(s, p, "claimed", claimed)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbInterfaceStore>()
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
    match create_data(s, a.get(1).uint32_value(s).unwrap_or(0), false) {
        Ok(v) => {
            s.get_slot_mut::<UsbInterfaceStore>()
                .unwrap()
                .records
                .insert(a.this().get_identity_hash().get(), v);
            r.set(a.this().into())
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn create_data(
    s: &mut v8::PinScope<'_, '_>,
    number: u32,
    claimed: bool,
) -> Result<InterfaceData, String> {
    let alternate = super::usb_alternate_interface::create(s, 0)?;
    let array = v8::Array::new(s, 1);
    let _ = array.set_index(s, 0, alternate.into());
    Ok(InterfaceData {
        number,
        alternate: v8::Global::new(s, alternate),
        alternates: v8::Global::new(s, array),
        claimed,
    })
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    number: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBInterface".to_owned());
    }
    let data = create_data(s, number, false)?;
    s.get_slot_mut::<UsbInterfaceStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), data);
    Ok(o)
}
pub(crate) fn set_claimed(s: &mut v8::PinScope<'_, '_>, number: u32, value: bool) {
    if let Some(store) = s.get_slot_mut::<UsbInterfaceStore>() {
        for item in store.records.values_mut() {
            if item.number == number {
                item.claimed = value
            }
        }
    }
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<InterfaceData> {
    s.get_slot::<UsbInterfaceStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.number).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn alternate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.alternate).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn alternates(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.alternates).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn claimed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.claimed).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbInterfaceStore>() {
        store.constructor.remove(realm_id);
    }
}
