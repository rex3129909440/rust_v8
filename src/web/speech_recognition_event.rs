use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct SpeechRecognitionEventRecord {
    pub(crate) result_index: u32,
    pub(crate) results: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct SpeechRecognitionEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SpeechRecognitionEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SpeechRecognitionEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SpeechRecognitionEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SpeechRecognitionEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SpeechRecognitionEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::speech_recognition_event_result_index_property::define(scope, prototype)?;
    super::speech_recognition_event_results_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SpeechRecognitionEventStore>()
        .ok_or_else(|| "SpeechRecognitionEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
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
            "Failed to construct 'SpeechRecognitionEvent': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SpeechRecognitionEvent': 1 argument required, but only 0 present.",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let result_index = number_property(scope, init, "resultIndex").max(0) as u32;
    let results = value_property(scope, init, "results").unwrap_or_else(|| v8::null(scope).into());
    if !results.is_null() && !results.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SpeechRecognitionEvent': Failed to convert value to 'SpeechRecognitionResultList'.",
        );
        return;
    }
    let bubbles =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "bubbles"));
    let cancelable =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "cancelable"));
    let composed =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "composed"));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let record = SpeechRecognitionEventRecord {
        result_index,
        results: v8::Global::new(scope, results),
    };
    scope
        .get_slot_mut::<SpeechRecognitionEventStore>()
        .expect("SpeechRecognitionEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

pub(crate) fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = object?;
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> i64 {
    value_property(scope, object, name)
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.integer_value(scope))
        .unwrap_or(0)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SpeechRecognitionEventRecord> {
    scope
        .get_slot::<SpeechRecognitionEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_result_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.result_index).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_results(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.results));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
