use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WritableStreamDefaultWriterStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WriterRecord>,
}

#[derive(Clone)]
struct WriterRecord {
    stream: v8::Global<v8::Object>,
    released: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WritableStreamDefaultWriterStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WritableStreamDefaultWriter", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<WritableStreamDefaultWriterStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WritableStreamDefaultWriter",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "closed", get_closed)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "desiredSize", get_desired_size)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ready", get_ready)?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "releaseLock", 0, release_lock)?;
    crate::webidl::define_method(scope, prototype, "write", 0, write)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WritableStreamDefaultWriterStore>()
        .ok_or_else(|| "WritableStreamDefaultWriter state was not prepared".to_owned())?
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
            "Failed to construct 'WritableStreamDefaultWriter': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WritableStreamDefaultWriter': 1 argument required",
        );
        return;
    }
    let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The provided value is not a WritableStream");
        return;
    };
    if let Err(message) = attach(scope, arguments.this(), stream) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let writer = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, writer, prototype.into()) != Some(true) {
        return Err("cannot create WritableStreamDefaultWriter".to_owned());
    }
    attach(scope, writer, stream)?;
    Ok(writer)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    writer: v8::Local<'_, v8::Object>,
    stream: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let record = super::writable_stream::record(scope, stream)
        .ok_or_else(|| "The provided value is not a WritableStream".to_owned())?;
    if record.locked {
        return Err("This WritableStream is already locked".to_owned());
    }
    if !super::writable_stream::set_locked(scope, stream, true) {
        return Err("cannot lock WritableStream".to_owned());
    }
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<WritableStreamDefaultWriterStore>()
        .ok_or_else(|| "WritableStreamDefaultWriter state was not prepared".to_owned())?
        .records
        .insert(
            writer.get_identity_hash().get(),
            WriterRecord {
                stream,
                released: false,
            },
        );
    Ok(())
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<WriterRecord> {
    scope
        .get_slot::<WritableStreamDefaultWriterStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn active_stream<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    writer: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let Some(record) = record(scope, writer) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return None;
    };
    if record.released {
        crate::webidl::throw_type_error(scope, "The writer lock has been released");
        return None;
    }
    Some(v8::Local::new(scope, &record.stream))
}

fn get_closed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = active_stream(scope, arguments.this()) else {
        return;
    };
    let Some(record) = super::writable_stream::record(scope, stream) else {
        return;
    };
    let promise = match record.state {
        super::writable_stream::StreamState::Closed => {
            let undefined = v8::undefined(scope);
            super::writable_stream::resolved_promise(scope, undefined.into())
        }
        super::writable_stream::StreamState::Errored => {
            let reason = record
                .stored_error
                .as_ref()
                .map(|error| v8::Local::new(scope, error))
                .unwrap_or_else(|| v8::undefined(scope).into());
            super::writable_stream::rejected_promise(scope, reason)
        }
        super::writable_stream::StreamState::Writable => {
            super::writable_stream::pending_promise(scope)
        }
    };
    if let Ok(promise) = promise {
        result.set(promise.into());
    }
}

fn get_desired_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = active_stream(scope, arguments.this()) else {
        return;
    };
    let Some(record) = super::writable_stream::record(scope, stream) else {
        return;
    };
    match record.state {
        super::writable_stream::StreamState::Writable => {
            result.set(v8::Number::new(scope, 1.0).into())
        }
        super::writable_stream::StreamState::Closed => {
            result.set(v8::Number::new(scope, 0.0).into())
        }
        super::writable_stream::StreamState::Errored => result.set(v8::null(scope).into()),
    }
}

fn get_ready(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = active_stream(scope, arguments.this()) else {
        return;
    };
    let Some(record) = super::writable_stream::record(scope, stream) else {
        return;
    };
    let promise = if record.state == super::writable_stream::StreamState::Errored {
        let reason = record
            .stored_error
            .as_ref()
            .map(|error| v8::Local::new(scope, error))
            .unwrap_or_else(|| v8::undefined(scope).into());
        super::writable_stream::rejected_promise(scope, reason)
    } else {
        let undefined = v8::undefined(scope);
        super::writable_stream::resolved_promise(scope, undefined.into())
    };
    if let Ok(promise) = promise {
        result.set(promise.into());
    }
}

fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = active_stream(scope, arguments.this()) else {
        return;
    };
    match super::writable_stream::abort_stream(scope, stream, arguments.get(0)) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = active_stream(scope, arguments.this()) else {
        return;
    };
    match super::writable_stream::close_stream(scope, stream) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn release_lock(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.released {
        return;
    }
    let stream = v8::Local::new(scope, &snapshot.stream);
    super::writable_stream::set_locked(scope, stream, false);
    if let Some(record) = scope
        .get_slot_mut::<WritableStreamDefaultWriterStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.released = true;
    }
}

fn write(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = active_stream(scope, arguments.this()) else {
        return;
    };
    match super::writable_stream::write_value(scope, stream, arguments.get(0)) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WritableStreamDefaultWriterStore>() {
        store.constructor.remove(realm_id);
    }
}
