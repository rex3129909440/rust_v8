use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ReadableStreamByobReaderStore {
    constructor: crate::webidl::RealmConstructor,
    streams: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ReadableStreamByobReaderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ReadableStreamBYOBReader", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ReadableStreamByobReaderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ReadableStreamBYOBReader",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "read", 1, read)?;
    crate::webidl::define_method(scope, prototype, "releaseLock", 0, release_lock)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "closed", get_closed)?;
    crate::webidl::define_method(scope, prototype, "cancel", 0, cancel)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ReadableStreamByobReaderStore>()
        .ok_or_else(|| "ReadableStreamBYOBReader state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let record = super::readable_stream::record(scope, stream)
        .ok_or_else(|| "The supplied value is not a ReadableStream".to_owned())?;
    if !record.byte_stream {
        return Err("Cannot use a BYOB reader with a non-byte stream".to_owned());
    }
    if record.locked {
        return Err("The stream is already locked".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let reader = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, reader, prototype.into()) != Some(true) {
        return Err("cannot create ReadableStreamBYOBReader".to_owned());
    }
    super::readable_stream::set_locked(scope, stream, true);
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<ReadableStreamByobReaderStore>()
        .ok_or_else(|| "ReadableStreamBYOBReader state was not prepared".to_owned())?
        .streams
        .insert(reader.get_identity_hash().get(), Some(stream));
    Ok(reader)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReadableStreamBYOBReader': 1 argument required",
        );
        return;
    }
    let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReadableStreamBYOBReader': parameter 1 is not of type 'ReadableStream'.",
        );
        return;
    };
    let Some(record) = super::readable_stream::record(scope, stream) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReadableStreamBYOBReader': parameter 1 is not of type 'ReadableStream'.",
        );
        return;
    };
    if !record.byte_stream || record.locked {
        crate::webidl::throw_type_error(scope, "The byte stream is unavailable");
        return;
    }
    super::readable_stream::set_locked(scope, stream, true);
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<ReadableStreamByobReaderStore>()
        .expect("ReadableStreamBYOBReader state")
        .streams
        .insert(arguments.this().get_identity_hash().get(), Some(stream));
    result.set(arguments.this().into());
}

fn stream(
    scope: &v8::PinScope<'_, '_>,
    reader: v8::Local<'_, v8::Object>,
) -> Option<Option<v8::Global<v8::Object>>> {
    scope
        .get_slot::<ReadableStreamByobReaderStore>()?
        .streams
        .get(&reader.get_identity_hash().get())
        .cloned()
}

fn read(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let stream = match stream(scope, arguments.this()) {
        None => {
            crate::webidl::reject_illegal_invocation_promise(
                scope,
                "ReadableStreamBYOBReader",
                "read",
                result,
            );
            return;
        }
        Some(None) => {
            if let Some(promise) =
                crate::webidl::rejected_type_error_promise(scope, "Reader has no stream")
            {
                result.set(promise.into());
            }
            return;
        }
        Some(Some(stream)) => stream,
    };
    let Ok(view) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "read requires an ArrayBufferView");
        return;
    };
    let stream = v8::Local::new(scope, &stream);
    let Some(record) = super::readable_stream::record(scope, stream) else {
        crate::webidl::throw_type_error(scope, "Reader has no stream");
        return;
    };
    if !record.queue.is_empty() || record.state != super::readable_stream::ReadableState::Readable {
        match super::readable_stream::read(scope, stream) {
            Ok(promise) => result.set(promise.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let promise = resolver.get_promise(scope);
    match super::readable_stream_byob_request::create(scope, stream, view, resolver) {
        Ok(_) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn release_lock(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = stream(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(stream) = current {
        let stream = v8::Local::new(scope, &stream);
        super::readable_stream::set_locked(scope, stream, false);
    }
    if let Some(entry) = scope
        .get_slot_mut::<ReadableStreamByobReaderStore>()
        .and_then(|store| {
            store
                .streams
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *entry = None;
    }
}

fn get_closed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(current) = stream(scope, arguments.this()) else {
        if let Some(promise) = crate::webidl::rejected_type_error_promise(
            scope,
            "Failed to read the 'closed' property from 'ReadableStreamBYOBReader': Illegal invocation",
        ) {
            result.set(promise.into());
        }
        return;
    };
    let value = v8::undefined(scope);
    let promise = if current.is_some() {
        super::writable_stream::resolved_promise(scope, value.into())
    } else {
        super::writable_stream::rejected_promise(scope, value.into())
    };
    if let Ok(promise) = promise {
        result.set(promise.into());
    }
}

fn cancel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let stream = match stream(scope, arguments.this()) {
        None => {
            crate::webidl::reject_illegal_invocation_promise(
                scope,
                "ReadableStreamBYOBReader",
                "cancel",
                result,
            );
            return;
        }
        Some(None) => {
            if let Some(promise) =
                crate::webidl::rejected_type_error_promise(scope, "Reader has no stream")
            {
                result.set(promise.into());
            }
            return;
        }
        Some(Some(stream)) => stream,
    };
    let stream = v8::Local::new(scope, &stream);
    super::readable_stream::close(scope, stream);
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ReadableStreamByobReaderStore>() {
        store.constructor.remove(realm_id);
    }
}
