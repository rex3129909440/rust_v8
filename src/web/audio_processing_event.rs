use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct AudioProcessingEventRecord {
    pub(crate) playback_time: f64,
    pub(crate) input_buffer: v8::Global<v8::Object>,
    pub(crate) output_buffer: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct AudioProcessingEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, AudioProcessingEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioProcessingEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioProcessingEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioProcessingEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioProcessingEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::audio_processing_event_playback_time_property::define(scope, prototype)?;
    super::audio_processing_event_input_buffer_property::define(scope, prototype)?;
    super::audio_processing_event_output_buffer_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioProcessingEventStore>()
        .ok_or_else(|| "AudioProcessingEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "AudioProcessingEvent requires a type and initializer",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "AudioProcessingEvent initializer must be an object",
        );
        return;
    };
    let Some(input_buffer) = property(scope, init, "inputBuffer")
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
    else {
        crate::webidl::throw_type_error(scope, "inputBuffer is required");
        return;
    };
    let Some(output_buffer) = property(scope, init, "outputBuffer")
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
    else {
        crate::webidl::throw_type_error(scope, "outputBuffer is required");
        return;
    };
    let playback_time = property(scope, init, "playbackTime")
        .and_then(|value| value.number_value(scope))
        .unwrap_or(0.0);
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let input_buffer = v8::Global::new(scope, input_buffer);
    let output_buffer = v8::Global::new(scope, output_buffer);
    scope
        .get_slot_mut::<AudioProcessingEventStore>()
        .expect("AudioProcessingEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            AudioProcessingEventRecord {
                playback_time,
                input_buffer,
                output_buffer,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioProcessingEventRecord> {
    scope
        .get_slot::<AudioProcessingEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_playback_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Number::new(s, v.playback_time).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_input_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.input_buffer).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_output_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.output_buffer).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
