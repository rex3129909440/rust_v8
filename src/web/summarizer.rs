use std::collections::HashMap;
#[derive(Clone)]
struct SummarizerData {
    destroyed: bool,
    shared: String,
    kind: String,
    format: String,
    length: String,
    languages: v8::Global<v8::Array>,
    output: String,
}
#[derive(Default)]
pub(crate) struct SummarizerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SummarizerData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SummarizerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "Summarizer", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<SummarizerStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "Summarizer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "sharedContext", shared)?;
    crate::webidl::define_readonly_accessor(s, p, "type", kind)?;
    crate::webidl::define_readonly_accessor(s, p, "format", format)?;
    crate::webidl::define_readonly_accessor(s, p, "length", length)?;
    crate::webidl::define_readonly_accessor(s, p, "expectedInputLanguages", languages)?;
    crate::webidl::define_readonly_accessor(s, p, "expectedContextLanguages", languages)?;
    crate::webidl::define_readonly_accessor(s, p, "outputLanguage", output)?;
    crate::webidl::define_readonly_accessor(s, p, "inputQuota", quota)?;
    crate::webidl::define_method(s, p, "destroy", 0, destroy)?;
    crate::webidl::define_method(s, p, "measureInputUsage", 1, measure)?;
    crate::webidl::define_method(s, p, "summarize", 1, summarize)?;
    crate::webidl::define_method(s, p, "summarizeStreaming", 1, stream)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_method(s, c.into(), "availability", 0, availability)?;
    crate::webidl::define_method(s, c.into(), "create", 0, static_create)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SummarizerStore>()
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
        return Err("cannot create Summarizer".to_owned());
    }
    let languages = v8::Global::new(s, v8::Array::new(s, 0));
    s.get_slot_mut::<SummarizerStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        SummarizerData {
            destroyed: false,
            shared: String::new(),
            kind: "key-points".to_owned(),
            format: "plain-text".to_owned(),
            length: "medium".to_owned(),
            languages,
            output: "en".to_owned(),
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<SummarizerData> {
    s.get_slot::<SummarizerStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(SummarizerData) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn shared(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.shared)
}
fn kind(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.kind)
}
fn format(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.format)
}
fn length(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.length)
}
fn output(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.output)
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
        .get_slot_mut::<SummarizerStore>()
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
fn summarize(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let input = crate::webidl::value_to_string(s, a.get(0));
    let summary = input
        .split_whitespace()
        .take(32)
        .collect::<Vec<_>>()
        .join(" ");
    let x = v8::String::new(s, &summary).unwrap();
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
