use std::collections::HashMap;
#[derive(Clone)]
struct SessionRecord {
    id: String,
    expiration: f64,
    closed: v8::Global<v8::Promise>,
    close_resolver: v8::Global<v8::PromiseResolver>,
    statuses: v8::Global<v8::Object>,
    onstatus: Option<v8::Global<v8::Value>>,
    onmessage: Option<v8::Global<v8::Value>>,
    active: bool,
}
#[derive(Default)]
pub(crate) struct MediaKeySessionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SessionRecord>,
    next_id: u64,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaKeySessionStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaKeySession", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MediaKeySessionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaKeySession",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sessionId", get_session_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "expiration", get_expiration)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "closed", get_closed)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "keyStatuses", get_key_statuses)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onkeystatuseschange",
        get_onstatus,
        set_onstatus,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onmessage", get_onmessage, set_onmessage)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "generateRequest", 2, generate_request)?;
    crate::webidl::define_method(scope, prototype, "load", 1, load)?;
    crate::webidl::define_method(scope, prototype, "remove", 0, remove)?;
    crate::webidl::define_method(scope, prototype, "update", 1, update)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MediaKeySessionStore>()
        .ok_or_else(|| "MediaKeySession state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(s)?;
    let prototype = crate::webidl::prototype(s, constructor)?;
    let object = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaKeySession".to_owned());
    }
    super::event_target::attach(s, object);
    let statuses = super::media_key_status_map::create(s)?;
    let resolver =
        v8::PromiseResolver::new(s).ok_or_else(|| "cannot create closed promise".to_owned())?;
    let closed = resolver.get_promise(s);
    let next = {
        let store = s
            .get_slot_mut::<MediaKeySessionStore>()
            .ok_or_else(|| "MediaKeySession state was not prepared".to_owned())?;
        store.next_id += 1;
        store.next_id
    };
    let record = SessionRecord {
        id: format!("edge-session-{next}"),
        expiration: f64::NAN,
        closed: v8::Global::new(s, closed),
        close_resolver: v8::Global::new(s, resolver),
        statuses: v8::Global::new(s, statuses),
        onstatus: None,
        onmessage: None,
        active: true,
    };
    s.get_slot_mut::<MediaKeySessionStore>()
        .unwrap()
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<SessionRecord> {
    s.get_slot::<MediaKeySessionStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_session_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(value) = v8::String::new(s, &v.id)
    {
        r.set(value.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_expiration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Number::new(s, v.expiration).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_closed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.closed).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_key_statuses(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.statuses).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_onstatus(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.onstatus),
        r,
    )
}
fn get_onmessage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.onmessage),
        r,
    )
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    v: v8::Local<'_, v8::Value>,
    status: bool,
) {
    let handler = super::window_event_handler_support::handler_value(s, v);
    if let Some(record) = s
        .get_slot_mut::<MediaKeySessionStore>()
        .and_then(|store| store.records.get_mut(&o.get_identity_hash().get()))
    {
        if status {
            record.onstatus = handler
        } else {
            record.onmessage = handler
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_onstatus(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a.this(), a.get(0), true)
}
fn set_onmessage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a.this(), a.get(0), false)
}
fn resolve(
    s: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(s, value) {
        r.set(promise.into())
    }
}
fn active(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>) -> bool {
    if record(s, a.this()).is_some_and(|v| v.active) {
        true
    } else {
        crate::webidl::throw_type_error(s, "MediaKeySession is closed");
        false
    }
}
fn generate_request(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if active(s, a) {
        resolve(s, v8::undefined(s).into(), r)
    }
}
fn load(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if active(s, a) {
        resolve(s, v8::Boolean::new(s, false).into(), r)
    }
}
fn remove(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if active(s, a) {
        resolve(s, v8::undefined(s).into(), r)
    }
}
fn update(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if active(s, a) {
        resolve(s, v8::undefined(s).into(), r)
    }
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let resolver = {
        let Some(record) = s
            .get_slot_mut::<MediaKeySessionStore>()
            .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
        else {
            crate::webidl::throw_type_error(s, "Illegal invocation");
            return;
        };
        record.active = false;
        record.close_resolver.clone()
    };
    let resolver = v8::Local::new(s, &resolver);
    let _ = resolver.resolve(s, v8::undefined(s).into());
    resolve(s, v8::undefined(s).into(), r)
}
