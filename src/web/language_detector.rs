use std::collections::HashMap;
#[derive(Clone)]
struct DetectorRecord {
    destroyed: bool,
    languages: v8::Global<v8::Array>,
}
#[derive(Default)]
pub(crate) struct LanguageDetectorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DetectorRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(LanguageDetectorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "LanguageDetector", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<LanguageDetectorStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "LanguageDetector",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "expectedInputLanguages", languages)?;
    crate::webidl::define_readonly_accessor(s, p, "inputQuota", quota)?;
    crate::webidl::define_method(s, p, "destroy", 0, destroy)?;
    crate::webidl::define_method(s, p, "detect", 1, detect)?;
    crate::webidl::define_method(s, p, "measureInputUsage", 1, measure)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_method(s, c.into(), "availability", 0, availability)?;
    crate::webidl::define_method(s, c.into(), "create", 0, static_create)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<LanguageDetectorStore>()
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
fn create<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create LanguageDetector".to_owned());
    }
    let array = v8::Array::new(s, 0);
    let languages = v8::Global::new(s, array);
    s.get_slot_mut::<LanguageDetectorStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            DetectorRecord {
                destroyed: false,
                languages,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<DetectorRecord> {
    s.get_slot::<LanguageDetectorStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn languages(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.languages).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn quota(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Number::new(s, 4096.0).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn destroy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<LanguageDetectorStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.destroyed = true
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn detect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "LanguageDetector", "detect", r);
        return;
    }
    let item = v8::Object::new(s);
    if let (Some(k), Some(v)) = (
        v8::String::new(s, "detectedLanguage"),
        v8::String::new(s, "en"),
    ) {
        let _ = item.set(s, k.into(), v.into());
    }
    if let Some(k) = v8::String::new(s, "confidence") {
        let _ = item.set(s, k.into(), v8::Number::new(s, 0.99).into());
    }
    let array = v8::Array::new(s, 1);
    let _ = array.set_index(s, 0, item.into());
    promise(s, array.into(), r)
}
fn measure(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "LanguageDetector",
            "measureInputUsage",
            r,
        );
        return;
    }
    let n = crate::webidl::value_to_string(s, a.get(0)).chars().count() as f64;
    let v = v8::Number::new(s, n);
    promise(s, v.into(), r)
}
fn availability(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let v = v8::String::new(s, "available").unwrap();
    promise(s, v.into(), r)
}
fn static_create(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match create(s) {
        Ok(v) => promise(s, v.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
