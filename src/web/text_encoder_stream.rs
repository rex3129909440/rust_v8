use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextEncoderStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, StreamRecord>,
    sinks: HashMap<i32, i32>,
}

#[derive(Clone)]
struct StreamRecord {
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextEncoderStreamStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextEncoderStream", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TextEncoderStreamStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TextEncoderStream",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "encoding", get_encoding)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readable", get_readable)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "writable", get_writable)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TextEncoderStreamStore>()
        .ok_or_else(|| "TextEncoderStream state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "Failed to construct 'TextEncoderStream': use new");
        return;
    }
    let readable = match super::readable_stream::create_empty(scope) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let sink = match create_sink(scope) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let writable_constructor = match super::writable_stream::ensure_constructor(scope) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let Some(writable) = writable_constructor.new_instance(scope, &[sink.into()]) else {
        crate::webidl::throw_type_error(scope, "cannot create encoder writable side");
        return;
    };
    let object = arguments.this();
    let identity = object.get_identity_hash().get();
    let record = StreamRecord {
        readable: v8::Global::new(scope, readable),
        writable: v8::Global::new(scope, writable),
    };
    let store = scope
        .get_slot_mut::<TextEncoderStreamStore>()
        .expect("TextEncoderStream state");
    store.sinks.insert(sink.get_identity_hash().get(), identity);
    store.records.insert(identity, record);
    result.set(object.into());
}

fn create_sink<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Object>, String> {
    let sink = v8::Object::new(scope);
    define_method(scope, sink, "write", 1, sink_write)?;
    define_method(scope, sink, "close", 0, sink_close)?;
    define_method(scope, sink, "abort", 1, sink_abort)?;
    Ok(sink)
}

fn define_method(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
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
    if object.create_data_property(scope, key.into(), function.into()) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define encoder sink {name}"))
    }
}

fn record_for_sink(
    scope: &v8::PinScope<'_, '_>,
    sink: v8::Local<'_, v8::Object>,
) -> Option<StreamRecord> {
    let store = scope.get_slot::<TextEncoderStreamStore>()?;
    let identity = store.sinks.get(&sink.get_identity_hash().get())?;
    store.records.get(identity).cloned()
}

fn sink_write(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid encoder stream sink");
        return;
    };
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(bytes) = super::text_encoder::uint8_array(scope, text.into_bytes()) else {
        crate::webidl::throw_type_error(scope, "cannot encode chunk");
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::enqueue(scope, readable, bytes.into());
}

fn sink_close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid encoder stream sink");
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::close(scope, readable);
}

fn sink_abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid encoder stream sink");
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::error(scope, readable, arguments.get(0));
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<StreamRecord> {
    scope
        .get_slot::<TextEncoderStreamStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_encoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = v8::String::new(scope, "utf-8") {
        result.set(value.into())
    }
}
fn get_readable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.readable).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.writable).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TextEncoderStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
