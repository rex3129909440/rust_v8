use std::collections::HashMap;

#[derive(Clone, Default)]
struct SystemRecord {
    on_device_change: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct XrSystemStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SystemRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrSystemStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRSystem", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrSystemStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRSystem",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ondevicechange",
        get_on_device_change,
        set_on_device_change,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "isSessionSupported",
        1,
        is_session_supported,
    )?;
    crate::webidl::define_method(scope, prototype, "requestSession", 1, request_session)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrSystemStore>()
        .ok_or_else(|| "XRSystem state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRSystem".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<XrSystemStore>()
        .ok_or_else(|| "XRSystem state missing".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), SystemRecord::default());
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<SystemRecord> {
    scope
        .get_slot::<XrSystemStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_on_device_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(system) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, system.on_device_change, result);
}

fn set_on_device_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    let Some(system) = scope.get_slot_mut::<XrSystemStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    system.on_device_change = handler;
}

fn supported_mode(scope: &v8::PinScope<'_, '_>, mode: &str) -> bool {
    crate::fingerprint::edge(scope)
        .xr
        .supported_session_modes
        .iter()
        .any(|configured| configured == mode)
}

fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into())
    }
}

fn is_session_supported(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mode = crate::webidl::value_to_string(scope, arguments.get(0));
    let supported = v8::Boolean::new(scope, supported_mode(scope, &mode));
    resolve(scope, supported.into(), result);
}

fn request_session(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mode = crate::webidl::value_to_string(scope, arguments.get(0));
    if !supported_mode(scope, &mode) {
        crate::webidl::throw_type_error(scope, "Unsupported XR session mode");
        return;
    }
    match super::xr_session::create(scope, mode) {
        Ok(session) => resolve(scope, session.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
