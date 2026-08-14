use std::collections::HashMap;

#[derive(Clone)]
struct AudioChunkRecord {
    chunk_type: String,
    timestamp: i64,
    bytes: Vec<u8>,
    duration: Option<u64>,
    decoded_audio: Option<super::audio_data::AudioDataEncodingSnapshot>,
}

#[derive(Default)]
pub(crate) struct EncodedAudioChunkStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioChunkRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(EncodedAudioChunkStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "EncodedAudioChunk", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(c) = s
        .get_slot::<EncodedAudioChunkStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "EncodedAudioChunk",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "type", get_type)?;
    crate::webidl::define_readonly_accessor(s, p, "timestamp", get_timestamp)?;
    crate::webidl::define_readonly_accessor(s, p, "byteLength", get_byte_length)?;
    crate::webidl::define_readonly_accessor(s, p, "duration", get_duration)?;
    crate::webidl::define_method(s, p, "copyTo", 1, copy_to)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<EncodedAudioChunkStore>()
        .ok_or_else(|| "EncodedAudioChunk state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'EncodedAudioChunk': 1 argument required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'EncodedAudioChunk': The provided value is not of type 'EncodedAudioChunkInit'.",
        );
        return;
    };
    let Some(data_value) = member(s, init, "data").filter(|value| !value.is_undefined()) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'EncodedAudioChunk': Failed to read the 'data' property from 'EncodedAudioChunkInit': Required member is undefined.",
        );
        return;
    };
    let Some(chunk_type) = string_member(s, init, "type") else {
        crate::webidl::throw_type_error(s, "Required member type is undefined");
        return;
    };
    if !matches!(chunk_type.as_str(), "key" | "delta") {
        crate::webidl::throw_type_error(s, "The provided value is not a valid enum value");
        return;
    }
    let Some(timestamp) = number_member(s, init, "timestamp").map(|v| v as i64) else {
        crate::webidl::throw_type_error(s, "Required member timestamp is undefined");
        return;
    };
    let duration = number_member(s, init, "duration").map(|v| v.max(0.0) as u64);
    let Some(data) = bytes(s, data_value) else {
        crate::webidl::throw_type_error(s, "Required member data is not a BufferSource");
        return;
    };
    s.get_slot_mut::<EncodedAudioChunkStore>()
        .expect("EncodedAudioChunk state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            AudioChunkRecord {
                chunk_type,
                timestamp,
                bytes: data,
                duration,
                decoded_audio: None,
            },
        );
    r.set(a.this().into())
}

pub(crate) fn create_from_audio_data<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    snapshot: super::audio_data::AudioDataEncodingSnapshot,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create EncodedAudioChunk".to_owned());
    }
    let duration =
        (snapshot.number_of_frames as f64 * 1_000_000.0 / snapshot.sample_rate).trunc() as u64;
    scope
        .get_slot_mut::<EncodedAudioChunkStore>()
        .ok_or_else(|| "EncodedAudioChunk state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AudioChunkRecord {
                chunk_type: "key".to_owned(),
                timestamp: snapshot.timestamp,
                bytes: snapshot.bytes.clone(),
                duration: Some(duration),
                decoded_audio: Some(snapshot),
            },
        );
    Ok(object)
}

pub(crate) fn is_encoded_audio_chunk(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<EncodedAudioChunkStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

pub(crate) fn decoded_audio_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<super::audio_data::AudioDataEncodingSnapshot> {
    record(scope, object)?.decoded_audio
}
fn member<'s>(
    s: &v8::PinScope<'s, '_>,
    o: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(s, name)?;
    let v = o.get(s, key.into())?;
    (!v.is_undefined()).then_some(v)
}
fn string_member(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    member(s, o, name).map(|v| crate::webidl::value_to_string(s, v))
}
fn number_member(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    member(s, o, name)?.number_value(s)
}
fn bytes(_: &v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>) -> Option<Vec<u8>> {
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(v) {
        let mut out = vec![0; view.byte_length()];
        let n = view.copy_contents(&mut out);
        out.truncate(n);
        return Some(out);
    }
    let buffer = v8::Local::<v8::ArrayBuffer>::try_from(v).ok()?;
    let store = buffer.get_backing_store();
    let data = store.data()?;
    Some(
        unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), store.byte_length()) }
            .to_vec(),
    )
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<AudioChunkRecord> {
    s.get_slot::<EncodedAudioChunkStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        if let Some(v) = v8::String::new(s, &x.chunk_type) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.timestamp as f64).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_byte_length(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, x.bytes.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        if let Some(v) = x.duration {
            r.set(v8::Number::new(s, v as f64).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn copy_to(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let Ok(out) = v8::Local::<v8::Uint8Array>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "destination is not a BufferSource");
        return;
    };
    if out.byte_length() < x.bytes.len() {
        crate::webidl::throw_type_error(s, "destination is too small");
        return;
    }
    if !x.bytes.is_empty() {
        unsafe {
            std::ptr::copy_nonoverlapping(x.bytes.as_ptr(), out.data().cast::<u8>(), x.bytes.len())
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<EncodedAudioChunkStore>() {
        store.constructor.remove(realm_id);
    }
}
