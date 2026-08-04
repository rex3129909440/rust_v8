use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TaskPriorityChangeEventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TaskPriorityChangeEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TaskPriorityChangeEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TaskPriorityChangeEventStore>()
        .and_then(|s| s.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TaskPriorityChangeEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::task_priority_change_event_previous_priority_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TaskPriorityChangeEventStore>()
        .ok_or_else(|| "TaskPriorityChangeEvent state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(scope, "TaskPriorityChangeEvent requires 2 arguments");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, a.get(0));
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let previous = init
        .and_then(|o| {
            v8::String::new(scope, "previousPriority").and_then(|k| o.get(scope, k.into()))
        })
        .map(|v| crate::webidl::value_to_string(scope, v))
        .unwrap_or_default();
    let bubbles = init.is_some_and(|o| super::event::boolean_property(scope, o, "bubbles"));
    let cancelable = init.is_some_and(|o| super::event::boolean_property(scope, o, "cancelable"));
    let composed = init.is_some_and(|o| super::event::boolean_property(scope, o, "composed"));
    super::event::attach(scope, a.this(), event_type, bubbles, cancelable, composed);
    scope
        .get_slot_mut::<TaskPriorityChangeEventStore>()
        .expect("TaskPriorityChangeEvent state")
        .records
        .insert(a.this().get_identity_hash().get(), previous);
    r.set(a.this().into())
}
pub(crate) fn get_previous_priority(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = scope
        .get_slot::<TaskPriorityChangeEventStore>()
        .and_then(|s| s.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, &v) {
        r.set(s.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TaskPriorityChangeEventStore>() {
        store.constructors.remove(&realm_id);
    }
}
