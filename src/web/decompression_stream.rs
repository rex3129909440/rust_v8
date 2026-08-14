use std::collections::HashMap;

#[derive(Clone, Copy)]
enum CompressionFormat {
    Deflate,
    DeflateRaw,
    Gzip,
}

#[derive(Clone)]
struct DecompressionRecord {
    format: CompressionFormat,
    compressed: Vec<u8>,
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct DecompressionStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DecompressionRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DecompressionStreamStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DecompressionStream", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<DecompressionStreamStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DecompressionStream",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readable", get_readable)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "writable", get_writable)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DecompressionStreamStore>()
        .ok_or_else(|| "DecompressionStream state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "DecompressionStream requires a format");
        return;
    }
    let Some(format_name) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    let format = match format_name.as_str() {
        "deflate" => CompressionFormat::Deflate,
        "deflate-raw" => CompressionFormat::DeflateRaw,
        "gzip" => CompressionFormat::Gzip,
        _ => {
            crate::webidl::throw_type_error(
                scope,
                &format!(
                    "Failed to construct 'DecompressionStream': Unsupported compression format: '{format_name}'"
                ),
            );
            return;
        }
    };
    let readable = match super::readable_stream::create_empty(scope) {
        Ok(readable) => readable,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let sink = v8::Object::new(scope);
    let write = match v8::Function::builder(write_chunk)
        .data(arguments.this().into())
        .length(1)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        Some(write) => write,
        None => {
            crate::webidl::throw_type_error(scope, "cannot create decompression sink");
            return;
        }
    };
    write.set_name(v8::String::new(scope, "write").expect("short string"));
    let close = match v8::Function::builder(close_sink)
        .data(arguments.this().into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        Some(close) => close,
        None => {
            crate::webidl::throw_type_error(scope, "cannot create decompression sink");
            return;
        }
    };
    close.set_name(v8::String::new(scope, "close").expect("short string"));
    define_function(scope, sink, "write", write);
    define_function(scope, sink, "close", close);
    let writable_constructor = match super::writable_stream::ensure_constructor(scope) {
        Ok(constructor) => constructor,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let Some(writable) = writable_constructor.new_instance(scope, &[sink.into()]) else {
        crate::webidl::throw_type_error(scope, "cannot create decompression writable stream");
        return;
    };
    let readable = v8::Global::new(scope, readable);
    let writable = v8::Global::new(scope, writable);
    scope
        .get_slot_mut::<DecompressionStreamStore>()
        .expect("DecompressionStream state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            DecompressionRecord {
                format,
                compressed: Vec::new(),
                readable,
                writable,
            },
        );
    result.set(arguments.this().into());
}

fn define_function(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    function: v8::Local<'_, v8::Function>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), function.into());
    }
}

fn target_identity(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<i32> {
    v8::Local::<v8::Object>::try_from(arguments.data())
        .ok()
        .map(|object| object.get_identity_hash().get())
}

fn bytes(value: v8::Local<'_, v8::Value>) -> Option<Vec<u8>> {
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let mut bytes = vec![0_u8; view.byte_length()];
        let copied = view.copy_contents(&mut bytes);
        bytes.truncate(copied);
        return Some(bytes);
    }
    let buffer = v8::Local::<v8::ArrayBuffer>::try_from(value).ok()?;
    let backing = buffer.get_backing_store();
    let data = backing.data()?;
    Some(
        unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), backing.byte_length()) }
            .to_vec(),
    )
}

fn write_chunk(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(target) = target_identity(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Invalid DecompressionStream sink");
        return;
    };
    let Some(bytes) = bytes(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "DecompressionStream chunks must be BufferSource");
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<DecompressionStreamStore>()
        .and_then(|store| store.records.get_mut(&target))
    {
        record.compressed.extend_from_slice(&bytes);
    } else {
        crate::webidl::throw_type_error(scope, "Invalid DecompressionStream sink");
    }
}

fn close_sink(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(target) = target_identity(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Invalid DecompressionStream sink");
        return;
    };
    let Some(record) = scope
        .get_slot::<DecompressionStreamStore>()
        .and_then(|store| store.records.get(&target))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Invalid DecompressionStream sink");
        return;
    };
    let decoded: Result<Vec<u8>, ()> = match record.format {
        CompressionFormat::Deflate => {
            miniz_oxide::inflate::decompress_to_vec_zlib(&record.compressed).map_err(|_| ())
        }
        CompressionFormat::DeflateRaw => {
            miniz_oxide::inflate::decompress_to_vec(&record.compressed).map_err(|_| ())
        }
        CompressionFormat::Gzip => gzip_payload(&record.compressed)
            .ok_or(())
            .and_then(|payload| miniz_oxide::inflate::decompress_to_vec(payload).map_err(|_| ())),
    };
    let readable = v8::Local::new(scope, &record.readable);
    match decoded {
        Ok(bytes) => {
            let length = bytes.len();
            let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
            let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
            if let Some(view) = v8::Uint8Array::new(scope, buffer, 0, length) {
                super::readable_stream::enqueue(scope, readable, view.into());
            }
            super::readable_stream::close(scope, readable);
        }
        Err(_) => {
            let message =
                v8::String::new(scope, "The compressed data is invalid").expect("short string");
            let error = v8::Exception::type_error(scope, message);
            super::readable_stream::error(scope, readable, error);
            scope.throw_exception(error);
        }
    }
}

fn gzip_payload(bytes: &[u8]) -> Option<&[u8]> {
    if bytes.len() < 18 || bytes[0] != 0x1f || bytes[1] != 0x8b || bytes[2] != 8 {
        return None;
    }
    let flags = bytes[3];
    let mut index = 10_usize;
    if flags & 4 != 0 {
        let length = u16::from_le_bytes([*bytes.get(index)?, *bytes.get(index + 1)?]) as usize;
        index = index.checked_add(2 + length)?;
    }
    if flags & 8 != 0 {
        index = skip_zero_terminated(bytes, index)?;
    }
    if flags & 16 != 0 {
        index = skip_zero_terminated(bytes, index)?;
    }
    if flags & 2 != 0 {
        index = index.checked_add(2)?;
    }
    let end = bytes.len().checked_sub(8)?;
    (index <= end).then_some(&bytes[index..end])
}

fn skip_zero_terminated(bytes: &[u8], start: usize) -> Option<usize> {
    let relative = bytes.get(start..)?.iter().position(|value| *value == 0)?;
    start.checked_add(relative + 1)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DecompressionRecord> {
    scope
        .get_slot::<DecompressionStreamStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_readable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.readable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.writable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DecompressionStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
