use std::collections::HashMap;

#[derive(Clone)]
struct SpeechSynthesisVoiceRecord {
    voice_uri: String,
    name: String,
    lang: String,
    local_service: bool,
    is_default: bool,
}

#[derive(Default)]
pub(crate) struct SpeechSynthesisVoiceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SpeechSynthesisVoiceRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechSynthesisVoiceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechSynthesisVoice", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<SpeechSynthesisVoiceStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechSynthesisVoice",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "voiceURI", get_voice_uri)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lang", get_lang)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "localService", get_local_service)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "default", get_default)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechSynthesisVoiceStore>()
        .ok_or_else(|| "SpeechSynthesisVoice state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SpeechSynthesisVoice': Illegal constructor",
    )
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    voice_uri: String,
    name: String,
    lang: String,
    local_service: bool,
    is_default: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SpeechSynthesisVoice".to_owned());
    }
    let record = SpeechSynthesisVoiceRecord {
        voice_uri,
        name,
        lang,
        local_service,
        is_default,
    };
    scope
        .get_slot_mut::<SpeechSynthesisVoiceStore>()
        .ok_or_else(|| "SpeechSynthesisVoice state is unavailable".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<SpeechSynthesisVoiceStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechSynthesisVoiceRecord> {
    scope
        .get_slot::<SpeechSynthesisVoiceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn text_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&SpeechSynthesisVoiceRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

fn get_voice_uri(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |record| &record.voice_uri)
}
fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |record| &record.name)
}
fn get_lang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |record| &record.lang)
}
fn get_local_service(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.local_service).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_default(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.is_default).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
