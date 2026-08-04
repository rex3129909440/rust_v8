use std::collections::HashMap;

#[derive(Clone)]
struct ChunkRecord {
    chunk_type: String,
    timestamp: i64,
    duration: Option<u64>,
    bytes: Vec<u8>,
    decoded_frame: Option<super::video_frame::VideoFrameEncodingSnapshot>,
}

#[derive(Default)]
pub(crate) struct EncodedVideoChunkStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ChunkRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(EncodedVideoChunkStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "EncodedVideoChunk", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<EncodedVideoChunkStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let c = crate::webidl::create_function(
        scope,
        "EncodedVideoChunk",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, p, "timestamp", get_timestamp)?;
    crate::webidl::define_readonly_accessor(scope, p, "duration", get_duration)?;
    crate::webidl::define_readonly_accessor(scope, p, "byteLength", get_byte_length)?;
    crate::webidl::define_method(scope, p, "copyTo", 1, copy_to)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<EncodedVideoChunkStore>()
        .ok_or_else(|| "EncodedVideoChunk state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'EncodedVideoChunk': 1 argument required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "The chunk init dictionary is required");
        return;
    };
    let Some(chunk_type) = string_member(scope, init, "type") else {
        crate::webidl::throw_type_error(scope, "Required member type is undefined");
        return;
    };
    if !matches!(chunk_type.as_str(), "key" | "delta") {
        crate::webidl::throw_type_error(scope, "The provided value is not a valid enum value");
        return;
    }
    let Some(timestamp) = number_member(scope, init, "timestamp").map(|v| v as i64) else {
        crate::webidl::throw_type_error(scope, "Required member timestamp is undefined");
        return;
    };
    let duration = number_member(scope, init, "duration").map(|v| v.max(0.0) as u64);
    let Some(data) = member(scope, init, "data").and_then(|v| bytes(scope, v)) else {
        crate::webidl::throw_type_error(scope, "Required member data is not a BufferSource");
        return;
    };
    scope
        .get_slot_mut::<EncodedVideoChunkStore>()
        .expect("EncodedVideoChunk state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            ChunkRecord {
                chunk_type,
                timestamp,
                duration,
                bytes: data,
                decoded_frame: None,
            },
        );
    r.set(a.this().into())
}

pub(crate) fn create_from_video_frame<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    snapshot: super::video_frame::VideoFrameEncodingSnapshot,
    key_frame: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create EncodedVideoChunk".to_owned());
    }
    scope
        .get_slot_mut::<EncodedVideoChunkStore>()
        .ok_or_else(|| "EncodedVideoChunk state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ChunkRecord {
                chunk_type: if key_frame { "key" } else { "delta" }.to_owned(),
                timestamp: snapshot.timestamp as i64,
                duration: snapshot.duration.map(|duration| duration.max(0.0) as u64),
                bytes: snapshot.bytes.clone(),
                decoded_frame: Some(snapshot),
            },
        );
    Ok(object)
}

pub(crate) fn is_encoded_video_chunk(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<EncodedVideoChunkStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

pub(crate) fn decoded_frame_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<super::video_frame::VideoFrameEncodingSnapshot> {
    record(scope, object)?.decoded_frame
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
fn bytes(s: &v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>) -> Option<Vec<u8>> {
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
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ChunkRecord> {
    s.get_slot::<EncodedVideoChunkStore>()?
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
    if let Some(store) = scope.get_slot_mut::<EncodedVideoChunkStore>() {
        store.constructor.remove(realm_id);
    }
}
