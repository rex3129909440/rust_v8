use std::collections::HashMap;
#[derive(Clone)]
struct TranslatorData {
    destroyed: bool,
    source: String,
    target: String,
}
#[derive(Default)]
pub(crate) struct TranslatorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TranslatorData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(TranslatorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "Translator", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<TranslatorStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "Translator",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "inputQuota", quota)?;
    crate::webidl::define_readonly_accessor(s, p, "sourceLanguage", source)?;
    crate::webidl::define_readonly_accessor(s, p, "targetLanguage", target)?;
    crate::webidl::define_method(s, p, "destroy", 0, destroy)?;
    crate::webidl::define_method(s, p, "measureInputUsage", 1, measure)?;
    crate::webidl::define_method(s, p, "translate", 1, translate)?;
    crate::webidl::define_method(s, p, "translateStreaming", 1, stream)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_method(s, c.into(), "availability", 1, availability)?;
    crate::webidl::define_method(s, c.into(), "create", 1, static_create)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<TranslatorStore>()
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
        return Err("cannot create Translator".to_owned());
    }
    s.get_slot_mut::<TranslatorStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        TranslatorData {
            destroyed: false,
            source: "en".to_owned(),
            target: "zh".to_owned(),
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<TranslatorData> {
    s.get_slot::<TranslatorStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
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
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(TranslatorData) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn source(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.source)
}
fn target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.target)
}
fn destroy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<TranslatorStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.destroyed = true
    }
}
fn measure(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let n = crate::webidl::value_to_string(s, a.get(0)).len() as f64;
    let x = v8::Number::new(s, n);
    promise(s, x.into(), r)
}
fn translate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    let x = v8::String::new(s, &value).unwrap();
    promise(s, x.into(), r)
}
fn stream(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(x) = super::readable_stream::create_empty(s) {
        r.set(x.into())
    }
}
fn availability(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let x = v8::String::new(s, "available").unwrap();
    promise(s, x.into(), r)
}
fn static_create(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match create(s) {
        Ok(x) => promise(s, x.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
