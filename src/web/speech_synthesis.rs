use std::collections::HashMap;

#[derive(Clone, Default)]
struct SpeechSynthesisRecord {
    object: Option<v8::Global<v8::Object>>,
    voices: Vec<v8::Global<v8::Object>>,
    queue: Vec<v8::Global<v8::Object>>,
    paused: bool,
    scheduled: bool,
    on_voices_changed: Option<v8::Global<v8::Value>>,
    preload_count: usize,
}

#[derive(Default)]
pub(crate) struct SpeechSynthesisStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SpeechSynthesisRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechSynthesisStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechSynthesis", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<SpeechSynthesisStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechSynthesis",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pending", get_pending)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "speaking", get_speaking)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "paused", get_paused)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onvoiceschanged",
        get_on_voices_changed,
        set_on_voices_changed,
    )?;
    crate::webidl::define_method(scope, prototype, "cancel", 0, cancel)?;
    crate::webidl::define_method(scope, prototype, "getVoices", 0, get_voices)?;
    crate::webidl::define_method(scope, prototype, "pause", 0, pause)?;
    crate::webidl::define_method(scope, prototype, "resume", 0, resume)?;
    crate::webidl::define_method(scope, prototype, "speak", 1, speak)?;
    crate::webidl::define_method(scope, prototype, "preload", 2, preload)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechSynthesisStore>()
        .ok_or_else(|| "SpeechSynthesis state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let synthesis = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, synthesis, prototype.into()) != Some(true) {
        return Err("cannot create SpeechSynthesis".to_owned());
    }
    super::event_target::attach(scope, synthesis);
    let profiles = crate::fingerprint::edge(scope).speech.voices.clone();
    let mut voices = Vec::with_capacity(profiles.len());
    for profile in profiles {
        let voice = super::speech_synthesis_voice::create(
            scope,
            profile.voice_uri,
            profile.name,
            profile.lang,
            profile.local_service,
            profile.is_default,
        )?;
        voices.push(v8::Global::new(scope, voice));
    }
    let synthesis_global = v8::Global::new(scope, synthesis);
    scope
        .get_slot_mut::<SpeechSynthesisStore>()
        .ok_or_else(|| "SpeechSynthesis state was not prepared".to_owned())?
        .records
        .insert(
            synthesis.get_identity_hash().get(),
            SpeechSynthesisRecord {
                object: Some(synthesis_global),
                voices,
                ..SpeechSynthesisRecord::default()
            },
        );
    Ok(synthesis)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SpeechSynthesis': Illegal constructor",
    );
}

fn record_for(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechSynthesisRecord> {
    scope
        .get_slot::<SpeechSynthesisStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update_record(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    update: impl FnOnce(&mut SpeechSynthesisRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<SpeechSynthesisStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    update(record);
    true
}

fn return_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&SpeechSynthesisRecord) -> bool,
) {
    let Some(record) = record_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, select(&record)).into());
}

fn get_pending(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_state(scope, arguments, result, |record| record.queue.len() > 1);
}

fn get_speaking(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_state(scope, arguments, result, |record| {
        !record.queue.is_empty() && !record.paused
    });
}

fn get_paused(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_state(scope, arguments, result, |record| record.paused);
}

fn get_on_voices_changed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = record_for(scope, arguments.this()).and_then(|record| record.on_voices_changed);
    if record_for(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::window_event_handler_support::return_handler(scope, handler, result);
}

fn set_on_voices_changed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if !update_record(scope, arguments.this(), |record| {
        record.on_voices_changed = handler
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn cancel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !update_record(scope, arguments.this(), |record| {
        record.queue.clear();
        record.paused = false;
        record.scheduled = false;
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_voices(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.voices.len() as i32);
    for (index, voice) in record.voices.iter().enumerate() {
        let voice = v8::Local::new(scope, voice);
        let _ = array.set_index(scope, index as u32, voice.into());
    }
    result.set(array.into());
}

fn pause(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !update_record(scope, arguments.this(), |record| {
        if !record.queue.is_empty() {
            record.paused = true;
        }
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn resume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let current = record_for(scope, arguments.this())
        .and_then(|record| record.queue.first().cloned())
        .map(|utterance| v8::Local::new(scope, &utterance));
    if !update_record(scope, arguments.this(), |record| record.paused = false) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(utterance) = current {
        super::speech_synthesis_utterance::fire(scope, utterance, "resume");
    }
    schedule_processing(scope, arguments.this().get_identity_hash().get());
}

fn speak(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record_for(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'speak' on 'SpeechSynthesis': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(utterance) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        throw_utterance_type(scope);
        return;
    };
    if !super::speech_synthesis_utterance::is_instance(scope, utterance) {
        throw_utterance_type(scope);
        return;
    }
    let utterance = v8::Global::new(scope, utterance);
    let synthesis_id = arguments.this().get_identity_hash().get();
    let _ = update_record(scope, arguments.this(), |record| {
        record.queue.push(utterance)
    });
    schedule_processing(scope, synthesis_id);
}

fn preload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record_for(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'preload' on 'SpeechSynthesis': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let Ok(sequence) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'preload' on 'SpeechSynthesis': The provided value cannot be converted to a sequence.",
        );
        return;
    };
    let count = sequence.length() as usize;
    let _ = update_record(scope, arguments.this(), |record| {
        record.preload_count = count
    });
}

fn throw_utterance_type(scope: &mut v8::PinScope<'_, '_>) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to execute 'speak' on 'SpeechSynthesis': parameter 1 is not of type 'SpeechSynthesisUtterance'.",
    );
}

fn schedule_processing(scope: &mut v8::PinScope<'_, '_>, synthesis_id: i32) {
    let should_schedule = scope
        .get_slot_mut::<SpeechSynthesisStore>()
        .and_then(|store| store.records.get_mut(&synthesis_id))
        .is_some_and(|record| {
            if record.scheduled || record.paused || record.queue.is_empty() {
                false
            } else {
                record.scheduled = true;
                true
            }
        });
    if !should_schedule {
        return;
    }
    let data = v8::Integer::new(scope, synthesis_id);
    if let Some(task) = v8::Function::builder(process_queue)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        scope.enqueue_microtask(task);
    }
}

fn process_queue(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(synthesis_id) = arguments.data().int32_value(scope) else {
        return;
    };
    let Some((synthesis, utterance, paused)) = scope
        .get_slot_mut::<SpeechSynthesisStore>()
        .and_then(|store| store.records.get_mut(&synthesis_id))
        .and_then(|record| {
            record.scheduled = false;
            let synthesis = record.object.clone()?;
            let utterance = record.queue.first()?.clone();
            Some((synthesis, utterance, record.paused))
        })
    else {
        return;
    };
    if paused {
        return;
    }
    let utterance = v8::Local::new(scope, &utterance);
    super::speech_synthesis_utterance::fire(scope, utterance, "start");
    super::speech_synthesis_utterance::fire(scope, utterance, "end");
    if let Some(record) = scope
        .get_slot_mut::<SpeechSynthesisStore>()
        .and_then(|store| store.records.get_mut(&synthesis_id))
    {
        if !record.queue.is_empty() {
            record.queue.remove(0);
        }
    }
    let _ = synthesis;
    schedule_processing(scope, synthesis_id);
}
