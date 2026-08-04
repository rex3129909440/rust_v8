use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct SpeechSynthesisEventRecord {
    pub(crate) utterance: v8::Global<v8::Object>,
    pub(crate) char_index: u32,
    pub(crate) char_length: u32,
    pub(crate) elapsed_time: f64,
    pub(crate) name: String,
}

#[derive(Default)]
pub(crate) struct SpeechSynthesisEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SpeechSynthesisEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechSynthesisEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechSynthesisEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<SpeechSynthesisEventStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechSynthesisEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::speech_synthesis_event_utterance_property::define(scope, prototype)?;
    super::speech_synthesis_event_char_index_property::define(scope, prototype)?;
    super::speech_synthesis_event_char_length_property::define(scope, prototype)?;
    super::speech_synthesis_event_elapsed_time_property::define(scope, prototype)?;
    super::speech_synthesis_event_name_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechSynthesisEventStore>()
        .ok_or_else(|| "SpeechSynthesisEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SpeechSynthesisEvent': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to construct 'SpeechSynthesisEvent': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        throw_required_utterance(scope, "SpeechSynthesisEvent");
        return;
    };
    let Some(utterance) = object_property(scope, init, "utterance") else {
        throw_required_utterance(scope, "SpeechSynthesisEvent");
        return;
    };
    if !super::speech_synthesis_utterance::is_instance(scope, utterance) {
        throw_required_utterance(scope, "SpeechSynthesisEvent");
        return;
    }
    let char_index = integer_property(scope, init, "charIndex");
    let char_length = integer_property(scope, init, "charLength");
    let elapsed_time = number_property(scope, init, "elapsedTime");
    let name = string_property(scope, init, "name");
    let bubbles = super::event::boolean_property(scope, init, "bubbles");
    let cancelable = super::event::boolean_property(scope, init, "cancelable");
    let composed = super::event::boolean_property(scope, init, "composed");
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    attach(
        scope,
        arguments.this(),
        utterance,
        char_index,
        char_length,
        elapsed_time,
        name,
    );
    result.set(arguments.this().into());
}

pub(crate) fn throw_required_utterance(scope: &mut v8::PinScope<'_, '_>, interface: &str) {
    crate::webidl::throw_type_error(
        scope,
        &format!(
            "Failed to construct '{interface}': Failed to read the 'utterance' property: Required member is undefined."
        ),
    )
}

pub(crate) fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    v8::Local::<v8::Object>::try_from(value_property(scope, object, name)?).ok()
}

pub(crate) fn integer_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> u32 {
    value_property(scope, object, name)
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.integer_value(scope))
        .unwrap_or(0)
        .clamp(0, u32::MAX as i64) as u32
}
pub(crate) fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> f64 {
    value_property(scope, object, name)
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(0.0)
}
pub(crate) fn string_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    value_property(scope, object, name)
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    utterance: v8::Local<'_, v8::Object>,
    char_index: u32,
    char_length: u32,
    elapsed_time: f64,
    name: String,
) {
    let record = SpeechSynthesisEventRecord {
        utterance: v8::Global::new(scope, utterance),
        char_index,
        char_length,
        elapsed_time,
        name,
    };
    scope
        .get_slot_mut::<SpeechSynthesisEventStore>()
        .expect("SpeechSynthesisEvent state")
        .records
        .insert(event.get_identity_hash().get(), record);
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    utterance: v8::Local<'_, v8::Object>,
    char_index: u32,
    char_length: u32,
    elapsed_time: f64,
    name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create SpeechSynthesisEvent".to_owned());
    }
    super::event::attach(scope, event, event_type.to_owned(), false, false, false);
    attach(
        scope,
        event,
        utterance,
        char_index,
        char_length,
        elapsed_time,
        name.to_owned(),
    );
    Ok(event)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechSynthesisEventRecord> {
    scope
        .get_slot::<SpeechSynthesisEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_utterance(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.utterance).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_char_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.char_index).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_char_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.char_length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_elapsed_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.elapsed_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.name) {
        result.set(value.into());
    }
}
