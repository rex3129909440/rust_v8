use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CoordinatesRecord {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
}
#[derive(Default)]
pub(crate) struct GeolocationCoordinatesStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CoordinatesRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(GeolocationCoordinatesStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "GeolocationCoordinates", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<GeolocationCoordinatesStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "GeolocationCoordinates",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "latitude", get_latitude)?;
    crate::webidl::define_readonly_accessor(s, p, "longitude", get_longitude)?;
    crate::webidl::define_readonly_accessor(s, p, "altitude", get_altitude)?;
    crate::webidl::define_readonly_accessor(s, p, "accuracy", get_accuracy)?;
    crate::webidl::define_readonly_accessor(s, p, "altitudeAccuracy", get_altitude_accuracy)?;
    crate::webidl::define_readonly_accessor(s, p, "heading", get_heading)?;
    crate::webidl::define_readonly_accessor(s, p, "speed", get_speed)?;
    crate::webidl::define_method(s, p, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<GeolocationCoordinatesStore>()
        .ok_or_else(|| "GeolocationCoordinates state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    record: CoordinatesRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create GeolocationCoordinates".to_owned());
    }
    s.get_slot_mut::<GeolocationCoordinatesStore>()
        .ok_or_else(|| "GeolocationCoordinates state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(o)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<CoordinatesRecord> {
    s.get_slot::<GeolocationCoordinatesStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(&CoordinatesRecord) -> Option<f64>,
) {
    if let Some(x) = record(s, a.this()) {
        match f(&x) {
            Some(v) => r.set(v8::Number::new(s, v).into()),
            None => r.set(v8::null(s).into()),
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_latitude(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| Some(x.latitude))
}
fn get_longitude(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| Some(x.longitude))
}
fn get_altitude(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.altitude)
}
fn get_accuracy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| Some(x.accuracy))
}
fn get_altitude_accuracy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.altitude_accuracy)
}
fn get_heading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.heading)
}
fn get_speed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.speed)
}
fn data(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(k) = v8::String::new(s, name) {
        let _ = o.create_data_property(s, k.into(), value);
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
    data(s, o, "latitude", v8::Number::new(s, x.latitude).into());
    data(s, o, "longitude", v8::Number::new(s, x.longitude).into());
    data(
        s,
        o,
        "altitude",
        x.altitude
            .map(|v| v8::Number::new(s, v).into())
            .unwrap_or_else(|| v8::null(s).into()),
    );
    data(s, o, "accuracy", v8::Number::new(s, x.accuracy).into());
    data(
        s,
        o,
        "altitudeAccuracy",
        x.altitude_accuracy
            .map(|v| v8::Number::new(s, v).into())
            .unwrap_or_else(|| v8::null(s).into()),
    );
    data(
        s,
        o,
        "heading",
        x.heading
            .map(|v| v8::Number::new(s, v).into())
            .unwrap_or_else(|| v8::null(s).into()),
    );
    data(
        s,
        o,
        "speed",
        x.speed
            .map(|v| v8::Number::new(s, v).into())
            .unwrap_or_else(|| v8::null(s).into()),
    );
    r.set(o.into())
}
