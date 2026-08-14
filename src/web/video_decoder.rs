use std::collections::HashMap;
#[derive(Clone)]
struct Codec {
    state: String,
    queue: u32,
    output: v8::Global<v8::Function>,
    error: v8::Global<v8::Function>,
    handler: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct VideoDecoderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Codec>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(VideoDecoderStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "VideoDecoder", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<VideoDecoderStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "VideoDecoder",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "decodeQueueSize", queue)?;
    crate::webidl::define_accessor(s, p, "ondequeue", get_handler, set_handler)?;
    crate::webidl::define_readonly_accessor(s, p, "state", state)?;
    crate::webidl::define_method(s, p, "close", 0, close)?;
    crate::webidl::define_method(s, p, "configure", 1, configure)?;
    crate::webidl::define_method(s, p, "decode", 1, decode)?;
    crate::webidl::define_method(s, p, "flush", 0, flush)?;
    crate::webidl::define_method(s, p, "reset", 0, reset)?;
    crate::webidl::finish_constructor(s, p, c)?;
    super::video_decoder_is_config_supported::define(s, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let g = v8::Global::new(s, c);
    s.get_slot_mut::<VideoDecoderStore>()
        .ok_or_else(|| "VideoDecoder state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
    Ok(c)
}
fn callback<'s>(
    s: &v8::PinScope<'s, '_>,
    o: v8::Local<'s, v8::Object>,
    n: &str,
) -> Option<v8::Local<'s, v8::Function>> {
    v8::String::new(s, n)
        .and_then(|k| o.get(s, k.into()))
        .and_then(|v| v8::Local::<v8::Function>::try_from(v).ok())
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "VideoDecoder requires callbacks");
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'VideoDecoder': The provided value is not of type 'VideoDecoderInit'.",
        );
        return;
    };
    let Some(error) = callback(s, init, "error") else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'VideoDecoder': Failed to read the 'error' property from 'VideoDecoderInit': Required member is undefined.",
        );
        return;
    };
    let Some(output) = callback(s, init, "output") else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'VideoDecoder': Failed to read the 'output' property from 'VideoDecoderInit': Required member is undefined.",
        );
        return;
    };
    super::event_target::attach(s, a.this());
    let output = v8::Global::new(s, output);
    let error = v8::Global::new(s, error);
    s.get_slot_mut::<VideoDecoderStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            Codec {
                state: "unconfigured".to_owned(),
                queue: 0,
                output,
                error,
                handler: None,
            },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Codec> {
    s.get_slot::<VideoDecoderStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn queue(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.queue).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(v) = v8::String::new(s, &v.state)
    {
        r.set(v.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.handler, r)
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<VideoDecoderStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn configure(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let configuration = super::webcodecs_config_support::dictionary(s, a.get(0));
    if super::webcodecs_config_support::string_member(s, configuration, "codec").is_none() {
        crate::webidl::throw_type_error(s, "VideoDecoder configuration requires codec");
        return;
    }
    let Some(current) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if current.state == "closed" {
        super::webcodecs_state::throw_invalid_state(
            s,
            "Cannot call 'configure' on a closed codec.",
        );
        return;
    }
    if let Some(v) = s
        .get_slot_mut::<VideoDecoderStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.queue = 0;
        v.state = "configured".to_owned()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn reset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if current.state == "closed" {
        super::webcodecs_state::throw_invalid_state(s, "Cannot reset a closed codec.");
        return;
    }
    if let Some(v) = s
        .get_slot_mut::<VideoDecoderStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.queue = 0;
        v.state = "unconfigured".to_owned()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if current.state == "closed" {
        super::webcodecs_state::throw_invalid_state(s, "Codec is already closed.");
        return;
    }
    if let Some(v) = s
        .get_slot_mut::<VideoDecoderStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.queue = 0;
        v.state = "closed".to_owned()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn decode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if v.state != "configured" {
        super::webcodecs_state::throw_invalid_state(
            s,
            "Cannot call 'decode' on an unconfigured codec.",
        );
        return;
    }
    let Ok(chunk) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        super::webcodecs_state::throw_argument_type(s, "decode", "EncodedVideoChunk");
        return;
    };
    if !super::encoded_video_chunk::is_encoded_video_chunk(s, chunk) {
        super::webcodecs_state::throw_argument_type(s, "decode", "EncodedVideoChunk");
        return;
    }
    if let Some(current) = s
        .get_slot_mut::<VideoDecoderStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        current.queue += 1;
    }
    if let Some(snapshot) = super::encoded_video_chunk::decoded_frame_snapshot(s, chunk) {
        if let Ok(frame) = super::video_frame::create_from_encoding_snapshot(s, snapshot) {
            let output = v8::Local::new(s, &v.output);
            let receiver = v8::undefined(s);
            let _ = output.call(s, receiver.into(), &[frame.into()]);
        }
    } else if let Some(error) = super::webcodecs_state::encoding_error(
        s,
        "The encoded video chunk could not be decoded by the deterministic codec backend.",
    ) {
        let callback = v8::Local::new(s, &v.error);
        let receiver = v8::undefined(s);
        let _ = callback.call(s, receiver.into(), &[error.into()]);
        if let Some(current) = s
            .get_slot_mut::<VideoDecoderStore>()
            .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
        {
            current.state = "closed".to_owned();
        }
    }
    if let Some(current) = s
        .get_slot_mut::<VideoDecoderStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        current.queue = current.queue.saturating_sub(1);
    }
    super::webcodecs_state::fire_dequeue(s, a.this(), v.handler);
}
fn flush(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(current) = record(s, a.this()) else {
        crate::webidl::reject_illegal_invocation_promise(s, "VideoDecoder", "flush", r);
        return;
    };
    if current.state != "configured" {
        super::webcodecs_state::reject_invalid_state(
            s,
            "Cannot call 'flush' on an unconfigured codec.",
            r,
        );
        return;
    }
    if let Ok(p) = super::writable_stream::resolved_promise(s, v8::undefined(s).into()) {
        r.set(p.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<VideoDecoderStore>() {
        store.constructor.remove(realm_id);
    }
}
