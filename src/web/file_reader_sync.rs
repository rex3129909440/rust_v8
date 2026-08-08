use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct FileReaderSyncStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FileReaderSyncStore::default());
}

pub(crate) fn install_in_worker_realm(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "FileReaderSync", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<FileReaderSyncStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FileReaderSync",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::file_reader_sync_read_as_array_buffer::define(scope, prototype)?;
    super::file_reader_sync_read_as_binary_string::define(scope, prototype)?;
    super::file_reader_sync_read_as_text::define(scope, prototype)?;
    super::file_reader_sync_read_as_data_url::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FileReaderSyncStore>()
        .ok_or_else(|| "FileReaderSync state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
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
            "Failed to construct 'FileReaderSync': Please use the 'new' operator",
        );
        return;
    }
    scope
        .get_slot_mut::<FileReaderSyncStore>()
        .expect("FileReaderSync state")
        .instances
        .insert(arguments.this().get_identity_hash().get());
    result.set(arguments.this().into());
}

#[derive(Clone, Copy)]
pub(crate) enum ReadKind {
    ArrayBuffer,
    BinaryString,
    Text,
    DataUrl,
}

pub(crate) fn read(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    kind: ReadKind,
    mut result: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<FileReaderSyncStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains(&arguments.this().get_identity_hash().get())
        })
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let Ok(blob) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Blob");
        return;
    };
    let Some((bytes, media_type)) = super::blob::byte_snapshot(scope, blob) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Blob");
        return;
    };
    match kind {
        ReadKind::ArrayBuffer => {
            let store = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
            result.set(v8::ArrayBuffer::with_backing_store(scope, &store).into());
        }
        ReadKind::BinaryString => {
            let text = bytes
                .iter()
                .map(|byte| char::from(*byte))
                .collect::<String>();
            if let Some(text) = v8::String::new(scope, &text) {
                result.set(text.into());
            }
        }
        ReadKind::Text => {
            let text = String::from_utf8_lossy(&bytes);
            if let Some(text) = v8::String::new(scope, &text) {
                result.set(text.into());
            }
        }
        ReadKind::DataUrl => {
            let media_type = if media_type.is_empty() {
                "application/octet-stream"
            } else {
                &media_type
            };
            let value = format!("data:{media_type};base64,{}", encode_base64(&bytes));
            if let Some(value) = v8::String::new(scope, &value) {
                result.set(value.into());
            }
        }
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 3) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            ALPHABET[(((second & 15) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[(third & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileReaderSyncStore>() {
        store.constructor.remove(realm_id);
    }
}
