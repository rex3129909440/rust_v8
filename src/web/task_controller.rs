#[derive(Default)]
pub(crate) struct TaskControllerStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TaskControllerStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TaskController", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TaskControllerStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TaskController",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_method(scope, p, "setPriority", 1, set_priority)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::abort_controller::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TaskControllerStore>()
        .ok_or_else(|| "TaskController state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'TaskController': use new");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(a.get(0)).ok();
    let priority = options
        .and_then(|o| v8::String::new(scope, "priority").and_then(|k| o.get(scope, k.into())))
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(scope, v))
        .unwrap_or_else(|| "user-visible".to_owned());
    if !matches!(
        priority.as_str(),
        "user-blocking" | "user-visible" | "background"
    ) {
        crate::webidl::throw_type_error(scope, "Invalid task priority");
        return;
    }
    let signal = match super::task_signal::create(scope, priority, None) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    super::abort_controller::attach(scope, a.this(), signal);
    r.set(a.this().into())
}
fn set_priority(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(signal) = super::abort_controller::signal(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let priority = crate::webidl::value_to_string(scope, a.get(0));
    if !matches!(
        priority.as_str(),
        "user-blocking" | "user-visible" | "background"
    ) {
        crate::webidl::throw_type_error(scope, "Invalid task priority");
        return;
    }
    let signal = v8::Local::new(scope, &signal);
    super::task_signal::set_priority(scope, signal, priority);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TaskControllerStore>() {
        store.constructor.remove(realm_id);
    }
}
