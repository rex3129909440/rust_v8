use std::collections::HashMap;

#[derive(Clone)]
struct AudioDecoderRecord {
    state: String,
    queue_size: u32,
    output: v8::Global<v8::Function>,
    error: v8::Global<v8::Function>,
    ondequeue: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct AudioDecoderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioDecoderRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioDecoderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioDecoder", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<AudioDecoderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "AudioDecoder",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "decodeQueueSize", get_queue_size)?;
    crate::webidl::define_accessor(scope, prototype, "ondequeue", get_ondequeue, set_ondequeue)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "configure", 1, configure)?;
    crate::webidl::define_method(scope, prototype, "decode", 1, decode)?;
    crate::webidl::define_method(scope, prototype, "flush", 0, flush)?;
    crate::webidl::define_method(scope, prototype, "reset", 0, reset)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::audio_decoder_is_config_supported::define(scope, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioDecoderStore>()
        .ok_or_else(|| "AudioDecoder state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn callback_member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Function>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    v8::Local::<v8::Function>::try_from(value).ok()
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "AudioDecoder must be constructed");
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "AudioDecoder requires an init dictionary");
        return;
    };
    let Some(output) = callback_member(scope, init, "output") else {
        crate::webidl::throw_type_error(scope, "AudioDecoder output callback is required");
        return;
    };
    let Some(error) = callback_member(scope, init, "error") else {
        crate::webidl::throw_type_error(scope, "AudioDecoder error callback is required");
        return;
    };
    super::event_target::attach(scope, arguments.this());
    let output = v8::Global::new(scope, output);
    let error = v8::Global::new(scope, error);
    scope
        .get_slot_mut::<AudioDecoderStore>()
        .expect("AudioDecoder state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            AudioDecoderRecord {
                state: "unconfigured".to_owned(),
                queue_size: 0,
                output,
                error,
                ondequeue: None,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioDecoderRecord> {
    scope
        .get_slot::<AudioDecoderStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut AudioDecoderRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<AudioDecoderStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_queue_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.queue_size).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &record.state)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_ondequeue(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|record| record.ondequeue);
    super::window_event_handler_support::return_handler(scope, handler, result);
}

fn set_ondequeue(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.ondequeue = handler);
}

fn configure(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let configuration = super::webcodecs_config_support::dictionary(scope, arguments.get(0));
    if super::webcodecs_config_support::string_member(scope, configuration, "codec").is_none()
        || super::webcodecs_config_support::number_member(scope, configuration, "numberOfChannels")
            .is_none()
        || super::webcodecs_config_support::number_member(scope, configuration, "sampleRate")
            .is_none()
    {
        crate::webidl::throw_type_error(
            scope,
            "AudioDecoder configuration requires codec, numberOfChannels, and sampleRate",
        );
        return;
    }
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.state == "closed" {
        super::webcodecs_state::throw_invalid_state(
            scope,
            "Cannot call 'configure' on a closed codec.",
        );
        return;
    }
    update(scope, arguments.this(), |record| {
        record.state = "configured".to_owned();
        record.queue_size = 0;
    });
}

fn decode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.state != "configured" {
        super::webcodecs_state::throw_invalid_state(
            scope,
            "Cannot call 'decode' on an unconfigured codec.",
        );
        return;
    }
    let Ok(chunk) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        super::webcodecs_state::throw_argument_type(scope, "decode", "EncodedAudioChunk");
        return;
    };
    if !super::encoded_audio_chunk::is_encoded_audio_chunk(scope, chunk) {
        super::webcodecs_state::throw_argument_type(scope, "decode", "EncodedAudioChunk");
        return;
    }
    update(scope, arguments.this(), |record| record.queue_size += 1);
    if let Some(snapshot) = super::encoded_audio_chunk::decoded_audio_snapshot(scope, chunk) {
        if let Ok(audio_data) = super::audio_data::create_from_encoding_snapshot(scope, snapshot) {
            let output = v8::Local::new(scope, &current.output);
            let receiver = v8::undefined(scope);
            let _ = output.call(scope, receiver.into(), &[audio_data.into()]);
        }
    } else if let Some(error) = super::webcodecs_state::encoding_error(
        scope,
        "The encoded audio chunk could not be decoded by the deterministic codec backend.",
    ) {
        let callback = v8::Local::new(scope, &current.error);
        let receiver = v8::undefined(scope);
        let _ = callback.call(scope, receiver.into(), &[error.into()]);
        update(scope, arguments.this(), |record| {
            record.state = "closed".to_owned();
        });
    }
    update(scope, arguments.this(), |record| {
        record.queue_size = record.queue_size.saturating_sub(1)
    });
    super::webcodecs_state::fire_dequeue(scope, arguments.this(), current.ondequeue);
}

fn flush(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.state != "configured" {
        super::webcodecs_state::reject_invalid_state(
            scope,
            "Cannot call 'flush' on an unconfigured codec.",
            result,
        );
        return;
    }
    if let Ok(promise) =
        super::writable_stream::resolved_promise(scope, v8::undefined(scope).into())
    {
        result.set(promise.into());
    }
}

fn reset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.state == "closed" {
        super::webcodecs_state::throw_invalid_state(scope, "Cannot reset a closed codec.");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.state = "unconfigured".to_owned();
        record.queue_size = 0;
    });
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.state == "closed" {
        super::webcodecs_state::throw_invalid_state(scope, "Codec is already closed.");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.state = "closed".to_owned();
        record.queue_size = 0;
    });
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<AudioDecoderStore>() {
        store.constructor.remove(realm_id);
    }
}
