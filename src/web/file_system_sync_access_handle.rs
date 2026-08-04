use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AccessRecord {
    bytes: Arc<Mutex<Vec<u8>>>,
    closed: bool,
}

#[derive(Default)]
pub(crate) struct FileSystemSyncAccessHandleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AccessRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FileSystemSyncAccessHandleStore::default());
}

pub(crate) fn install_in_worker_realm(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "FileSystemSyncAccessHandle", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<FileSystemSyncAccessHandleStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FileSystemSyncAccessHandle",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::file_system_sync_access_handle_close::define(scope, prototype)?;
    super::file_system_sync_access_handle_flush::define(scope, prototype)?;
    super::file_system_sync_access_handle_get_size::define(scope, prototype)?;
    super::file_system_sync_access_handle_read::define(scope, prototype)?;
    super::file_system_sync_access_handle_truncate::define(scope, prototype)?;
    super::file_system_sync_access_handle_write::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FileSystemSyncAccessHandleStore>()
        .ok_or_else(|| "FileSystemSyncAccessHandle state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    bytes: Arc<Mutex<Vec<u8>>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create FileSystemSyncAccessHandle".to_owned());
    }
    scope
        .get_slot_mut::<FileSystemSyncAccessHandleStore>()
        .ok_or_else(|| "FileSystemSyncAccessHandle state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AccessRecord {
                bytes,
                closed: false,
            },
        );
    Ok(object)
}

fn record_id(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<i32> {
    let id = object.get_identity_hash().get();
    let record = scope
        .get_slot::<FileSystemSyncAccessHandleStore>()?
        .records
        .get(&id)?;
    if record.closed {
        crate::webidl::throw_type_error(scope, "The access handle is closed");
        None
    } else {
        Some(id)
    }
}

pub(crate) fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    if let Some(record) = scope
        .get_slot_mut::<FileSystemSyncAccessHandleStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.closed = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn flush(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
) {
    let _ = record_id(scope, arguments.this());
}

pub(crate) fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(id) = record_id(scope, arguments.this()) else {
        return;
    };
    let size = scope
        .get_slot::<FileSystemSyncAccessHandleStore>()
        .and_then(|store| store.records.get(&id))
        .and_then(|record| record.bytes.lock().ok().map(|bytes| bytes.len()))
        .unwrap_or(0);
    result.set(v8::Number::new(scope, size as f64).into());
}

fn at_option(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> usize {
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        return 0;
    };
    let Some(key) = v8::String::new(scope, "at") else {
        return 0;
    };
    options
        .get(scope, key.into())
        .and_then(|value| value.integer_value(scope))
        .unwrap_or(0)
        .max(0) as usize
}

pub(crate) fn read(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(id) = record_id(scope, arguments.this()) else {
        return;
    };
    let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The first argument must be an ArrayBufferView");
        return;
    };
    let at = at_option(scope, arguments.get(1));
    let source = scope
        .get_slot::<FileSystemSyncAccessHandleStore>()
        .and_then(|store| store.records.get(&id))
        .and_then(|record| record.bytes.lock().ok().map(|bytes| bytes.clone()))
        .unwrap_or_default();
    let amount = view.byte_length().min(source.len().saturating_sub(at));
    if let Some(buffer) = view.buffer(scope) {
        let store = buffer.get_backing_store();
        if let Some(pointer) = store.data() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    source.as_ptr().add(at),
                    pointer.as_ptr().cast::<u8>().add(view.byte_offset()),
                    amount,
                );
            }
        }
    }
    result.set(v8::Number::new(scope, amount as f64).into());
}

pub(crate) fn write(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(id) = record_id(scope, arguments.this()) else {
        return;
    };
    let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The first argument must be an ArrayBufferView");
        return;
    };
    let mut input = vec![0_u8; view.byte_length()];
    let amount = view.copy_contents(&mut input);
    input.truncate(amount);
    let at = at_option(scope, arguments.get(1));
    if let Some(record) = scope
        .get_slot::<FileSystemSyncAccessHandleStore>()
        .and_then(|store| store.records.get(&id))
        .and_then(|record| record.bytes.lock().ok())
    {
        let mut bytes = record;
        let new_len = bytes.len().max(at + input.len());
        bytes.resize(new_len, 0);
        bytes[at..at + input.len()].copy_from_slice(&input);
    }
    result.set(v8::Number::new(scope, input.len() as f64).into());
}

pub(crate) fn truncate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
) {
    let Some(id) = record_id(scope, arguments.this()) else {
        return;
    };
    let size = arguments.get(0).integer_value(scope).unwrap_or(0).max(0) as usize;
    if let Some(mut bytes) = scope
        .get_slot::<FileSystemSyncAccessHandleStore>()
        .and_then(|store| store.records.get(&id))
        .and_then(|record| record.bytes.lock().ok())
    {
        bytes.resize(size, 0);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileSystemSyncAccessHandleStore>() {
        store.constructor.remove(realm_id);
    }
}
