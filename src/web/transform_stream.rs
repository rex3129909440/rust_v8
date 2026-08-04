use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TransformStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TransformRecord>,
    sink_to_transform: HashMap<i32, i32>,
}

#[derive(Clone)]
struct TransformRecord {
    transformer: Option<v8::Global<v8::Object>>,
    transform_callback: Option<v8::Global<v8::Function>>,
    flush_callback: Option<v8::Global<v8::Function>>,
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
    controller: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TransformStreamStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TransformStream", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TransformStreamStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TransformStream",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readable", get_readable)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "writable", get_writable)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TransformStreamStore>()
        .ok_or_else(|| "TransformStream state was not prepared".to_owned())?
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
            "Failed to construct 'TransformStream': use the new operator",
        );
        return;
    }
    let object = arguments.this();
    let transformer = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let transform_callback =
        transformer.and_then(|value| function_property(scope, value, "transform"));
    let flush_callback = transformer.and_then(|value| function_property(scope, value, "flush"));
    let start_callback = transformer.and_then(|value| function_property(scope, value, "start"));
    let readable = match super::readable_stream::create_empty(scope) {
        Ok(readable) => readable,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let controller = match super::transform_stream_default_controller::create(scope, readable) {
        Ok(controller) => controller,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let sink = match create_sink(scope) {
        Ok(sink) => sink,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let writable_constructor = match super::writable_stream::ensure_constructor(scope) {
        Ok(constructor) => constructor,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let Some(writable) = writable_constructor.new_instance(scope, &[sink.into()]) else {
        crate::webidl::throw_type_error(scope, "cannot create TransformStream writable side");
        return;
    };
    let transform_identity = object.get_identity_hash().get();
    let record = TransformRecord {
        transformer: transformer.map(|value| v8::Global::new(scope, value)),
        transform_callback,
        flush_callback,
        readable: v8::Global::new(scope, readable),
        writable: v8::Global::new(scope, writable),
        controller: v8::Global::new(scope, controller),
    };
    let store = scope
        .get_slot_mut::<TransformStreamStore>()
        .expect("TransformStream state");
    store
        .sink_to_transform
        .insert(sink.get_identity_hash().get(), transform_identity);
    store.records.insert(transform_identity, record);
    if let (Some(transformer), Some(start)) = (transformer, start_callback) {
        let callback = v8::Local::new(scope, &start);
        let _ = callback.call(scope, transformer.into(), &[controller.into()]);
    }
    result.set(object.into());
}

fn create_sink<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Object>, String> {
    let sink = v8::Object::new(scope);
    define_sink_method(scope, sink, "write", 1, sink_write)?;
    define_sink_method(scope, sink, "close", 0, sink_close)?;
    define_sink_method(scope, sink, "abort", 1, sink_abort)?;
    Ok(sink)
}

fn define_sink_method(
    scope: &mut v8::PinScope<'_, '_>,
    sink: v8::Local<'_, v8::Object>,
    name: &str,
    length: i32,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        name,
        length,
        v8::ConstructorBehavior::Throw,
        callback,
    )?;
    let key = crate::webidl::string(scope, name)?;
    if sink.create_data_property(scope, key.into(), function.into()) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot create transform sink method {name}"))
    }
}

fn record_for_sink(
    scope: &v8::PinScope<'_, '_>,
    sink: v8::Local<'_, v8::Object>,
) -> Option<TransformRecord> {
    let store = scope.get_slot::<TransformStreamStore>()?;
    let identity = store
        .sink_to_transform
        .get(&sink.get_identity_hash().get())?;
    store.records.get(identity).cloned()
}

fn sink_write(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid transform sink");
        return;
    };
    let controller = v8::Local::new(scope, &record.controller);
    let returned = if let Some(callback) = record.transform_callback {
        let callback = v8::Local::new(scope, &callback);
        let receiver = record
            .transformer
            .as_ref()
            .map(|value| v8::Local::new(scope, value).into())
            .unwrap_or_else(|| v8::undefined(scope).into());
        callback
            .call(scope, receiver, &[arguments.get(0), controller.into()])
            .unwrap_or_else(|| v8::undefined(scope).into())
    } else {
        let readable = v8::Local::new(scope, &record.readable);
        super::readable_stream::enqueue(scope, readable, arguments.get(0));
        v8::undefined(scope).into()
    };
    result.set(returned);
}

fn sink_close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid transform sink");
        return;
    };
    let returned = if let Some(callback) = record.flush_callback {
        let callback = v8::Local::new(scope, &callback);
        let receiver = record
            .transformer
            .as_ref()
            .map(|value| v8::Local::new(scope, value).into())
            .unwrap_or_else(|| v8::undefined(scope).into());
        let controller = v8::Local::new(scope, &record.controller);
        callback
            .call(scope, receiver, &[controller.into()])
            .unwrap_or_else(|| v8::undefined(scope).into())
    } else {
        v8::undefined(scope).into()
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::close(scope, readable);
    result.set(returned);
}

fn sink_abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid transform sink");
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::error(scope, readable, arguments.get(0));
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TransformRecord> {
    scope
        .get_slot::<TransformStreamStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_readable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.readable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.writable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TransformStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
