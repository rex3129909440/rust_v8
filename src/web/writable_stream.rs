use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WritableStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, StreamRecord>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StreamState {
    Writable,
    Closed,
    Errored,
}

#[derive(Clone)]
pub(crate) struct StreamRecord {
    pub locked: bool,
    pub state: StreamState,
    pub stored_error: Option<v8::Global<v8::Value>>,
    sink: Option<v8::Global<v8::Object>>,
    write_callback: Option<v8::Global<v8::Function>>,
    close_callback: Option<v8::Global<v8::Function>>,
    abort_callback: Option<v8::Global<v8::Function>>,
    controller: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WritableStreamStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WritableStream", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<WritableStreamStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WritableStream",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "locked", get_locked)?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "getWriter", 0, get_writer)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WritableStreamStore>()
        .ok_or_else(|| "WritableStream state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_empty<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    constructor
        .new_instance(scope, &[])
        .ok_or_else(|| "cannot create WritableStream".to_owned())
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WritableStream': Please use the 'new' operator",
        );
        return;
    }
    let object = arguments.this();
    let controller = match super::writable_stream_default_controller::create(scope, object) {
        Ok(controller) => controller,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let sink = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let write_callback = sink.and_then(|sink| function_property(scope, sink, "write"));
    let close_callback = sink.and_then(|sink| function_property(scope, sink, "close"));
    let abort_callback = sink.and_then(|sink| function_property(scope, sink, "abort"));
    let start_callback = sink.and_then(|sink| function_property(scope, sink, "start"));
    let sink_global = sink.map(|sink| v8::Global::new(scope, sink));
    let controller_global = v8::Global::new(scope, controller);
    scope
        .get_slot_mut::<WritableStreamStore>()
        .expect("WritableStream state")
        .records
        .insert(
            object.get_identity_hash().get(),
            StreamRecord {
                locked: false,
                state: StreamState::Writable,
                stored_error: None,
                sink: sink_global,
                write_callback,
                close_callback,
                abort_callback,
                controller: controller_global,
            },
        );
    if let (Some(sink), Some(start_callback)) = (sink, start_callback) {
        let callback = v8::Local::new(scope, &start_callback);
        let _ = callback.call(scope, sink.into(), &[controller.into()]);
    }
    result.set(object.into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<StreamRecord> {
    scope
        .get_slot::<WritableStreamStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut StreamRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<WritableStreamStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(record);
    true
}

pub(crate) fn set_locked(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    locked: bool,
) -> bool {
    update(scope, object, |record| record.locked = locked)
}

pub(crate) fn error_stream(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    reason: v8::Local<'_, v8::Value>,
) -> bool {
    let reason = v8::Global::new(scope, reason);
    update(scope, object, |record| {
        if record.state == StreamState::Writable {
            record.state = StreamState::Errored;
            record.stored_error = Some(reason);
        }
    })
}

pub(crate) fn write_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let record = record(scope, object).ok_or_else(|| "Illegal invocation".to_owned())?;
    if record.state != StreamState::Writable {
        return rejected_promise(
            scope,
            record
                .stored_error
                .as_ref()
                .map(|error| v8::Local::new(scope, error))
                .unwrap_or_else(|| string_value(scope, "The stream is not writable")),
        );
    }
    let returned = if let Some(callback) = record.write_callback {
        let callback = v8::Local::new(scope, &callback);
        let receiver = record
            .sink
            .as_ref()
            .map(|sink| v8::Local::new(scope, sink).into())
            .unwrap_or_else(|| v8::undefined(scope).into());
        let controller = v8::Local::new(scope, &record.controller);
        callback
            .call(scope, receiver, &[value, controller.into()])
            .unwrap_or_else(|| v8::undefined(scope).into())
    } else {
        v8::undefined(scope).into()
    };
    resolved_promise(scope, returned)
}

pub(crate) fn close_stream<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let record = record(scope, object).ok_or_else(|| "Illegal invocation".to_owned())?;
    if record.state != StreamState::Writable {
        return rejected_promise(
            scope,
            record
                .stored_error
                .as_ref()
                .map(|error| v8::Local::new(scope, error))
                .unwrap_or_else(|| string_value(scope, "The stream cannot be closed")),
        );
    }
    let returned = call_sink(scope, &record, record.close_callback.as_ref(), &[]);
    update(scope, object, |record| record.state = StreamState::Closed);
    resolved_promise(scope, returned)
}

pub(crate) fn abort_stream<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    reason: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let record = record(scope, object).ok_or_else(|| "Illegal invocation".to_owned())?;
    if record.state == StreamState::Closed {
        return resolved_promise(scope, v8::undefined(scope).into());
    }
    let reason = v8::Global::new(scope, reason);
    let reason = v8::Local::new(scope, &reason);
    let returned = call_sink(scope, &record, record.abort_callback.as_ref(), &[reason]);
    error_stream(scope, object, reason);
    resolved_promise(scope, returned)
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

fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some_and(|record| record.locked) {
        let message = string_value(scope, "Cannot abort a locked stream");
        if let Ok(promise) = rejected_promise(scope, message) {
            result.set(promise.into());
        }
        return;
    }
    match abort_stream(scope, arguments.this(), arguments.get(0)) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some_and(|record| record.locked) {
        let message = string_value(scope, "Cannot close a locked stream");
        if let Ok(promise) = rejected_promise(scope, message) {
            result.set(promise.into());
        }
        return;
    }
    match close_stream(scope, arguments.this()) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_writer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::writable_stream_default_writer::create(scope, arguments.this()) {
        Ok(writer) => result.set(writer.into()),
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

fn call_sink<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: &StreamRecord,
    callback: Option<&v8::Global<v8::Function>>,
    arguments: &[v8::Local<'s, v8::Value>],
) -> v8::Local<'s, v8::Value> {
    let Some(callback) = callback else {
        return v8::undefined(scope).into();
    };
    let callback = v8::Local::new(scope, callback);
    let receiver = record
        .sink
        .as_ref()
        .map(|sink| v8::Local::new(scope, sink).into())
        .unwrap_or_else(|| v8::undefined(scope).into());
    callback
        .call(scope, receiver, arguments)
        .unwrap_or_else(|| v8::undefined(scope).into())
}

pub(crate) fn resolved_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let resolver =
        v8::PromiseResolver::new(scope).ok_or_else(|| "cannot create promise".to_owned())?;
    let promise = resolver.get_promise(scope);
    let _ = resolver.resolve(scope, value);
    Ok(promise)
}

pub(crate) fn rejected_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    reason: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let resolver =
        v8::PromiseResolver::new(scope).ok_or_else(|| "cannot create promise".to_owned())?;
    let promise = resolver.get_promise(scope);
    let _ = resolver.reject(scope, reason);
    Ok(promise)
}

pub(crate) fn pending_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let resolver =
        v8::PromiseResolver::new(scope).ok_or_else(|| "cannot create promise".to_owned())?;
    Ok(resolver.get_promise(scope))
}

fn string_value<'s>(scope: &v8::PinScope<'s, '_>, value: &str) -> v8::Local<'s, v8::Value> {
    v8::String::new(scope, value)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into())
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WritableStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
