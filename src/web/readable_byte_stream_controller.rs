use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ReadableByteStreamControllerStore {
    constructor: crate::webidl::RealmConstructor,
    streams: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ReadableByteStreamControllerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ReadableByteStreamController", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ReadableByteStreamControllerStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ReadableByteStreamController",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "byobRequest", get_byob_request)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "desiredSize", get_desired_size)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "enqueue", 1, enqueue)?;
    crate::webidl::define_method(scope, prototype, "error", 0, error)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ReadableByteStreamControllerStore>()
        .ok_or_else(|| "ReadableByteStreamController state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let controller = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, controller, prototype.into()) != Some(true) {
        return Err("cannot create ReadableByteStreamController".to_owned());
    }
    let stream = v8::Global::new(scope, stream);
    scope
        .get_slot_mut::<ReadableByteStreamControllerStore>()
        .ok_or_else(|| "ReadableByteStreamController state was not prepared".to_owned())?
        .streams
        .insert(controller.get_identity_hash().get(), stream);
    Ok(controller)
}

fn stream(
    scope: &v8::PinScope<'_, '_>,
    controller: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<ReadableByteStreamControllerStore>()?
        .streams
        .get(&controller.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ReadableByteStreamController': Illegal constructor",
    );
}

fn get_byob_request(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = stream(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let stream = v8::Local::new(scope, &stream);
    if let Some(request) = super::readable_stream_byob_request::current_for_stream(scope, stream) {
        result.set(request.into())
    } else {
        result.set(v8::null(scope).into())
    }
}

fn get_desired_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(stream) = stream(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let stream = v8::Local::new(scope, &stream);
    let desired = super::readable_stream::record(scope, stream)
        .map(|record| -(record.queue.len() as f64))
        .unwrap_or(0.0);
    result.set(v8::Number::new(scope, desired).into());
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(stream) = stream(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let stream = v8::Local::new(scope, &stream);
    let _ = super::readable_stream_byob_request::close_stream(scope, stream);
    if !super::readable_stream::close(scope, stream) {
        crate::webidl::throw_type_error(scope, "The stream cannot be closed");
    }
}

fn enqueue(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(stream) = stream(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let stream = v8::Local::new(scope, &stream);
    if super::readable_stream_byob_request::fulfill_stream(scope, stream, arguments.get(0)) {
        return;
    }
    if !super::readable_stream::enqueue(scope, stream, arguments.get(0)) {
        crate::webidl::throw_type_error(scope, "The stream is not readable");
    }
}

fn error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(stream) = stream(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let stream = v8::Local::new(scope, &stream);
    let _ = super::readable_stream::error(scope, stream, arguments.get(0));
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ReadableByteStreamControllerStore>() {
        store.constructor.remove(realm_id);
    }
}
