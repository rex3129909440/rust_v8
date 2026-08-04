use std::collections::HashMap;

#[derive(Clone, Copy)]
enum CompressionFormat {
    Deflate,
    DeflateRaw,
    Gzip,
}

#[derive(Clone)]
struct CompressionRecord {
    format: CompressionFormat,
    input: Vec<u8>,
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CompressionStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CompressionRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CompressionStreamStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CompressionStream", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CompressionStreamStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CompressionStream",
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
        .get_slot_mut::<CompressionStreamStore>()
        .ok_or_else(|| "CompressionStream state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "CompressionStream requires a format");
        return;
    }
    let format = match crate::webidl::value_to_string(scope, arguments.get(0)).as_str() {
        "deflate" => CompressionFormat::Deflate,
        "deflate-raw" => CompressionFormat::DeflateRaw,
        "gzip" => CompressionFormat::Gzip,
        _ => {
            crate::webidl::throw_type_error(scope, "Unsupported compression format");
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
    let Some(write) = v8::Function::builder(write_chunk)
        .data(arguments.this().into())
        .length(1)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    else {
        crate::webidl::throw_type_error(scope, "cannot create compression sink");
        return;
    };
    write.set_name(v8::String::new(scope, "write").expect("short string"));
    let Some(close) = v8::Function::builder(close_sink)
        .data(arguments.this().into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    else {
        crate::webidl::throw_type_error(scope, "cannot create compression sink");
        return;
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
        crate::webidl::throw_type_error(scope, "cannot create compression writable stream");
        return;
    };
    let readable = v8::Global::new(scope, readable);
    let writable = v8::Global::new(scope, writable);
    scope
        .get_slot_mut::<CompressionStreamStore>()
        .expect("CompressionStream state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CompressionRecord {
                format,
                input: Vec::new(),
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

fn target_identity(arguments: &v8::FunctionCallbackArguments<'_>) -> Option<i32> {
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
    let Some(target) = target_identity(&arguments) else {
        crate::webidl::throw_type_error(scope, "Invalid CompressionStream sink");
        return;
    };
    let Some(bytes) = bytes(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "CompressionStream chunks must be BufferSource");
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<CompressionStreamStore>()
        .and_then(|store| store.records.get_mut(&target))
    {
        record.input.extend_from_slice(&bytes);
    } else {
        crate::webidl::throw_type_error(scope, "Invalid CompressionStream sink");
    }
}

fn close_sink(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(target) = target_identity(&arguments) else {
        crate::webidl::throw_type_error(scope, "Invalid CompressionStream sink");
        return;
    };
    let Some(record) = scope
        .get_slot::<CompressionStreamStore>()
        .and_then(|store| store.records.get(&target))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Invalid CompressionStream sink");
        return;
    };
    let output = match record.format {
        CompressionFormat::Deflate => miniz_oxide::deflate::compress_to_vec_zlib(&record.input, 6),
        CompressionFormat::DeflateRaw => miniz_oxide::deflate::compress_to_vec(&record.input, 6),
        CompressionFormat::Gzip => gzip(&record.input),
    };
    let length = output.len();
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(output).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    let readable = v8::Local::new(scope, &record.readable);
    if let Some(view) = v8::Uint8Array::new(scope, buffer, 0, length) {
        super::readable_stream::enqueue(scope, readable, view.into());
    }
    super::readable_stream::close(scope, readable);
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn gzip(input: &[u8]) -> Vec<u8> {
    let compressed = miniz_oxide::deflate::compress_to_vec(input, 6);
    let mut output = Vec::with_capacity(compressed.len() + 18);
    output.extend_from_slice(&[0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255]);
    output.extend_from_slice(&compressed);
    output.extend_from_slice(&crc32(input).to_le_bytes());
    output.extend_from_slice(&(input.len() as u32).to_le_bytes());
    output
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CompressionRecord> {
    scope
        .get_slot::<CompressionStreamStore>()?
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
    if let Some(store) = scope.get_slot_mut::<CompressionStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
