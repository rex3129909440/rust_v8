use std::collections::HashMap;
#[derive(Clone, Default)]
struct IdleRecord {
    started: bool,
    onchange: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct IdleDetectorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdleRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdleDetectorStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IdleDetector", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<IdleDetectorStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IdleDetector",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "userState", get_user_state)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "screenState", get_screen_state)?;
    crate::webidl::define_accessor(scope, prototype, "onchange", get_onchange, set_onchange)?;
    crate::webidl::define_method(scope, prototype, "start", 0, start)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let request = crate::webidl::create_function(
        scope,
        "requestPermission",
        0,
        v8::ConstructorBehavior::Throw,
        request_permission,
    )?;
    let key = crate::webidl::string(scope, "requestPermission")?;
    let _ = constructor.define_own_property(
        scope,
        key.into(),
        request.into(),
        v8::PropertyAttribute::NONE,
    );
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdleDetectorStore>()
        .ok_or_else(|| "IdleDetector state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Please use the 'new' operator");
        return;
    }
    super::event_target::attach(scope, arguments.this());
    scope
        .get_slot_mut::<IdleDetectorStore>()
        .expect("IdleDetector state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            IdleRecord::default(),
        );
    result.set(arguments.this().into())
}
fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<IdleRecord> {
    scope
        .get_slot::<IdleDetectorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    active: &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if record.started {
            if let Some(value) = v8::String::new(scope, active) {
                result.set(value.into())
            }
        } else {
            result.set(v8::null(scope).into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_user_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    state(s, a, r, "active")
}
fn get_screen_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    state(s, a, r, "unlocked")
}
fn get_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        scope,
        record(scope, arguments.this()).and_then(|v| v.onchange),
        result,
    )
}
fn set_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<IdleDetectorStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.onchange = handler
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<IdleDetectorStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.started = true;
        let value = v8::undefined(scope);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
            result.set(promise.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn request_permission(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = v8::String::new(scope, "granted")
        && let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into())
    {
        result.set(promise.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdleDetectorStore>() {
        store.constructor.remove(realm_id);
    }
}
