use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WritableStreamDefaultControllerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ControllerRecord>,
}

#[derive(Clone)]
struct ControllerRecord {
    stream: v8::Global<v8::Object>,
    signal: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WritableStreamDefaultControllerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WritableStreamDefaultController", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<WritableStreamDefaultControllerStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WritableStreamDefaultController",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "signal", get_signal)?;
    crate::webidl::define_method(scope, prototype, "error", 0, error)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WritableStreamDefaultControllerStore>()
        .ok_or_else(|| "WritableStreamDefaultController state was not prepared".to_owned())?
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
        return Err("cannot create WritableStreamDefaultController".to_owned());
    }
    let signal = v8::Object::new(scope);
    define_data(
        scope,
        signal,
        "aborted",
        v8::Boolean::new(scope, false).into(),
    );
    define_data(scope, signal, "reason", v8::undefined(scope).into());
    super::event_target::attach(scope, signal);
    let record = ControllerRecord {
        stream: v8::Global::new(scope, stream),
        signal: v8::Global::new(scope, signal),
    };
    scope
        .get_slot_mut::<WritableStreamDefaultControllerStore>()
        .ok_or_else(|| "WritableStreamDefaultController state was not prepared".to_owned())?
        .records
        .insert(controller.get_identity_hash().get(), record);
    Ok(controller)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'WritableStreamDefaultController': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ControllerRecord> {
    scope
        .get_slot::<WritableStreamDefaultControllerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_signal(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.signal).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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
    let stream = v8::Local::new(scope, &record.stream);
    super::writable_stream::error_stream(scope, stream, arguments.get(0));
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
    if let Some(store) = scope.get_slot_mut::<WritableStreamDefaultControllerStore>() {
        store.constructor.remove(realm_id);
    }
}
