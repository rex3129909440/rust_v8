use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TransformStreamDefaultControllerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ControllerRecord>,
}

#[derive(Clone)]
struct ControllerRecord {
    readable: v8::Global<v8::Object>,
    terminated: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TransformStreamDefaultControllerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "TransformStreamDefaultController",
        constructor.into(),
    )
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TransformStreamDefaultControllerStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TransformStreamDefaultController",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "desiredSize", get_desired_size)?;
    crate::webidl::define_method(scope, prototype, "enqueue", 0, enqueue)?;
    crate::webidl::define_method(scope, prototype, "error", 0, error)?;
    crate::webidl::define_method(scope, prototype, "terminate", 0, terminate)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TransformStreamDefaultControllerStore>()
        .ok_or_else(|| "TransformStreamDefaultController state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    readable: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let controller = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, controller, prototype.into()) != Some(true) {
        return Err("cannot create TransformStreamDefaultController".to_owned());
    }
    let readable = v8::Global::new(scope, readable);
    scope
        .get_slot_mut::<TransformStreamDefaultControllerStore>()
        .ok_or_else(|| "TransformStreamDefaultController state was not prepared".to_owned())?
        .records
        .insert(
            controller.get_identity_hash().get(),
            ControllerRecord {
                readable,
                terminated: false,
            },
        );
    Ok(controller)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TransformStreamDefaultController': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    controller: v8::Local<'_, v8::Object>,
) -> Option<ControllerRecord> {
    scope
        .get_slot::<TransformStreamDefaultControllerStore>()?
        .records
        .get(&controller.get_identity_hash().get())
        .cloned()
}

fn get_desired_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.terminated {
        result.set(v8::null(scope).into());
        return;
    }
    let readable = v8::Local::new(scope, &record.readable);
    let desired = super::readable_stream::record(scope, readable)
        .map(|stream| 1.0 - stream.queue.len() as f64)
        .unwrap_or(0.0);
    result.set(v8::Number::new(scope, desired).into());
}

fn enqueue(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.terminated {
        crate::webidl::throw_type_error(scope, "The transform stream has terminated");
        return;
    }
    let readable = v8::Local::new(scope, &record.readable);
    if !super::readable_stream::enqueue(scope, readable, arguments.get(0)) {
        crate::webidl::throw_type_error(scope, "The readable side is not available");
    }
}

fn error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::error(scope, readable, arguments.get(0));
    mark_terminated(scope, arguments.this());
}

fn terminate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::close(scope, readable);
    mark_terminated(scope, arguments.this());
}

fn mark_terminated(scope: &mut v8::PinScope<'_, '_>, controller: v8::Local<'_, v8::Object>) {
    if let Some(record) = scope
        .get_slot_mut::<TransformStreamDefaultControllerStore>()
        .and_then(|store| store.records.get_mut(&controller.get_identity_hash().get()))
    {
        record.terminated = true;
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TransformStreamDefaultControllerStore>() {
        store.constructor.remove(realm_id);
    }
}
