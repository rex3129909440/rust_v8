use std::collections::HashMap;

const EMPTY: i32 = 0;
const LOADING: i32 = 1;
const DONE: i32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ReaderHandler {
    LoadStart,
    Progress,
    Load,
    Abort,
    Error,
    LoadEnd,
}

#[derive(Clone)]
struct FileReaderRecord {
    object: v8::Global<v8::Object>,
    ready_state: i32,
    result: Option<v8::Global<v8::Value>>,
    error: Option<v8::Global<v8::Object>>,
    handlers: HashMap<ReaderHandler, v8::Global<v8::Function>>,
    pending: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct FileReaderStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, FileReaderRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FileReaderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FileReader", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FileReaderStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FileReader",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readyState", get_ready_state)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "result", get_result)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "error", get_error)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onloadstart",
        get_on_load_start,
        set_on_load_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onprogress",
        get_on_progress,
        set_on_progress,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onload", get_on_load, set_on_load)?;
    crate::webidl::define_accessor(scope, prototype, "onabort", get_on_abort, set_on_abort)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_on_error, set_on_error)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onloadend",
        get_on_load_end,
        set_on_load_end,
    )?;
    define_constants(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "readAsArrayBuffer",
        1,
        read_as_array_buffer,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "readAsBinaryString",
        1,
        read_as_binary_string,
    )?;
    crate::webidl::define_method(scope, prototype, "readAsDataURL", 1, read_as_data_url)?;
    crate::webidl::define_method(scope, prototype, "readAsText", 1, read_as_text)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FileReaderStore>()
        .ok_or_else(|| "FileReader state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "EMPTY", EMPTY)?;
    crate::webidl::define_constant(scope, object, "LOADING", LOADING)?;
    crate::webidl::define_constant(scope, object, "DONE", DONE)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'FileReader': Please use the 'new' operator",
        );
        return;
    }
    super::event_target::attach(scope, arguments.this());
    let record = FileReaderRecord {
        object: v8::Global::new(scope, arguments.this()),
        ready_state: EMPTY,
        result: None,
        error: None,
        handlers: HashMap::new(),
        pending: None,
    };
    scope
        .get_slot_mut::<FileReaderStore>()
        .expect("FileReader state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<FileReaderRecord> {
    scope
        .get_slot::<FileReaderStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_ready_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.ready_state).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_result(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.result {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.error {
            Some(value) => result.set(v8::Local::new(scope, &value).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    handler: ReaderHandler,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.handlers.get(&handler) {
            Some(value) => result.set(v8::Local::new(scope, value).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    handler: ReaderHandler,
) {
    let value = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|function| v8::Global::new(scope, function));
    let Some(record) = scope.get_slot_mut::<FileReaderStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = value {
        record.handlers.insert(handler, value);
    } else {
        record.handlers.remove(&handler);
    }
}

fn get_on_load_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ReaderHandler::LoadStart)
}
fn set_on_load_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ReaderHandler::LoadStart)
}
fn get_on_progress(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ReaderHandler::Progress)
}
fn set_on_progress(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ReaderHandler::Progress)
}
fn get_on_load(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ReaderHandler::Load)
}
fn set_on_load(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ReaderHandler::Load)
}
fn get_on_abort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ReaderHandler::Abort)
}
fn set_on_abort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ReaderHandler::Abort)
}
fn get_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ReaderHandler::Error)
}
fn set_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ReaderHandler::Error)
}
fn get_on_load_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ReaderHandler::LoadEnd)
}
fn set_on_load_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ReaderHandler::LoadEnd)
}

fn fire(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event_name: &str,
    handler_slot: ReaderHandler,
) {
    let event = super::event_target::create_event(scope, event_name);
    let handler =
        record(scope, target).and_then(|record| record.handlers.get(&handler_slot).cloned());
    if let Some(handler) = handler {
        let handler = v8::Local::new(scope, &handler);
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
    super::event_target::dispatch(scope, target, event);
}

fn begin_read(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    value: v8::Local<'_, v8::Value>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.ready_state == LOADING {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The object is already busy reading Blobs.".to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let pending = v8::Global::new(scope, value);
    if let Some(record) = scope
        .get_slot_mut::<FileReaderStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.ready_state = LOADING;
        record.result = None;
        record.error = None;
        record.pending = Some(pending);
    }
    fire(
        scope,
        arguments.this(),
        "loadstart",
        ReaderHandler::LoadStart,
    );
    let data = v8::Integer::new(scope, id);
    if let Some(task) = v8::Function::builder(deliver)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        scope.enqueue_microtask(task);
    }
}

fn blob_bytes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<(Vec<u8>, String)> {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return None;
    }
    let Ok(blob) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'Blob'");
        return None;
    };
    match super::blob::byte_snapshot(scope, blob) {
        Some(value) => Some(value),
        None => {
            crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'Blob'");
            None
        }
    }
}

fn read_as_array_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some((bytes, _)) = blob_bytes(scope, &arguments) else {
        return;
    };
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    begin_read(scope, arguments, buffer.into());
}

fn read_as_binary_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some((bytes, _)) = blob_bytes(scope, &arguments) else {
        return;
    };
    let text: String = bytes.into_iter().map(char::from).collect();
    if let Some(value) = v8::String::new(scope, &text) {
        begin_read(scope, arguments, value.into());
    }
}

fn read_as_data_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some((bytes, media_type)) = blob_bytes(scope, &arguments) else {
        return;
    };
    let media_type = if media_type.is_empty() {
        "application/octet-stream"
    } else {
        &media_type
    };
    let text = format!("data:{media_type};base64,{}", encode_base64(&bytes));
    if let Some(value) = v8::String::new(scope, &text) {
        begin_read(scope, arguments, value.into());
    }
}

fn read_as_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some((bytes, _)) = blob_bytes(scope, &arguments) else {
        return;
    };
    let text = String::from_utf8_lossy(&bytes);
    if let Some(value) = v8::String::new(scope, &text) {
        begin_read(scope, arguments, value.into());
    }
}

fn deliver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(id) = arguments.data().int32_value(scope) else {
        return;
    };
    let completed = {
        let Some(record) = scope
            .get_slot_mut::<FileReaderStore>()
            .and_then(|store| store.records.get_mut(&id))
        else {
            return;
        };
        let Some(value) = record.pending.take() else {
            return;
        };
        record.result = Some(value);
        record.ready_state = DONE;
        record.object.clone()
    };
    let target = v8::Local::new(scope, &completed);
    fire(scope, target, "progress", ReaderHandler::Progress);
    fire(scope, target, "load", ReaderHandler::Load);
    fire(scope, target, "loadend", ReaderHandler::LoadEnd);
}

fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(record) = scope.get_slot_mut::<FileReaderStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.result = None;
        record.pending = None;
        record.ready_state = DONE;
    }
    if current.ready_state == LOADING {
        fire(scope, arguments.this(), "abort", ReaderHandler::Abort);
        fire(scope, arguments.this(), "loadend", ReaderHandler::LoadEnd);
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;
    while index < bytes.len() {
        let first = bytes[index];
        let second = bytes.get(index + 1).copied();
        let third = bytes.get(index + 2).copied();
        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 0x03) << 4) | second.unwrap_or(0) >> 4) as usize] as char);
        output.push(match second {
            Some(second) => {
                ALPHABET[(((second & 0x0f) << 2) | third.unwrap_or(0) >> 6) as usize] as char
            }
            None => '=',
        });
        output.push(match third {
            Some(third) => ALPHABET[(third & 0x3f) as usize] as char,
            None => '=',
        });
        index += 3;
    }
    output
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileReaderStore>() {
        store.constructors.remove(&realm_id);
    }
}
