use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AbortSignalStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, AbortSignalRecord>,
}

#[derive(Clone)]
pub(crate) struct AbortSignalRecord {
    pub aborted: bool,
    pub reason: Option<v8::Global<v8::Value>>,
    pub onabort: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AbortSignalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AbortSignal", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AbortSignalStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AbortSignal",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "aborted", get_aborted)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reason", get_reason)?;
    crate::webidl::define_accessor(scope, prototype, "onabort", get_onabort, set_onabort)?;
    crate::webidl::define_method(scope, prototype, "throwIfAborted", 0, throw_if_aborted)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "abort", 0, static_abort)?;
    crate::webidl::define_method(scope, constructor.into(), "any", 1, static_any)?;
    crate::webidl::define_method(scope, constructor.into(), "timeout", 1, static_timeout)?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AbortSignalStore>()
        .ok_or_else(|| "AbortSignal state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    reason: Option<v8::Local<'_, v8::Value>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let signal = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, signal, prototype.into()) != Some(true) {
        return Err("cannot create AbortSignal".to_owned());
    }
    attach(scope, signal, reason);
    Ok(signal)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    signal: v8::Local<'_, v8::Object>,
    reason: Option<v8::Local<'_, v8::Value>>,
) {
    super::event_target::attach(scope, signal);
    let reason = reason.map(|value| v8::Global::new(scope, value));
    if let Some(store) = scope.get_slot_mut::<AbortSignalStore>() {
        store.records.insert(
            signal.get_identity_hash().get(),
            AbortSignalRecord {
                aborted: reason.is_some(),
                reason,
                onabort: None,
            },
        );
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    signal: v8::Local<'_, v8::Object>,
) -> Option<AbortSignalRecord> {
    scope
        .get_slot::<AbortSignalStore>()?
        .records
        .get(&signal.get_identity_hash().get())
        .cloned()
}

pub(crate) fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    signal: v8::Local<'_, v8::Object>,
    reason: v8::Local<'_, v8::Value>,
) -> bool {
    let reason_global = v8::Global::new(scope, reason);
    {
        let Some(record) = scope
            .get_slot_mut::<AbortSignalStore>()
            .and_then(|store| store.records.get_mut(&signal.get_identity_hash().get()))
        else {
            return false;
        };
        if record.aborted {
            return true;
        }
        record.aborted = true;
        record.reason = Some(reason_global);
    }
    super::event_target::remove_signal_listeners(scope, signal);
    let event = super::event_target::create_event(scope, "abort");
    super::event_target::dispatch(scope, signal, event);
    true
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "abort" {
        return;
    }
    let handler = scope
        .get_slot::<AbortSignalStore>()
        .and_then(|store| store.records.get(&target.get_identity_hash().get()))
        .and_then(|record| record.onabort.clone());
    let Some(handler) = handler else {
        return;
    };
    let handler = v8::Local::new(scope, &handler);
    let Ok(function) = v8::Local::<v8::Function>::try_from(handler) else {
        return;
    };
    v8::tc_scope!(let try_catch, scope);
    let _ = function.call(try_catch, target.into(), &[event.into()]);
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'AbortSignal': Illegal constructor",
    );
}

fn get_aborted(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.aborted).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_reason(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(reason) = v.reason {
        r.set(v8::Local::new(scope, &reason))
    } else {
        r.set(v8::undefined(scope).into())
    }
}
fn get_onabort(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = v.onabort {
        r.set(v8::Local::new(scope, &handler))
    } else {
        r.set(v8::null(scope).into())
    }
}
fn set_onabort(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0);
    let handler = if value.is_object() {
        Some(v8::Global::new(scope, value))
    } else {
        None
    };
    let present = handler.is_some();
    if let Some(v) = scope
        .get_slot_mut::<AbortSignalStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.onabort = handler;
        super::event_target::set_attribute_handler(scope, a.this(), "abort", present);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn throw_if_aborted(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if v.aborted {
        let reason = v
            .reason
            .as_ref()
            .map(|v| v8::Local::new(scope, v))
            .unwrap_or_else(|| v8::undefined(scope).into());
        scope.throw_exception(reason);
    }
}
fn static_abort(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let reason = if a.length() == 0 {
        v8::String::new(scope, "This operation was aborted")
            .map(Into::into)
            .unwrap_or_else(|| v8::undefined(scope).into())
    } else {
        a.get(0)
    };
    match create(scope, Some(reason)) {
        Ok(v) => r.set(v.into()),
        Err(m) => crate::webidl::throw_type_error(scope, &m),
    }
}
fn static_any(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let sequence = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    let mut reason = None;
    if let Some(sequence) = sequence {
        let length = v8::String::new(scope, "length")
            .and_then(|k| sequence.get(scope, k.into()))
            .and_then(|v| v.uint32_value(scope))
            .unwrap_or(0);
        for i in 0..length {
            let Some(value) = sequence.get_index(scope, i) else {
                continue;
            };
            let Ok(signal) = v8::Local::<v8::Object>::try_from(value) else {
                crate::webidl::throw_type_error(scope, "Sequence contains a non-AbortSignal");
                return;
            };
            let Some(record) = record(scope, signal) else {
                crate::webidl::throw_type_error(scope, "Sequence contains a non-AbortSignal");
                return;
            };
            if record.aborted {
                reason = record.reason.map(|v| v8::Local::new(scope, &v));
                break;
            }
        }
    }
    match create(scope, reason) {
        Ok(v) => r.set(v.into()),
        Err(m) => crate::webidl::throw_type_error(scope, &m),
    }
}
fn static_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let ms = a.get(0).uint32_value(scope).unwrap_or(0);
    let message = format!("The operation timed out after {ms} ms");
    let reason = v8::String::new(scope, &message)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into());
    match create(scope, Some(reason)) {
        Ok(v) => r.set(v.into()),
        Err(m) => crate::webidl::throw_type_error(scope, &m),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<AbortSignalStore>() {
        store.constructors.remove(&realm_id);
    }
}
