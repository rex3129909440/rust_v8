use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct CreateMonitorStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
    handlers: HashMap<i32, v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CreateMonitorStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CreateMonitor", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CreateMonitorStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "CreateMonitor",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ondownloadprogress",
        get_handler,
        set_handler,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CreateMonitorStore>()
        .ok_or_else(|| "CreateMonitor state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CreateMonitor': Illegal constructor",
    );
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CreateMonitor".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<CreateMonitorStore>()
        .ok_or_else(|| "CreateMonitor state was not prepared".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}
fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let valid = scope
        .get_slot::<CreateMonitorStore>()
        .is_some_and(|store| store.instances.contains(&identity));
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let handler = scope
        .get_slot::<CreateMonitorStore>()
        .and_then(|store| {
            store
                .handlers
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    super::window_event_handler_support::return_handler(scope, handler, result);
}
fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let valid = scope
        .get_slot::<CreateMonitorStore>()
        .is_some_and(|store| store.instances.contains(&identity));
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<CreateMonitorStore>() {
        if let Some(handler) = handler {
            store.handlers.insert(identity, handler);
        } else {
            store.handlers.remove(&identity);
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CreateMonitorStore>() {
        store.constructor.remove(realm_id);
    }
}
