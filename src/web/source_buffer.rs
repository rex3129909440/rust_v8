use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SourceBufferStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
struct Record {
    mode: String,
    updating: bool,
    data: Vec<u8>,
    timestamp_offset: f64,
    append_window_start: f64,
    append_window_end: f64,
    mime_type: String,
    handlers: HashMap<String, v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SourceBufferStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SourceBuffer", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<SourceBufferStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "SourceBuffer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_accessor(scope, p, "mode", get_mode, set_mode)?;
    crate::webidl::define_readonly_accessor(scope, p, "updating", get_updating)?;
    crate::webidl::define_readonly_accessor(scope, p, "buffered", get_buffered)?;
    crate::webidl::define_accessor(
        scope,
        p,
        "timestampOffset",
        get_timestamp_offset,
        set_timestamp_offset,
    )?;
    crate::webidl::define_accessor(
        scope,
        p,
        "appendWindowStart",
        get_append_window_start,
        set_append_window_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        p,
        "appendWindowEnd",
        get_append_window_end,
        set_append_window_end,
    )?;
    crate::webidl::define_accessor(
        scope,
        p,
        "onupdatestart",
        get_onupdatestart,
        set_onupdatestart,
    )?;
    crate::webidl::define_accessor(scope, p, "onupdate", get_onupdate, set_onupdate)?;
    crate::webidl::define_accessor(scope, p, "onupdateend", get_onupdateend, set_onupdateend)?;
    crate::webidl::define_accessor(scope, p, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_accessor(scope, p, "onabort", get_onabort, set_onabort)?;
    crate::webidl::define_method(scope, p, "abort", 0, abort)?;
    crate::webidl::define_method(scope, p, "appendBuffer", 1, append_buffer)?;
    crate::webidl::define_method(scope, p, "changeType", 1, change_type)?;
    crate::webidl::define_method(scope, p, "remove", 2, remove)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SourceBufferStore>()
        .ok_or_else(|| "SourceBuffer state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    mime_type: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create SourceBuffer".to_owned());
    }
    super::event_target::attach(scope, o);
    scope
        .get_slot_mut::<SourceBufferStore>()
        .ok_or_else(|| "SourceBuffer state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            Record {
                mode: "segments".to_owned(),
                updating: false,
                data: Vec::new(),
                timestamp_offset: 0.0,
                append_window_start: 0.0,
                append_window_end: f64::INFINITY,
                mime_type,
                handlers: HashMap::new(),
            },
        );
    Ok(o)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SourceBuffer': Illegal constructor",
    );
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<SourceBufferStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(v) = scope
        .get_slot_mut::<SourceBufferStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn string_get(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> &str,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, select(&v)) {
        r.set(s.into())
    }
}
fn get_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_get(s, a, r, |v| &v.mode)
}
fn set_mode(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if !matches!(v.as_str(), "segments" | "sequence") {
        crate::webidl::throw_type_error(scope, "Invalid SourceBuffer mode");
        return;
    }
    update(scope, a.this(), |r| r.mode = v)
}
fn get_updating(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.updating).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_buffered(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let duration = v.data.len() as f64 / 1000.0;
    match super::time_ranges::create(
        scope,
        if v.data.is_empty() {
            Vec::new()
        } else {
            vec![(0.0, duration)]
        },
    ) {
        Ok(v) => r.set(v.into()),
        Err(m) => crate::webidl::throw_type_error(scope, &m),
    }
}
fn number_get(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> f64,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, select(&v)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_timestamp_offset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |v| v.timestamp_offset)
}
fn get_append_window_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |v| v.append_window_start)
}
fn get_append_window_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |v| v.append_window_end)
}
fn set_timestamp_offset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(s).unwrap_or(0.0);
    update(s, a.this(), |r| r.timestamp_offset = v)
}
fn set_append_window_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(s).unwrap_or(0.0);
    if v < 0.0 {
        crate::webidl::throw_type_error(s, "appendWindowStart cannot be negative");
        return;
    }
    update(s, a.this(), |r| r.append_window_start = v)
}
fn set_append_window_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).number_value(s).unwrap_or(f64::INFINITY);
    update(s, a.this(), |r| r.append_window_end = v)
}
fn handler_get(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    name: &str,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(h) = v.handlers.get(name) {
        r.set(v8::Local::new(scope, h))
    } else {
        r.set(v8::null(scope).into())
    }
}
fn handler_set(scope: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, name: &str) {
    let value = a.get(0);
    let h = if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    };
    update(scope, a.this(), |r| {
        if let Some(h) = h {
            r.handlers.insert(name.to_owned(), h);
        } else {
            r.handlers.remove(name);
        }
    })
}
fn get_onupdatestart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, "updatestart")
}
fn set_onupdatestart(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, "updatestart")
}
fn get_onupdate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, "update")
}
fn set_onupdate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, "update")
}
fn get_onupdateend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, "updateend")
}
fn set_onupdateend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, "updateend")
}
fn get_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, "error")
}
fn set_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, "error")
}
fn get_onabort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, "abort")
}
fn set_onabort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, "abort")
}
fn dispatch(scope: &mut v8::PinScope<'_, '_>, target: v8::Local<'_, v8::Object>, name: &str) {
    let event = super::event_target::create_event(scope, name);
    super::event_target::dispatch(scope, target, event);
}
fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    update(scope, a.this(), |v| v.updating = false);
    dispatch(scope, a.this(), "abort");
    dispatch(scope, a.this(), "updateend");
}
fn append_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let bytes = match super::text_decoder::bytes_from_value(scope, a.get(0)) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    update(scope, a.this(), |v| {
        v.updating = true;
        v.data.extend(bytes);
    });
    dispatch(scope, a.this(), "updatestart");
    update(scope, a.this(), |v| v.updating = false);
    dispatch(scope, a.this(), "update");
    dispatch(scope, a.this(), "updateend");
}
fn change_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if value.is_empty() {
        crate::webidl::throw_type_error(scope, "MIME type cannot be empty");
        return;
    }
    update(scope, a.this(), |v| v.mime_type = value)
}
fn remove(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let start = a.get(0).number_value(scope).unwrap_or(0.0);
    let end = a.get(1).number_value(scope).unwrap_or(0.0);
    if start < 0.0 || end <= start {
        crate::webidl::throw_type_error(scope, "Invalid removal range");
        return;
    }
    update(scope, a.this(), |v| {
        let s = (start * 1000.0) as usize;
        let e = ((end * 1000.0) as usize).min(v.data.len());
        if s < e {
            v.data.drain(s..e);
        }
    });
    dispatch(scope, a.this(), "updateend");
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SourceBufferStore>() {
        store.constructor.remove(realm_id);
    }
}
