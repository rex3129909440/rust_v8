use std::collections::HashMap;
#[derive(Clone)]
struct PositionRecord {
    coords: v8::Global<v8::Object>,
    timestamp: f64,
}
#[derive(Default)]
pub(crate) struct GeolocationPositionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PositionRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(GeolocationPositionStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "GeolocationPosition", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<GeolocationPositionStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "GeolocationPosition",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "coords", get_coords)?;
    crate::webidl::define_readonly_accessor(s, p, "timestamp", get_timestamp)?;
    crate::webidl::define_method(s, p, "toJSON", 0, to_json)?;
    // Experimental Android-only accessor observed on HTTPS in Chromium
    // 146-148. Version reconciliation removes it everywhere else.
    crate::webidl::define_readonly_accessor(s, p, "accuracyMode", get_accuracy_mode)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<GeolocationPositionStore>()
        .ok_or_else(|| "GeolocationPosition state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    coords: v8::Local<'_, v8::Object>,
    timestamp: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create GeolocationPosition".to_owned());
    }
    let coords = v8::Global::new(s, coords);
    s.get_slot_mut::<GeolocationPositionStore>()
        .ok_or_else(|| "GeolocationPosition state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            PositionRecord { coords, timestamp },
        );
    Ok(o)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<PositionRecord> {
    s.get_slot::<GeolocationPositionStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_coords(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.coords).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.timestamp).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_accuracy_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if let Some(value) = v8::String::new(s, "precise") {
        r.set(value.into());
    }
}
fn to_json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let o = v8::Object::new(s);
    if let Some(k) = v8::String::new(s, "coords") {
        let _ = o.create_data_property(s, k.into(), v8::Local::new(s, &x.coords).into());
    }
    if let Some(k) = v8::String::new(s, "timestamp") {
        let value = v8::Number::new(s, x.timestamp);
        let _ = o.create_data_property(s, k.into(), value.into());
    }
    r.set(o.into())
}
