use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TaskSignalStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TaskSignalRecord>,
}
#[derive(Clone)]
struct TaskSignalRecord {
    priority: String,
    on_priority_change: Option<v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TaskSignalStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TaskSignal", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TaskSignalStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TaskSignal",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "priority", get_priority)?;
    crate::webidl::define_accessor(
        scope,
        p,
        "onprioritychange",
        get_on_priority_change,
        set_on_priority_change,
    )?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_method(scope, c.into(), "any", 1, static_any)?;
    let parent = super::abort_signal::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TaskSignalStore>()
        .ok_or_else(|| "TaskSignal state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    priority: String,
    reason: Option<v8::Local<'_, v8::Value>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create TaskSignal".to_owned());
    }
    super::abort_signal::attach(scope, o, reason);
    scope
        .get_slot_mut::<TaskSignalStore>()
        .ok_or_else(|| "TaskSignal state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            TaskSignalRecord {
                priority,
                on_priority_change: None,
            },
        );
    Ok(o)
}
pub(crate) fn set_priority(
    scope: &mut v8::PinScope<'_, '_>,
    signal: v8::Local<'_, v8::Object>,
    priority: String,
) -> bool {
    let handler = {
        let Some(v) = scope
            .get_slot_mut::<TaskSignalStore>()
            .and_then(|s| s.records.get_mut(&signal.get_identity_hash().get()))
        else {
            return false;
        };
        if v.priority == priority {
            return true;
        }
        v.priority = priority;
        v.on_priority_change.clone()
    };
    if let Some(handler) = handler
        && let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let event = super::event_target::create_event(scope, "prioritychange");
        let _ = function.call(scope, signal.into(), &[event.into()]);
    }
    true
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TaskSignal': Illegal constructor",
    );
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<TaskSignalRecord> {
    scope
        .get_slot::<TaskSignalStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}

pub(crate) fn priority(
    scope: &v8::PinScope<'_, '_>,
    signal: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, signal).map(|record| record.priority)
}
fn get_priority(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, &v.priority) {
        r.set(s.into())
    }
}
fn get_on_priority_change(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(h) = v.on_priority_change {
        r.set(v8::Local::new(scope, &h))
    } else {
        r.set(v8::null(scope).into())
    }
}
fn set_on_priority_change(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0);
    let handler = if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    };
    if let Some(v) = scope
        .get_slot_mut::<TaskSignalStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_priority_change = handler
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn static_any(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let sequence = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    let mut priority = "user-visible".to_owned();
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
            if let Some(task) = record(scope, signal) {
                priority = task.priority;
            }
            let Some(abort) = super::abort_signal::record(scope, signal) else {
                crate::webidl::throw_type_error(scope, "Sequence contains a non-AbortSignal");
                return;
            };
            if abort.aborted {
                reason = abort.reason.map(|v| v8::Local::new(scope, &v));
                break;
            }
        }
    }
    match create(scope, priority, reason) {
        Ok(v) => r.set(v.into()),
        Err(m) => crate::webidl::throw_type_error(scope, &m),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TaskSignalStore>() {
        store.constructor.remove(realm_id);
    }
}
