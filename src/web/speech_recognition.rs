use std::collections::HashMap;

#[derive(Clone, Copy)]
enum HandlerSlot {
    AudioStart,
    SoundStart,
    SpeechStart,
    SpeechEnd,
    SoundEnd,
    AudioEnd,
    Result,
    NoMatch,
    Error,
    Start,
    End,
}

#[derive(Clone)]
struct SpeechRecognitionRecord {
    object: v8::Global<v8::Object>,
    grammars: v8::Global<v8::Object>,
    lang: String,
    continuous: bool,
    interim_results: bool,
    max_alternatives: u32,
    onaudiostart: Option<v8::Global<v8::Value>>,
    onsoundstart: Option<v8::Global<v8::Value>>,
    onspeechstart: Option<v8::Global<v8::Value>>,
    onspeechend: Option<v8::Global<v8::Value>>,
    onsoundend: Option<v8::Global<v8::Value>>,
    onaudioend: Option<v8::Global<v8::Value>>,
    onresult: Option<v8::Global<v8::Value>>,
    onnomatch: Option<v8::Global<v8::Value>>,
    onerror: Option<v8::Global<v8::Value>>,
    onstart: Option<v8::Global<v8::Value>>,
    onend: Option<v8::Global<v8::Value>>,
    started: bool,
    quality: String,
    process_locally: bool,
    phrases: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct SpeechRecognitionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SpeechRecognitionRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechRecognitionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechRecognition", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SpeechRecognitionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechRecognition",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "grammars", get_grammars, set_grammars)?;
    crate::webidl::define_accessor(scope, prototype, "lang", get_lang, set_lang)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "continuous",
        get_continuous,
        set_continuous,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "interimResults",
        get_interim_results,
        set_interim_results,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "maxAlternatives",
        get_max_alternatives,
        set_max_alternatives,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onaudiostart",
        get_onaudiostart,
        set_onaudiostart,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onsoundstart",
        get_onsoundstart,
        set_onsoundstart,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onspeechstart",
        get_onspeechstart,
        set_onspeechstart,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onspeechend",
        get_onspeechend,
        set_onspeechend,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onsoundend",
        get_onsoundend,
        set_onsoundend,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onaudioend",
        get_onaudioend,
        set_onaudioend,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onresult", get_onresult, set_onresult)?;
    crate::webidl::define_accessor(scope, prototype, "onnomatch", get_onnomatch, set_onnomatch)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_accessor(scope, prototype, "onstart", get_onstart, set_onstart)?;
    crate::webidl::define_accessor(scope, prototype, "onend", get_onend, set_onend)?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::define_method(scope, prototype, "start", 0, start)?;
    crate::webidl::define_method(scope, prototype, "stop", 0, stop)?;
    crate::webidl::define_accessor(scope, prototype, "quality", get_quality, set_quality)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "processLocally",
        get_process_locally,
        set_process_locally,
    )?;
    crate::webidl::define_accessor(scope, prototype, "phrases", get_phrases, set_phrases)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "available", 1, available)?;
    crate::webidl::define_method(scope, constructor.into(), "install", 1, install_language)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechRecognitionStore>()
        .ok_or_else(|| "SpeechRecognition state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
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
            "Failed to construct 'SpeechRecognition': Please use the 'new' operator.",
        );
        return;
    }
    let Ok(grammars) = super::speech_grammar_list::create(scope) else {
        return;
    };
    let phrases = v8::Array::new(scope, 0);
    let phrases_value: v8::Local<v8::Value> = phrases.into();
    super::event_target::attach(scope, arguments.this());
    let record = SpeechRecognitionRecord {
        object: v8::Global::new(scope, arguments.this()),
        grammars: v8::Global::new(scope, grammars),
        lang: String::new(),
        continuous: false,
        interim_results: false,
        max_alternatives: 1,
        onaudiostart: None,
        onsoundstart: None,
        onspeechstart: None,
        onspeechend: None,
        onsoundend: None,
        onaudioend: None,
        onresult: None,
        onnomatch: None,
        onerror: None,
        onstart: None,
        onend: None,
        started: false,
        quality: "command".to_owned(),
        process_locally: false,
        phrases: v8::Global::new(scope, phrases_value),
    };
    scope
        .get_slot_mut::<SpeechRecognitionStore>()
        .expect("SpeechRecognition state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechRecognitionRecord> {
    scope
        .get_slot::<SpeechRecognitionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut SpeechRecognitionRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<SpeechRecognitionStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(record);
    true
}

fn get_grammars(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.grammars).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_grammars(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(value) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "grammars must be a SpeechGrammarList");
        return;
    };
    if !super::speech_grammar_list::is_instance(scope, value) {
        crate::webidl::throw_type_error(scope, "grammars must be a SpeechGrammarList");
        return;
    }
    let value = v8::Global::new(scope, value);
    update(scope, arguments.this(), |record| record.grammars = value);
}

fn text_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&SpeechRecognitionRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
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
fn get_quality(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |record| &record.quality)
}
fn set_quality(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value == "command" || value == "dictation" {
        update(scope, arguments.this(), |record| record.quality = value);
    } else if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn bool_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&SpeechRecognitionRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_continuous(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    bool_get(s, a, r, |record| record.continuous)
}
fn set_continuous(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.continuous = value);
}
fn get_interim_results(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    bool_get(s, a, r, |record| record.interim_results)
}
fn set_interim_results(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.interim_results = value
    });
}
fn get_process_locally(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    bool_get(s, a, r, |record| record.process_locally)
}
fn set_process_locally(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.process_locally = value
    });
}

fn get_max_alternatives(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.max_alternatives).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_max_alternatives(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .integer_value(scope)
        .unwrap_or(1)
        .clamp(0, u32::MAX as i64) as u32;
    update(scope, arguments.this(), |record| {
        record.max_alternatives = value
    });
}

fn get_phrases(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.phrases));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_phrases(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_array() {
        arguments.get(0)
    } else {
        v8::Array::new(scope, 0).into()
    };
    let value = v8::Global::new(scope, value);
    update(scope, arguments.this(), |record| record.phrases = value);
}

fn select_handler(
    record: &SpeechRecognitionRecord,
    slot: HandlerSlot,
) -> Option<v8::Global<v8::Value>> {
    match slot {
        HandlerSlot::AudioStart => record.onaudiostart.clone(),
        HandlerSlot::SoundStart => record.onsoundstart.clone(),
        HandlerSlot::SpeechStart => record.onspeechstart.clone(),
        HandlerSlot::SpeechEnd => record.onspeechend.clone(),
        HandlerSlot::SoundEnd => record.onsoundend.clone(),
        HandlerSlot::AudioEnd => record.onaudioend.clone(),
        HandlerSlot::Result => record.onresult.clone(),
        HandlerSlot::NoMatch => record.onnomatch.clone(),
        HandlerSlot::Error => record.onerror.clone(),
        HandlerSlot::Start => record.onstart.clone(),
        HandlerSlot::End => record.onend.clone(),
    }
}

fn assign_handler(
    record: &mut SpeechRecognitionRecord,
    slot: HandlerSlot,
    value: Option<v8::Global<v8::Value>>,
) {
    match slot {
        HandlerSlot::AudioStart => record.onaudiostart = value,
        HandlerSlot::SoundStart => record.onsoundstart = value,
        HandlerSlot::SpeechStart => record.onspeechstart = value,
        HandlerSlot::SpeechEnd => record.onspeechend = value,
        HandlerSlot::SoundEnd => record.onsoundend = value,
        HandlerSlot::AudioEnd => record.onaudioend = value,
        HandlerSlot::Result => record.onresult = value,
        HandlerSlot::NoMatch => record.onnomatch = value,
        HandlerSlot::Error => record.onerror = value,
        HandlerSlot::Start => record.onstart = value,
        HandlerSlot::End => record.onend = value,
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

fn get_onaudiostart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::AudioStart)
}
fn set_onaudiostart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::AudioStart)
}
fn get_onsoundstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::SoundStart)
}
fn set_onsoundstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::SoundStart)
}
fn get_onspeechstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::SpeechStart)
}
fn set_onspeechstart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::SpeechStart)
}
fn get_onspeechend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::SpeechEnd)
}
fn set_onspeechend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::SpeechEnd)
}
fn get_onsoundend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::SoundEnd)
}
fn set_onsoundend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::SoundEnd)
}
fn get_onaudioend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::AudioEnd)
}
fn set_onaudioend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::AudioEnd)
}
fn get_onresult(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::Result)
}
fn set_onresult(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::Result)
}
fn get_onnomatch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, HandlerSlot::NoMatch)
}
fn set_onnomatch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, HandlerSlot::NoMatch)
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

fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>) {
    if let Ok(exception) = super::dom_exception::create(
        scope,
        "recognition has already started.".to_owned(),
        "InvalidStateError".to_owned(),
    ) {
        scope.throw_exception(exception.into());
    }
}

fn start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.started {
        throw_invalid_state(scope);
        return;
    }
    update(scope, arguments.this(), |record| record.started = true);
    let data = v8::Integer::new(scope, arguments.this().get_identity_hash().get());
    if let Some(task) = v8::Function::builder(fail_start)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        scope.enqueue_microtask(task);
    }
}

fn fail_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(id) = arguments.data().int32_value(scope) else {
        return;
    };
    let Some((target, error_handler, end_handler)) = scope
        .get_slot_mut::<SpeechRecognitionStore>()
        .and_then(|store| store.records.get_mut(&id))
        .map(|record| {
            record.started = false;
            (
                record.object.clone(),
                record.onerror.clone(),
                record.onend.clone(),
            )
        })
    else {
        return;
    };
    let target = v8::Local::new(scope, &target);
    if let Ok(event) = super::speech_recognition_error_event::create(
        scope,
        "error",
        "not-allowed",
        "Speech recognition permission was not granted",
    ) {
        super::event_target::dispatch(scope, target, event);
        call_handler(scope, target, error_handler, event);
    }
    if let Ok(event) = super::event::create(scope, "end") {
        super::event_target::dispatch(scope, target, event);
        call_handler(scope, target, end_handler, event);
    }
}

fn call_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
    event: v8::Local<'_, v8::Object>,
) {
    let Some(handler) = handler else {
        return;
    };
    if let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) {
        let _ = function.call(scope, target.into(), &[event.into()]);
    }
}

fn finish_if_started(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let Some(current) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !current.started {
        return;
    }
    let handler = current.onend;
    update(scope, object, |record| record.started = false);
    if let Ok(event) = super::event::create(scope, "end") {
        super::event_target::dispatch(scope, object, event);
        call_handler(scope, object, handler, event);
    }
}
fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    finish_if_started(scope, arguments.this())
}
fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    finish_if_started(scope, arguments.this())
}

fn validate_languages(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    method: &str,
) -> bool {
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(
            scope,
            &format!("Failed to execute '{method}' on 'SpeechRecognition': options are required."),
        );
        return false;
    };
    let Some(key) = v8::String::new(scope, "langs") else {
        return false;
    };
    let Some(langs) = options.get(scope, key.into()) else {
        return false;
    };
    if langs.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'SpeechRecognition': Required member is undefined."
            ),
        );
        return false;
    }
    let Ok(langs) = v8::Local::<v8::Array>::try_from(langs) else {
        crate::webidl::throw_type_error(scope, "langs must be an array.");
        return false;
    };
    if langs.length() == 0 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'SpeechRecognition': Langs array cannot be empty."
            ),
        );
        return false;
    }
    true
}

fn available(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !validate_languages(scope, arguments.get(0), "available") {
        return;
    }
    if let Some(value) = v8::String::new(scope, "available") {
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
            result.set(promise.into());
        }
    }
}

fn install_language(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !validate_languages(scope, arguments.get(0), "install") {
        return;
    }
    let value = v8::Boolean::new(scope, true);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}
