use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextDecoderStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, StreamRecord>,
    sinks: HashMap<i32, i32>,
}
#[derive(Clone)]
struct StreamRecord {
    encoding: String,
    fatal: bool,
    ignore_bom: bool,
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextDecoderStreamStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextDecoderStream", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TextDecoderStreamStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TextDecoderStream",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "encoding", get_encoding)?;
    crate::webidl::define_readonly_accessor(scope, p, "fatal", get_fatal)?;
    crate::webidl::define_readonly_accessor(scope, p, "ignoreBOM", get_ignore_bom)?;
    crate::webidl::define_readonly_accessor(scope, p, "readable", get_readable)?;
    crate::webidl::define_readonly_accessor(scope, p, "writable", get_writable)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TextDecoderStreamStore>()
        .ok_or_else(|| "TextDecoderStream state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'TextDecoderStream': use new");
        return;
    }
    let label = if arguments.length() == 0 || arguments.get(0).is_undefined() {
        "utf-8".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let encoding = match label.trim().to_ascii_lowercase().as_str() {
        "utf-8" | "utf8" => "utf-8",
        "utf-16" | "utf-16le" => "utf-16le",
        "utf-16be" => "utf-16be",
        "windows-1252" | "latin1" | "iso-8859-1" | "ascii" => "windows-1252",
        _ => {
            crate::webidl::throw_type_error(scope, "Unsupported encoding");
            return;
        }
    };
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let fatal = options.is_some_and(|v| super::event::boolean_property(scope, v, "fatal"));
    let ignore_bom = options.is_some_and(|v| super::event::boolean_property(scope, v, "ignoreBOM"));
    let readable = match super::readable_stream::create_empty(scope) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    let sink = match create_sink(scope) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    let wc = match super::writable_stream::ensure_constructor(scope) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    let Some(writable) = wc.new_instance(scope, &[sink.into()]) else {
        crate::webidl::throw_type_error(scope, "cannot create decoder writable side");
        return;
    };
    let object = arguments.this();
    let identity = object.get_identity_hash().get();
    let record = StreamRecord {
        encoding: encoding.to_owned(),
        fatal,
        ignore_bom,
        readable: v8::Global::new(scope, readable),
        writable: v8::Global::new(scope, writable),
    };
    let store = scope
        .get_slot_mut::<TextDecoderStreamStore>()
        .expect("TextDecoderStream state");
    store.sinks.insert(sink.get_identity_hash().get(), identity);
    store.records.insert(identity, record);
    result.set(object.into());
}
fn create_sink<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Object>, String> {
    let o = v8::Object::new(scope);
    define_method(scope, o, "write", 1, sink_write)?;
    define_method(scope, o, "close", 0, sink_close)?;
    define_method(scope, o, "abort", 1, sink_abort)?;
    Ok(o)
}
fn define_method(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
    l: i32,
    cb: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let f = crate::webidl::create_function(scope, n, l, v8::ConstructorBehavior::Throw, cb)?;
    let k = crate::webidl::string(scope, n)?;
    if o.create_data_property(scope, k.into(), f.into()) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define decoder sink {n}"))
    }
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<StreamRecord> {
    scope
        .get_slot::<TextDecoderStreamStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn record_for_sink(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<StreamRecord> {
    let s = scope.get_slot::<TextDecoderStreamStore>()?;
    let id = s.sinks.get(&o.get_identity_hash().get())?;
    s.records.get(id).cloned()
}
fn sink_write(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid decoder stream sink");
        return;
    };
    let bytes = match super::text_decoder::bytes_from_value(scope, arguments.get(0)) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    let text = match super::text_decoder::decode_bytes(
        &record.encoding,
        &bytes,
        record.fatal,
        record.ignore_bom,
    ) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    let Some(text) = v8::String::new(scope, &text) else {
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::enqueue(scope, readable, text.into());
}
fn sink_close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for_sink(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Invalid decoder stream sink");
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
        crate::webidl::throw_type_error(scope, "Invalid decoder stream sink");
        return;
    };
    let readable = v8::Local::new(scope, &record.readable);
    super::readable_stream::error(scope, readable, arguments.get(0));
}
fn get_encoding(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, &v.encoding) {
        r.set(s.into())
    }
}
fn get_fatal(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.fatal).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_ignore_bom(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.ignore_bom).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_readable(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.readable).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_writable(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.writable).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TextDecoderStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
