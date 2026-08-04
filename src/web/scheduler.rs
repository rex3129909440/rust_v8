use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct SchedulerStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SchedulerStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Scheduler", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SchedulerStore>()
        .and_then(|s| s.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "Scheduler",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_method(scope, p, "postTask", 1, post_task)?;
    crate::webidl::define_method(scope, p, "yield", 0, yield_task)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SchedulerStore>()
        .ok_or_else(|| "Scheduler state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create Scheduler".to_owned());
    }
    scope
        .get_slot_mut::<SchedulerStore>()
        .ok_or_else(|| "Scheduler state was not prepared".to_owned())?
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn valid(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<SchedulerStore>()
        .is_some_and(|s| s.instances.contains(&o.get_identity_hash().get()))
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Scheduler': Illegal constructor",
    );
}
fn post_task(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(scope, a.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "postTask requires a function");
        return;
    };
    let value = callback
        .call(scope, v8::undefined(scope).into(), &[])
        .unwrap_or_else(|| v8::undefined(scope).into());
    if let Ok(p) = super::writable_stream::resolved_promise(scope, value) {
        r.set(p.into())
    }
}
fn yield_task(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(scope, a.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = v8::undefined(scope);
    if let Ok(p) = super::writable_stream::resolved_promise(scope, value.into()) {
        r.set(p.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SchedulerStore>() {
        store.constructors.remove(&realm_id);
    }
}
