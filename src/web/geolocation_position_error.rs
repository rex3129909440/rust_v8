use std::collections::HashMap;
#[derive(Clone)]
struct ErrorRecord {
    code: i32,
    message: String,
}
#[derive(Default)]
pub(crate) struct GeolocationPositionErrorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ErrorRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(GeolocationPositionErrorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "GeolocationPositionError", c.into())
}
fn constants(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Result<(), String> {
    crate::webidl::define_constant(s, o, "PERMISSION_DENIED", 1)?;
    crate::webidl::define_constant(s, o, "POSITION_UNAVAILABLE", 2)?;
    crate::webidl::define_constant(s, o, "TIMEOUT", 3)
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<GeolocationPositionErrorStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "GeolocationPositionError",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "code", get_code)?;
    crate::webidl::define_readonly_accessor(s, p, "message", get_message)?;
    constants(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    constants(s, c.into())?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<GeolocationPositionErrorStore>()
        .ok_or_else(|| "GeolocationPositionError state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    code: i32,
    message: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create GeolocationPositionError".to_owned());
    }
    s.get_slot_mut::<GeolocationPositionErrorStore>()
        .ok_or_else(|| "GeolocationPositionError state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), ErrorRecord { code, message });
    Ok(o)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ErrorRecord> {
    s.get_slot::<GeolocationPositionErrorStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_code(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Integer::new(s, x.code).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        if let Some(v) = v8::String::new(s, &x.message) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
