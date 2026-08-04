use std::collections::HashMap;
#[derive(Clone, Default)]
pub(crate) struct AddressRecord {
    city: String,
    country: String,
    dependent: String,
    organization: String,
    phone: String,
    postal: String,
    recipient: String,
    region: String,
    sorting: String,
    lines: Vec<String>,
}
#[derive(Default)]
pub(crate) struct PaymentAddressStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AddressRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PaymentAddressStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PaymentAddress", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PaymentAddressStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PaymentAddress",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "city", city)?;
    crate::webidl::define_readonly_accessor(s, p, "country", country)?;
    crate::webidl::define_readonly_accessor(s, p, "dependentLocality", dependent)?;
    crate::webidl::define_readonly_accessor(s, p, "organization", organization)?;
    crate::webidl::define_readonly_accessor(s, p, "phone", phone)?;
    crate::webidl::define_readonly_accessor(s, p, "postalCode", postal)?;
    crate::webidl::define_readonly_accessor(s, p, "recipient", recipient)?;
    crate::webidl::define_readonly_accessor(s, p, "region", region)?;
    crate::webidl::define_readonly_accessor(s, p, "sortingCode", sorting)?;
    crate::webidl::define_readonly_accessor(s, p, "addressLine", address_line)?;
    crate::webidl::define_method(s, p, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PaymentAddressStore>()
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
    record: AddressRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create PaymentAddress".to_owned());
    }
    s.get_slot_mut::<PaymentAddressStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<AddressRecord> {
    s.get_slot::<PaymentAddressStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(AddressRecord) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn city(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.city)
}
fn country(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.country)
}
fn dependent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.dependent)
}
fn organization(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.organization)
}
fn phone(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.phone)
}
fn postal(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.postal)
}
fn recipient(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.recipient)
}
fn region(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.region)
}
fn sorting(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.sorting)
}
fn address_line(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, v.lines.len() as i32);
    for (i, line) in v.lines.iter().enumerate() {
        if let Some(x) = v8::String::new(s, line) {
            let _ = array.set_index(s, i as u32, x.into());
        }
    }
    r.set(array.into())
}
fn to_json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let o = v8::Object::new(s);
    if let (Some(k), Some(x)) = (v8::String::new(s, "city"), v8::String::new(s, &v.city)) {
        let _ = o.set(s, k.into(), x.into());
    }
    if let (Some(k), Some(x)) = (
        v8::String::new(s, "country"),
        v8::String::new(s, &v.country),
    ) {
        let _ = o.set(s, k.into(), x.into());
    }
    r.set(o.into())
}
