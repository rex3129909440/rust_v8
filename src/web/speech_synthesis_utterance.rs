use std::collections::HashMap;

#[derive(Clone, Copy)]
enum HandlerSlot {
    Start,
    End,
    Error,
    Pause,
    Resume,
    Mark,
    Boundary,
}

#[derive(Clone)]
struct SpeechSynthesisUtteranceRecord {
    text: String,
    lang: String,
    voice: Option<v8::Global<v8::Object>>,
    volume: f64,
    rate: f64,
    pitch: f64,
    onstart: Option<v8::Global<v8::Value>>,
    onend: Option<v8::Global<v8::Value>>,
    onerror: Option<v8::Global<v8::Value>>,
    onpause: Option<v8::Global<v8::Value>>,
    onresume: Option<v8::Global<v8::Value>>,
    onmark: Option<v8::Global<v8::Value>>,
    onboundary: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct SpeechSynthesisUtteranceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SpeechSynthesisUtteranceRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechSynthesisUtteranceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechSynthesisUtterance", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<SpeechSynthesisUtteranceStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechSynthesisUtterance",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "text", get_text, set_text)?;
    crate::webidl::define_accessor(scope, prototype, "lang", get_lang, set_lang)?;
    crate::webidl::define_accessor(scope, prototype, "voice", get_voice, set_voice)?;
    crate::webidl::define_accessor(scope, prototype, "volume", get_volume, set_volume)?;
    crate::webidl::define_accessor(scope, prototype, "rate", get_rate, set_rate)?;
    crate::webidl::define_accessor(scope, prototype, "pitch", get_pitch, set_pitch)?;
    crate::webidl::define_accessor(scope, prototype, "onstart", get_onstart, set_onstart)?;
    crate::webidl::define_accessor(scope, prototype, "onend", get_onend, set_onend)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_accessor(scope, prototype, "onpause", get_onpause, set_onpause)?;
    crate::webidl::define_accessor(scope, prototype, "onresume", get_onresume, set_onresume)?;
    crate::webidl::define_accessor(scope, prototype, "onmark", get_onmark, set_onmark)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onboundary",
        get_onboundary,
        set_onboundary,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechSynthesisUtteranceStore>()
        .ok_or_else(|| "SpeechSynthesisUtterance state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SpeechSynthesisUtterance': Please use the 'new' operator.",
        );
        return;
    }
    let text = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    super::event_target::attach(scope, arguments.this());
    scope
        .get_slot_mut::<SpeechSynthesisUtteranceStore>()
        .expect("SpeechSynthesisUtterance state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            SpeechSynthesisUtteranceRecord {
                text,
                lang: String::new(),
                voice: None,
                volume: 1.0,
                rate: 1.0,
                pitch: 1.0,
                onstart: None,
                onend: None,
                onerror: None,
                onpause: None,
                onresume: None,
                onmark: None,
                onboundary: None,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<SpeechSynthesisUtteranceStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechSynthesisUtteranceRecord> {
    scope
        .get_slot::<SpeechSynthesisUtteranceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut SpeechSynthesisUtteranceRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<SpeechSynthesisUtteranceStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(record);
    true
}

fn text_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&SpeechSynthesisUtteranceRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}
fn get_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |record| &record.text)
}
fn set_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.text = value);
}
fn get_lang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |record| &record.lang)
}
fn set_lang(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.lang = value);
}

fn get_voice(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.voice {
        Some(voice) => result.set(v8::Local::new(scope, &voice).into()),
        None => result.set(v8::null(scope).into()),
    }
}
fn set_voice(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.get(0).is_null() {
        update(scope, arguments.this(), |record| record.voice = None);
        return;
    }
    let Ok(voice) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "voice must be a SpeechSynthesisVoice");
        return;
    };
    if !super::speech_synthesis_voice::is_instance(scope, voice) {
        crate::webidl::throw_type_error(scope, "voice must be a SpeechSynthesisVoice");
        return;
    }
    let voice = v8::Global::new(scope, voice);
    update(scope, arguments.this(), |record| record.voice = Some(voice));
}

fn number_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&SpeechSynthesisUtteranceRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_volume(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |record| record.volume)
}
fn set_volume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .number_value(scope)
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    update(scope, arguments.this(), |record| record.volume = value);
}
fn get_rate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |record| record.rate)
}
fn set_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .number_value(scope)
        .unwrap_or(1.0)
        .clamp(0.1, 10.0);
    update(scope, arguments.this(), |record| record.rate = value);
}
fn get_pitch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |record| record.pitch)
}
fn set_pitch(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .number_value(scope)
        .unwrap_or(1.0)
        .clamp(0.0, 2.0);
    update(scope, arguments.this(), |record| record.pitch = value);
}

fn select_handler(
    record: &SpeechSynthesisUtteranceRecord,
    slot: HandlerSlot,
) -> Option<v8::Global<v8::Value>> {
    match slot {
        HandlerSlot::Start => record.onstart.clone(),
        HandlerSlot::End => record.onend.clone(),
        HandlerSlot::Error => record.onerror.clone(),
        HandlerSlot::Pause => record.onpause.clone(),
        HandlerSlot::Resume => record.onresume.clone(),
        HandlerSlot::Mark => record.onmark.clone(),
        HandlerSlot::Boundary => record.onboundary.clone(),
    }
}
fn assign_handler(
    record: &mut SpeechSynthesisUtteranceRecord,
    slot: HandlerSlot,
    value: Option<v8::Global<v8::Value>>,
) {
    match slot {
        HandlerSlot::Start => record.onstart = value,
        HandlerSlot::End => record.onend = value,
        HandlerSlot::Error => record.onerror = value,
        HandlerSlot::Pause => record.onpause = value,
        HandlerSlot::Resume => record.onresume = value,
        HandlerSlot::Mark => record.onmark = value,
        HandlerSlot::Boundary => record.onboundary = value,
    }
}
fn handler_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
    slot: HandlerSlot,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(
        scope,
        select_handler(&record, slot),
        result,
    );
}
fn handler_set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    slot: HandlerSlot,
) {
    let value = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        assign_handler(record, slot, value)
    });
}

fn get_onstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::Start)
}
fn set_onstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::Start)
}
fn get_onend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::End)
}
fn set_onend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::End)
}
fn get_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::Error)
}
fn set_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::Error)
}
fn get_onpause(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::Pause)
}
fn set_onpause(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::Pause)
}
fn get_onresume(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::Resume)
}
fn set_onresume(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::Resume)
}
fn get_onmark(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::Mark)
}
fn set_onmark(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::Mark)
}
fn get_onboundary(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::Boundary)
}
fn set_onboundary(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::Boundary)
}

pub(crate) fn fire(
    scope: &mut v8::PinScope<'_, '_>,
    utterance: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    let Some(record) = record(scope, utterance) else {
        return;
    };
    let handler = match event_type {
        "start" => record.onstart,
        "end" => record.onend,
        "pause" => record.onpause,
        "resume" => record.onresume,
        _ => None,
    };
    let Ok(event) =
        super::speech_synthesis_event::create(scope, event_type, utterance, 0, 0, 0.0, "")
    else {
        return;
    };
    super::event_target::dispatch(scope, utterance, event);
    if let Some(handler) = handler
        && let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = function.call(scope, utterance.into(), &[event.into()]);
    }
}
