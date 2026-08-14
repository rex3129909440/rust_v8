use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct SpeechSynthesisErrorEventRecord {
    pub(crate) error: String,
}

#[derive(Default)]
pub(crate) struct SpeechSynthesisErrorEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SpeechSynthesisErrorEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechSynthesisErrorEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechSynthesisErrorEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<SpeechSynthesisErrorEventStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechSynthesisErrorEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::speech_synthesis_error_event_error_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::speech_synthesis_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechSynthesisErrorEventStore>()
        .ok_or_else(|| "SpeechSynthesisErrorEvent state was not prepared".to_owned())?
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
            "Failed to construct 'SpeechSynthesisErrorEvent': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to construct 'SpeechSynthesisErrorEvent': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SpeechSynthesisErrorEvent': The provided value is not of type 'SpeechSynthesisErrorEventInit'.",
        );
        return;
    };
    let Some(utterance) = object_property(scope, init, "utterance") else {
        super::speech_synthesis_event::throw_required_utterance(scope, "SpeechSynthesisErrorEvent");
        return;
    };
    if !super::speech_synthesis_utterance::is_instance(scope, utterance) {
        super::speech_synthesis_event::throw_required_utterance(scope, "SpeechSynthesisErrorEvent");
        return;
    }
    let char_index = integer_property(scope, init, "charIndex");
    let char_length = integer_property(scope, init, "charLength");
    let elapsed_time = number_property(scope, init, "elapsedTime");
    let name = string_property(scope, init, "name");
    let error = string_property(scope, init, "error");
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        super::event::boolean_property(scope, init, "bubbles"),
        super::event::boolean_property(scope, init, "cancelable"),
        super::event::boolean_property(scope, init, "composed"),
    );
    super::speech_synthesis_event::attach(
        scope,
        arguments.this(),
        utterance,
        char_index,
        char_length,
        elapsed_time,
        name,
    );
    scope
        .get_slot_mut::<SpeechSynthesisErrorEventStore>()
        .expect("SpeechSynthesisErrorEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            SpeechSynthesisErrorEventRecord { error },
        );
    result.set(arguments.this().into());
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

pub(crate) fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot::<SpeechSynthesisErrorEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.error) {
        result.set(value.into());
    }
}
