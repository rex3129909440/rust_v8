use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SubscriberStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SubscriberRecord>,
}
#[derive(Clone)]
struct SubscriberRecord {
    active: bool,
    signal: v8::Global<v8::Object>,
    observer: v8::Global<v8::Object>,
    next: Option<v8::Global<v8::Function>>,
    error: Option<v8::Global<v8::Function>>,
    complete: Option<v8::Global<v8::Function>>,
    teardowns: Vec<v8::Global<v8::Function>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SubscriberStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Subscriber", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<SubscriberStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "Subscriber",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "active", get_active)?;
    crate::webidl::define_readonly_accessor(scope, p, "signal", get_signal)?;
    crate::webidl::define_method(scope, p, "addTeardown", 1, add_teardown)?;
    crate::webidl::define_method(scope, p, "complete", 0, complete)?;
    crate::webidl::define_method(scope, p, "error", 1, error)?;
    crate::webidl::define_method(scope, p, "next", 1, next)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SubscriberStore>()
        .ok_or_else(|| "Subscriber state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    observer: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create Subscriber".to_owned());
    }
    let signal = super::abort_signal::create(scope, None)?;
    let record = SubscriberRecord {
        active: true,
        signal: v8::Global::new(scope, signal),
        observer: v8::Global::new(scope, observer),
        next: function(scope, observer, "next"),
        error: function(scope, observer, "error"),
        complete: function(scope, observer, "complete"),
        teardowns: Vec::new(),
    };
    scope
        .get_slot_mut::<SubscriberStore>()
        .ok_or_else(|| "Subscriber state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(o)
}
fn function(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> Option<v8::Global<v8::Function>> {
    let k = v8::String::new(scope, n)?;
    let v = o.get(scope, k.into())?;
    v8::Local::<v8::Function>::try_from(v)
        .ok()
        .map(|f| v8::Global::new(scope, f))
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Subscriber': Illegal constructor",
    );
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<SubscriberRecord> {
    scope
        .get_slot::<SubscriberStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_active(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.active).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_signal(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.signal).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn add_teardown(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(f) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "addTeardown requires a function");
        return;
    };
    let f = v8::Global::new(scope, f);
    let Some(v) = scope
        .get_slot_mut::<SubscriberStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if v.active {
        v.teardowns.push(f)
    } else {
        let f = v8::Local::new(scope, &f);
        let _ = f.call(scope, v8::undefined(scope).into(), &[]);
    }
}
fn call_observer(
    scope: &mut v8::PinScope<'_, '_>,
    record: &SubscriberRecord,
    callback: Option<&v8::Global<v8::Function>>,
    args: &[v8::Local<v8::Value>],
) {
    if let Some(callback) = callback {
        let f = v8::Local::new(scope, callback);
        let receiver = v8::Local::new(scope, &record.observer);
        let _ = f.call(scope, receiver.into(), args);
    }
}
fn finish(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    callback: impl FnOnce(&SubscriberRecord) -> Option<&v8::Global<v8::Function>>,
    args: &[v8::Local<v8::Value>],
) {
    let Some(record) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.active {
        return;
    }
    if let Some(v) = scope
        .get_slot_mut::<SubscriberStore>()
        .and_then(|s| s.records.get_mut(&object.get_identity_hash().get()))
    {
        v.active = false;
    }
    let signal = v8::Local::new(scope, &record.signal);
    let _ = super::abort_signal::abort(scope, signal, v8::undefined(scope).into());
    for teardown in &record.teardowns {
        let f = v8::Local::new(scope, teardown);
        let _ = f.call(scope, v8::undefined(scope).into(), &[]);
    }
    call_observer(scope, &record, callback(&record), args);
}
fn complete(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    finish(scope, a.this(), |v| v.complete.as_ref(), &[])
}
fn error(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0);
    finish(scope, a.this(), |v| v.error.as_ref(), &[value])
}
fn next(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.active {
        call_observer(scope, &record, record.next.as_ref(), &[a.get(0)]);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SubscriberStore>() {
        store.constructor.remove(realm_id);
    }
}
