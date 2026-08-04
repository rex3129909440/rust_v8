use std::collections::HashMap;
#[derive(Clone, Copy)]
struct Confidence {
    rate: f64,
    value: bool,
}
#[derive(Default)]
pub(crate) struct PerformanceTimingConfidenceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Confidence>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PerformanceTimingConfidenceStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PerformanceTimingConfidence", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<PerformanceTimingConfidenceStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "PerformanceTimingConfidence",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "randomizedTriggerRate", get_rate)?;
    crate::webidl::define_readonly_accessor(s, p, "value", get_value)?;
    crate::webidl::define_method(s, p, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PerformanceTimingConfidenceStore>()
        .ok_or_else(|| "PerformanceTimingConfidence state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
#[allow(dead_code)]
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    rate: f64,
    value: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create PerformanceTimingConfidence".to_owned());
    }
    s.get_slot_mut::<PerformanceTimingConfidenceStore>()
        .ok_or_else(|| "state missing".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), Confidence { rate, value });
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Confidence> {
    s.get_slot::<PerformanceTimingConfidenceStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .copied()
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'PerformanceTimingConfidence': Illegal constructor",
    )
}
fn get_rate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.rate).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, x.value).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
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
    let kr = crate::webidl::string(s, "randomizedTriggerRate").ok();
    let kv = crate::webidl::string(s, "value").ok();
    if let (Some(kr), Some(kv)) = (kr, kv) {
        let rate = v8::Number::new(s, x.rate);
        let value = v8::Boolean::new(s, x.value);
        let _ = o.define_own_property(s, kr.into(), rate.into(), v8::PropertyAttribute::NONE);
        let _ = o.define_own_property(s, kv.into(), value.into(), v8::PropertyAttribute::NONE);
        r.set(o.into())
    }
}
