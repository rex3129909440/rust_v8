use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ReadableStreamDefaultReaderStore {
    constructor: crate::webidl::RealmConstructor,
    streams: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ReadableStreamDefaultReaderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ReadableStreamDefaultReader", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ReadableStreamDefaultReaderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ReadableStreamDefaultReader",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "read", 0, read)?;
    crate::webidl::define_method(scope, prototype, "releaseLock", 0, release_lock)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "closed", get_closed)?;
    crate::webidl::define_method(scope, prototype, "cancel", 0, cancel)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ReadableStreamDefaultReaderStore>()
        .ok_or_else(|| "ReadableStreamDefaultReader state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if super::readable_stream::record(scope, stream).is_none() {
        return Err("The supplied value is not a ReadableStream".to_owned());
    }
    if super::readable_stream::record(scope, stream).is_some_and(|record| record.locked) {
        return Err("The stream is already locked".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let reader = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, reader, prototype.into()) != Some(true) {
        return Err("cannot create ReadableStreamDefaultReader".to_owned());
    }
    super::readable_stream::set_locked(scope, stream, true);
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<ReadableStreamDefaultReaderStore>()
        .ok_or_else(|| "ReadableStreamDefaultReader state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "ReadableStreamDefaultReader requires a stream");
        return;
    }
    let Ok(stream) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReadableStreamDefaultReader': parameter 1 is not of type 'ReadableStream'.",
        );
        return;
    };
    let Some(stream_record) = super::readable_stream::record(scope, stream) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ReadableStreamDefaultReader': parameter 1 is not of type 'ReadableStream'.",
        );
        return;
    };
    if stream_record.locked {
        crate::webidl::throw_type_error(scope, "The stream is unavailable");
        return;
    }
    super::readable_stream::set_locked(scope, stream, true);
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<ReadableStreamDefaultReaderStore>()
        .expect("ReadableStreamDefaultReader state")
        .streams
        .insert(arguments.this().get_identity_hash().get(), Some(stream));
    result.set(arguments.this().into());
}

fn stream(
    scope: &v8::PinScope<'_, '_>,
    reader: v8::Local<'_, v8::Object>,
) -> Option<Option<v8::Global<v8::Object>>> {
    scope
        .get_slot::<ReadableStreamDefaultReaderStore>()?
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
                "ReadableStreamDefaultReader",
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
    let stream = v8::Local::new(scope, &stream);
    match super::readable_stream::read(scope, stream) {
        Ok(promise) => result.set(promise.into()),
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
        .get_slot_mut::<ReadableStreamDefaultReaderStore>()
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
            "Failed to read the 'closed' property from 'ReadableStreamDefaultReader': Illegal invocation",
        ) {
            result.set(promise.into());
        }
        return;
    };
    let value: v8::Local<v8::Value> = match current.as_ref() {
        Some(stream) => {
            let stream = v8::Local::new(scope, stream);
            if super::readable_stream::record(scope, stream).is_some_and(|record| {
                record.state == super::readable_stream::ReadableState::Errored
            }) {
                super::readable_stream::record(scope, stream)
                    .and_then(|record| record.stored_error)
                    .map(|value| v8::Local::new(scope, &value))
                    .unwrap_or_else(|| v8::undefined(scope).into())
            } else {
                v8::undefined(scope).into()
            }
        }
        None => v8::undefined(scope).into(),
    };
    let promise = if let Some(stream) = current.as_ref() {
        let stream = v8::Local::new(scope, stream);
        if super::readable_stream::record(scope, stream)
            .is_some_and(|record| record.state == super::readable_stream::ReadableState::Errored)
        {
            super::writable_stream::rejected_promise(scope, value)
        } else {
            super::writable_stream::resolved_promise(scope, value)
        }
    } else {
        super::writable_stream::rejected_promise(scope, value)
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
                "ReadableStreamDefaultReader",
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
    let undefined = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ReadableStreamDefaultReaderStore>() {
        store.constructor.remove(realm_id);
    }
}
