use std::collections::HashMap;
#[derive(Clone)]
struct ProfilerRecord {
    sample_interval: f64,
    stopped: bool,
    started: f64,
    realm_id: i32,
}
#[derive(Default)]
pub(crate) struct ProfilerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ProfilerRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(ProfilerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "Profiler", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<ProfilerStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "Profiler",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "sampleInterval", get_interval)?;
    crate::webidl::define_readonly_accessor(s, p, "stopped", get_stopped)?;
    crate::webidl::define_method(s, p, "stop", 0, stop)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<ProfilerStore>()
        .ok_or_else(|| "Profiler state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'Profiler': 1 argument required, but only 0 present.",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    let interval = property_number(s, init, "sampleInterval").unwrap_or(10.0);
    super::event_target::attach(s, a.this());
    let started = super::performance::now_for_current_realm(s).unwrap_or(0.0);
    let realm_id = crate::webidl::realm_id(s);
    s.get_slot_mut::<ProfilerStore>()
        .expect("state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            ProfilerRecord {
                sample_interval: interval,
                stopped: false,
                started,
                realm_id,
            },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ProfilerRecord> {
    s.get_slot::<ProfilerStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_interval(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.sample_interval).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_stopped(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, x.stopped).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn stop(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(mut x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    x.stopped = true;
    let elapsed = (super::performance::now_for_realm(s, x.realm_id).unwrap_or(x.started)
        - x.started)
        .max(0.0);
    if let Some(v) = s
        .get_slot_mut::<ProfilerStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.stopped = true;
    }
    let profile = v8::Object::new(s);
    define_number(s, profile, "startTime", 0.0);
    define_number(s, profile, "endTime", elapsed);
    let frames = v8::Array::new(s, 0);
    let samples = v8::Array::new(s, 0);
    define_value(s, profile, "frames", frames.into());
    define_value(s, profile, "samples", samples.into());
    if let Ok(p) = super::writable_stream::resolved_promise(s, profile.into()) {
        r.set(p.into())
    }
}
fn property_number(
    s: &v8::PinScope<'_, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    n: &str,
) -> Option<f64> {
    let k = v8::String::new(s, n)?;
    o?.get(s, k.into())?.number_value(s)
}
fn define_number(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str, v: f64) {
    define_value(s, o, n, v8::Number::new(s, v).into())
}
fn define_value(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
    v: v8::Local<'_, v8::Value>,
) {
    if let Some(k) = v8::String::new(s, n) {
        let _ = o.define_own_property(s, k.into(), v, v8::PropertyAttribute::NONE);
    }
}
