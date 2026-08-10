use std::collections::{HashMap, HashSet};

struct ScheduledTask {
    sequence: u64,
    due_ms: f64,
    callback: Option<v8::Global<v8::Function>>,
    context: v8::Global<v8::Context>,
    resolver: v8::Global<v8::PromiseResolver>,
}

#[derive(Default)]
pub(crate) struct SchedulerStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashSet<i32>,
    pending: Vec<ScheduledTask>,
    next_sequence: u64,
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
    let delay = task_delay(scope, a.get(1));
    schedule(scope, Some(callback), delay, &mut r);
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
    schedule(scope, None, 0.0, &mut r);
}

fn task_delay(scope: &mut v8::PinScope<'_, '_>, options: v8::Local<'_, v8::Value>) -> f64 {
    let Ok(options) = v8::Local::<v8::Object>::try_from(options) else {
        return 0.0;
    };
    let Some(key) = v8::String::new(scope, "delay") else {
        return 0.0;
    };
    let Some(value) = options.get(scope, key.into()) else {
        return 0.0;
    };
    value
        .number_value(scope)
        .filter(|value| value.is_finite())
        .unwrap_or(0.0)
        .trunc()
        .max(0.0)
}

fn schedule(
    scope: &mut v8::PinScope<'_, '_>,
    callback: Option<v8::Local<'_, v8::Function>>,
    delay_ms: f64,
    result: &mut v8::ReturnValue<'_>,
) {
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let promise = resolver.get_promise(scope);
    let callback = callback.map(|callback| v8::Global::new(scope, callback));
    let context = v8::Global::new(scope, scope.get_current_context());
    let due_ms = crate::determinism::monotonic_snapshot_milliseconds(scope) + delay_ms;
    let resolver = v8::Global::new(scope, resolver);
    if let Some(store) = scope.get_slot_mut::<SchedulerStore>() {
        let sequence = store.next_sequence;
        store.next_sequence = sequence.saturating_add(1);
        store.pending.push(ScheduledTask {
            sequence,
            due_ms,
            callback,
            context,
            resolver,
        });
    }
    result.set(promise.into());
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<SchedulerStore>().and_then(|store| {
        store
            .pending
            .iter()
            .map(|task| task.due_ms)
            .min_by(f64::total_cmp)
    })
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let mut ready = Vec::new();
    if let Some(store) = scope.get_slot_mut::<SchedulerStore>() {
        let mut pending = Vec::with_capacity(store.pending.len());
        for task in std::mem::take(&mut store.pending) {
            if task.due_ms <= now {
                ready.push(task);
            } else {
                pending.push(task);
            }
        }
        store.pending = pending;
    }
    ready.sort_by(|left, right| {
        left.due_ms
            .total_cmp(&right.due_ms)
            .then_with(|| left.sequence.cmp(&right.sequence))
    });
    let ran = !ready.is_empty();
    for task in ready {
        let context = v8::Local::new(scope, &task.context);
        let task_scope = &mut v8::ContextScope::new(scope, context);
        let resolver = v8::Local::new(task_scope, &task.resolver);
        if let Some(callback) = task.callback {
            let callback = v8::Local::new(task_scope, &callback);
            v8::tc_scope!(let try_catch, task_scope);
            let receiver = v8::undefined(try_catch);
            if let Some(value) = callback.call(try_catch, receiver.into(), &[]) {
                let _ = resolver.resolve(try_catch, value);
            } else {
                let reason = try_catch
                    .exception()
                    .unwrap_or_else(|| v8::undefined(try_catch).into());
                let _ = resolver.reject(try_catch, reason);
            }
        } else {
            let value = v8::undefined(task_scope);
            let _ = resolver.resolve(task_scope, value.into());
        }
        task_scope.perform_microtask_checkpoint();
    }
    ran
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SchedulerStore>() {
        store.constructors.remove(&realm_id);
    }
}
