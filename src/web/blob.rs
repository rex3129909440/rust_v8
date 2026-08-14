use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct BlobStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, BlobRecord>,
}

#[derive(Clone)]
struct BlobRecord {
    bytes: Vec<u8>,
    media_type: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BlobStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Blob", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<BlobStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Blob",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_method(scope, prototype, "arrayBuffer", 0, array_buffer)?;
    crate::webidl::define_method(scope, prototype, "slice", 0, slice)?;
    crate::webidl::define_method(scope, prototype, "stream", 0, stream)?;
    crate::webidl::define_method(scope, prototype, "text", 0, text)?;
    crate::webidl::define_method(scope, prototype, "bytes", 0, bytes)?;
    crate::webidl::define_method(scope, prototype, "textStream", 0, text_stream)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BlobStore>()
        .ok_or_else(|| "Blob state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
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
            "Failed to construct 'Blob': Please use the 'new' operator",
        );
        return;
    }
    let mut output = Vec::new();
    if !arguments.get(0).is_undefined() {
        let Ok(parts) = crate::webidl::sequence_values(scope, arguments.get(0)) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'Blob': The object must have a callable @@iterator property.",
            );
            return;
        };
        for part in parts {
            let part = v8::Local::new(scope, &part);
            append_part(scope, part, &mut output);
        }
    }
    let media_type = v8::Local::<v8::Object>::try_from(arguments.get(1))
        .ok()
        .and_then(|options| {
            let key = v8::String::new(scope, "type")?;
            options.get(scope, key.into())
        })
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value).to_ascii_lowercase())
        .unwrap_or_default();
    attach(scope, arguments.this(), output, media_type);
    result.set(arguments.this().into());
}

fn append_part(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    output: &mut Vec<u8>,
) {
    if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
        let store = buffer.get_backing_store();
        if let Some(data) = store.data() {
            let bytes = unsafe {
                std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), store.byte_length())
            };
            output.extend_from_slice(bytes);
        }
        return;
    }
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let mut bytes = vec![0_u8; view.byte_length()];
        let copied = view.copy_contents(&mut bytes);
        output.extend_from_slice(&bytes[..copied]);
        return;
    }
    output.extend_from_slice(crate::webidl::value_to_string(scope, value).as_bytes());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    bytes: Vec<u8>,
    media_type: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let blob = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, blob, prototype.into()) != Some(true) {
        return Err("cannot create Blob".to_owned());
    }
    attach(scope, blob, bytes, media_type.to_ascii_lowercase());
    Ok(blob)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    bytes: Vec<u8>,
    media_type: String,
) {
    if let Some(store) = scope.get_slot_mut::<BlobStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            BlobRecord { bytes, media_type },
        );
    }
}
fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<BlobRecord> {
    scope
        .get_slot::<BlobStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn byte_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(Vec<u8>, String)> {
    record(scope, object).map(|record| (record.bytes, record.media_type))
}
fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, v.bytes.len() as f64).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        if let Some(s) = v8::String::new(scope, &v.media_type) {
            r.set(s.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into())
    }
}
fn reject_illegal_invocation(
    scope: &mut v8::PinScope<'_, '_>,
    method: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let message = format!("Failed to execute '{method}' on 'Blob': Illegal invocation");
    if let Some(promise) = crate::webidl::rejected_type_error_promise(scope, &message) {
        result.set(promise.into());
    }
}
fn array_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        reject_illegal_invocation(scope, "arrayBuffer", r);
        return;
    };
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(v.bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    resolve(scope, buffer.into(), r)
}
fn bytes(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        reject_illegal_invocation(scope, "bytes", r);
        return;
    };
    let length = v.bytes.len();
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(v.bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    if let Some(array) = v8::Uint8Array::new(scope, buffer, 0, length) {
        resolve(scope, array.into(), r)
    }
}
fn text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        reject_illegal_invocation(scope, "text", r);
        return;
    };
    let string = String::from_utf8_lossy(&v.bytes);
    let value = v8::String::new(scope, &string)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into());
    resolve(scope, value, r)
}
fn slice(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let length = v.bytes.len() as i64;
    let start = a.get(0).integer_value(scope).unwrap_or(0);
    let end = if a.get(1).is_undefined() {
        length
    } else {
        a.get(1).integer_value(scope).unwrap_or(0)
    };
    let normalize=|value:i64| if value<0{(length+value).max(0)}else{value.min(length)} as usize;
    let start = normalize(start);
    let end = normalize(end).max(start);
    let media_type = if a.get(2).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, a.get(2)).to_ascii_lowercase()
    };
    if let Ok(blob) = create(scope, v.bytes[start..end].to_vec(), &media_type) {
        r.set(blob.into())
    }
}
fn stream(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(stream) = super::readable_stream::create_empty(scope) else {
        return;
    };
    let length = v.bytes.len();
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(v.bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    if let Some(array) = v8::Uint8Array::new(scope, buffer, 0, length) {
        let _ = super::readable_stream::enqueue(scope, stream, array.into());
    }
    let _ = super::readable_stream::close(scope, stream);
    r.set(stream.into())
}

fn text_stream(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(stream) = super::readable_stream::create_empty(scope) else {
        return;
    };
    let text = String::from_utf8_lossy(&v.bytes);
    if let Some(value) = v8::String::new(scope, &text) {
        let _ = super::readable_stream::enqueue(scope, stream, value.into());
    }
    let _ = super::readable_stream::close(scope, stream);
    r.set(stream.into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<BlobStore>() {
        store.constructors.remove(&realm_id);
    }
}
