use std::collections::HashMap;
#[derive(Clone)]
struct PhraseData {
    phrase: String,
    boost: f64,
}
#[derive(Default)]
pub(crate) struct SpeechRecognitionPhraseStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PhraseData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SpeechRecognitionPhraseStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "SpeechRecognitionPhrase", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<SpeechRecognitionPhraseStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "SpeechRecognitionPhrase",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "phrase", phrase)?;
    crate::webidl::define_readonly_accessor(s, p, "boost", boost)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SpeechRecognitionPhraseStore>()
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
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "phrase required");
        return;
    }
    let phrase = crate::webidl::value_to_string(s, a.get(0));
    let boost = a.get(1).number_value(s).unwrap_or(1.0);
    s.get_slot_mut::<SpeechRecognitionPhraseStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            PhraseData { phrase, boost },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<PhraseData> {
    s.get_slot::<SpeechRecognitionPhraseStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn phrase(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.phrase)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn boost(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Number::new(s, v.boost).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
