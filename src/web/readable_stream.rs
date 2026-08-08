use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadableState {
    Readable,
    Closed,
    Errored,
}

#[derive(Clone)]
pub(crate) struct ReadableRecord {
    pub locked: bool,
    pub byte_stream: bool,
    pub state: ReadableState,
    pub queue: VecDeque<v8::Global<v8::Value>>,
    pub stored_error: Option<v8::Global<v8::Value>>,
    source: Option<v8::Global<v8::Object>>,
    cancel_callback: Option<v8::Global<v8::Function>>,
    controller: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct ReadableStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ReadableRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ReadableStreamStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ReadableStream", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ReadableStreamStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ReadableStream",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "locked", get_locked)?;
    crate::webidl::define_method(scope, prototype, "cancel", 0, cancel)?;
    crate::webidl::define_method(scope, prototype, "getReader", 0, get_reader)?;
    crate::webidl::define_method(scope, prototype, "pipeThrough", 1, pipe_through)?;
    crate::webidl::define_method(scope, prototype, "pipeTo", 1, pipe_to)?;
    crate::webidl::define_method(scope, prototype, "tee", 0, tee)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_async_iterator_alias(scope, prototype, "values")?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ReadableStreamStore>()
        .ok_or_else(|| "ReadableStream state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_empty<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_with_source(scope, None)
}

fn create_with_source<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let stream = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, stream, prototype.into()) != Some(true) {
        return Err("cannot create ReadableStream".to_owned());
    }
    let byte_stream = source.is_some_and(|object| {
        let Some(key) = v8::String::new(scope, "type") else {
            return false;
        };
        object
            .get(scope, key.into())
            .is_some_and(|value| crate::webidl::value_to_string(scope, value) == "bytes")
    });
    let controller = if byte_stream {
        super::readable_byte_stream_controller::create(scope, stream)?
    } else {
        super::readable_stream_default_controller::create(scope, stream)?
    };
    let cancel_callback = source.and_then(|object| function_property(scope, object, "cancel"));
    let start_callback = source.and_then(|object| function_property(scope, object, "start"));
    let source_global = source.map(|object| v8::Global::new(scope, object));
    let controller_global = v8::Global::new(scope, controller);
    scope
        .get_slot_mut::<ReadableStreamStore>()
        .ok_or_else(|| "ReadableStream state was not prepared".to_owned())?
        .records
        .insert(
            stream.get_identity_hash().get(),
            ReadableRecord {
                locked: false,
                byte_stream,
                state: ReadableState::Readable,
                queue: VecDeque::new(),
                stored_error: None,
                source: source_global,
                cancel_callback,
                controller: controller_global,
            },
        );
    if let (Some(source), Some(start)) = (source, start_callback) {
        let callback = v8::Local::new(scope, &start);
        let _ = callback.call(scope, source.into(), &[controller.into()]);
    }
    Ok(stream)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReadableStream': use the new operator",
        );
        return;
    }
    let source = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    match attach_constructed(scope, arguments.this(), source) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn attach_constructed(
    scope: &mut v8::PinScope<'_, '_>,
    stream: v8::Local<'_, v8::Object>,
    source: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let byte_stream = source.is_some_and(|object| {
        let Some(key) = v8::String::new(scope, "type") else {
            return false;
        };
        object
            .get(scope, key.into())
            .is_some_and(|value| crate::webidl::value_to_string(scope, value) == "bytes")
    });
    let controller = if byte_stream {
        match super::readable_byte_stream_controller::create(scope, stream) {
            Ok(controller) => controller,
            Err(message) => return Err(message),
        }
    } else {
        super::readable_stream_default_controller::create(scope, stream)?
    };
    let cancel_callback = source.and_then(|object| function_property(scope, object, "cancel"));
    let start_callback = source.and_then(|object| function_property(scope, object, "start"));
    let source_global = source.map(|object| v8::Global::new(scope, object));
    let controller_global = v8::Global::new(scope, controller);
    scope
        .get_slot_mut::<ReadableStreamStore>()
        .ok_or_else(|| "ReadableStream state was not prepared".to_owned())?
        .records
        .insert(
            stream.get_identity_hash().get(),
            ReadableRecord {
                locked: false,
                byte_stream,
                state: ReadableState::Readable,
                queue: VecDeque::new(),
                stored_error: None,
                source: source_global,
                cancel_callback,
                controller: controller_global,
            },
        );
    if let (Some(source), Some(start)) = (source, start_callback) {
        let callback = v8::Local::new(scope, &start);
        let _ = callback.call(scope, source.into(), &[controller.into()]);
    }
    Ok(())
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> Option<ReadableRecord> {
    scope
        .get_slot::<ReadableStreamStore>()?
        .records
        .get(&stream.get_identity_hash().get())
        .cloned()
}

pub(crate) fn set_locked(
    scope: &mut v8::PinScope<'_, '_>,
    stream: v8::Local<'_, v8::Object>,
    locked: bool,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<ReadableStreamStore>()
        .and_then(|store| store.records.get_mut(&stream.get_identity_hash().get()))
    else {
        return false;
    };
    record.locked = locked;
    true
}

pub(crate) fn enqueue(
    scope: &mut v8::PinScope<'_, '_>,
    stream: v8::Local<'_, v8::Object>,
    chunk: v8::Local<'_, v8::Value>,
) -> bool {
    let chunk = v8::Global::new(scope, chunk);
    let Some(record) = scope
        .get_slot_mut::<ReadableStreamStore>()
        .and_then(|store| store.records.get_mut(&stream.get_identity_hash().get()))
    else {
        return false;
    };
    if record.state != ReadableState::Readable {
        return false;
    }
    record.queue.push_back(chunk);
    true
}

pub(crate) fn close(scope: &mut v8::PinScope<'_, '_>, stream: v8::Local<'_, v8::Object>) -> bool {
    let Some(record) = scope
        .get_slot_mut::<ReadableStreamStore>()
        .and_then(|store| store.records.get_mut(&stream.get_identity_hash().get()))
    else {
        return false;
    };
    if record.state == ReadableState::Readable {
        record.state = ReadableState::Closed;
    }
    true
}

pub(crate) fn error(
    scope: &mut v8::PinScope<'_, '_>,
    stream: v8::Local<'_, v8::Object>,
    reason: v8::Local<'_, v8::Value>,
) -> bool {
    let reason = v8::Global::new(scope, reason);
    let Some(record) = scope
        .get_slot_mut::<ReadableStreamStore>()
        .and_then(|store| store.records.get_mut(&stream.get_identity_hash().get()))
    else {
        return false;
    };
    record.state = ReadableState::Errored;
    record.stored_error = Some(reason);
    record.queue.clear();
    true
}

pub(crate) fn read<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let identity = stream.get_identity_hash().get();
    let (state, stored_error, value) = {
        let Some(record) = scope
            .get_slot_mut::<ReadableStreamStore>()
            .and_then(|store| store.records.get_mut(&identity))
        else {
            return Err("Illegal invocation".to_owned());
        };
        (
            record.state,
            record.stored_error.clone(),
            record.queue.pop_front(),
        )
    };
    if state == ReadableState::Errored {
        let reason = stored_error
            .as_ref()
            .map(|value| v8::Local::new(scope, value))
            .unwrap_or_else(|| v8::undefined(scope).into());
        return super::writable_stream::rejected_promise(scope, reason);
    }
    let done = value.is_none() && state == ReadableState::Closed;
    let object = v8::Object::new(scope);
    define_data(
        scope,
        object,
        "value",
        value
            .as_ref()
            .map(|value| v8::Local::new(scope, value))
            .unwrap_or_else(|| v8::undefined(scope).into()),
    );
    define_data(scope, object, "done", v8::Boolean::new(scope, done).into());
    super::writable_stream::resolved_promise(scope, object.into())
}

fn get_locked(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.locked).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn cancel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.locked {
        let message = v8::String::new(scope, "Cannot cancel a locked stream")
            .map(Into::into)
            .unwrap_or_else(|| v8::undefined(scope).into());
        if let Ok(promise) = super::writable_stream::rejected_promise(scope, message) {
            result.set(promise.into());
        }
        return;
    }
    let returned = if let Some(callback) = record.cancel_callback {
        let callback = v8::Local::new(scope, &callback);
        let receiver = record
            .source
            .as_ref()
            .map(|source| v8::Local::new(scope, source).into())
            .unwrap_or_else(|| v8::undefined(scope).into());
        callback
            .call(scope, receiver, &[arguments.get(0)])
            .unwrap_or_else(|| v8::undefined(scope).into())
    } else {
        v8::undefined(scope).into()
    };
    close(scope, arguments.this());
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, returned) {
        result.set(promise.into());
    }
}

fn get_reader(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let byob = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|options| {
            let key = v8::String::new(scope, "mode")?;
            options.get(scope, key.into())
        })
        .is_some_and(|value| crate::webidl::value_to_string(scope, value) == "byob");
    let reader = if byob {
        super::readable_stream_byob_reader::create(scope, arguments.this())
    } else {
        super::readable_stream_default_reader::create(scope, arguments.this())
    };
    match reader {
        Ok(reader) => result.set(reader.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn pipe_through(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(transform) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "pipeThrough requires a transform pair");
        return;
    };
    let Some(writable_key) = v8::String::new(scope, "writable") else {
        return;
    };
    let Some(readable_key) = v8::String::new(scope, "readable") else {
        return;
    };
    let Some(writable) = transform
        .get(scope, writable_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
    else {
        crate::webidl::throw_type_error(scope, "transform has no writable side");
        return;
    };
    pipe_all(scope, arguments.this(), writable);
    if let Some(readable) = transform.get(scope, readable_key.into()) {
        result.set(readable);
    }
}

fn pipe_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(destination) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "pipeTo requires a WritableStream");
        return;
    };
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    pipe_all(scope, arguments.this(), destination);
    let undefined = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
        result.set(promise.into());
    }
}

fn pipe_all(
    scope: &mut v8::PinScope<'_, '_>,
    source: v8::Local<'_, v8::Object>,
    destination: v8::Local<'_, v8::Object>,
) {
    let values = record(scope, source)
        .map(|record| record.queue)
        .unwrap_or_default();
    for value in values {
        let value = v8::Local::new(scope, &value);
        let _ = super::writable_stream::write_value(scope, destination, value);
    }
    let _ = super::writable_stream::close_stream(scope, destination);
    close(scope, source);
}

fn tee(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(left) = create_empty(scope) else {
        return;
    };
    let Ok(right) = create_empty(scope) else {
        return;
    };
    for value in record.queue {
        let value = v8::Local::new(scope, &value);
        enqueue(scope, left, value);
        enqueue(scope, right, value);
    }
    if record.state == ReadableState::Closed {
        close(scope, left);
        close(scope, right);
    }
    let pair = v8::Array::new(scope, 2);
    let _ = pair.set_index(scope, 0, left.into());
    let _ = pair.set_index(scope, 1, right.into());
    result.set(pair.into());
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::readable_stream_default_reader::create(scope, arguments.this()) {
        Ok(reader) => result.set(reader.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn function_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Function>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    let function = v8::Local::<v8::Function>::try_from(value).ok()?;
    Some(v8::Global::new(scope, function))
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ReadableStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
