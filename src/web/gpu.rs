#[derive(Default)]
pub(crate) struct GpuStore {
    constructor: crate::webidl::RealmConstructor,
    instances: std::collections::HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPU", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<GpuStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPU",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "wgslLanguageFeatures",
        get_wgsl_language_features,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getPreferredCanvasFormat",
        0,
        get_preferred_canvas_format,
    )?;
    crate::webidl::define_method(scope, prototype, "requestAdapter", 0, request_adapter)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuStore>()
        .ok_or_else(|| "GPU state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPU".to_owned());
    }
    scope
        .get_slot_mut::<GpuStore>()
        .ok_or_else(|| "GPU state was not prepared".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<GpuStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}

fn get_wgsl_language_features(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::wgsl_language_features::create(scope) {
        Ok(features) => result.set(features.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_preferred_canvas_format(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = v8::String::new(scope, "bgra8unorm") {
        result.set(value.into());
    }
}

fn request_adapter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "GPU", "requestAdapter", result);
        return;
    }
    if !crate::fingerprint::edge(scope).rendering.webgpu.available {
        let null = v8::null(scope);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, null.into()) {
            result.set(promise.into());
        }
        return;
    }
    if let Ok(adapter) = super::gpu_adapter::create(scope)
        && let Ok(promise) = super::writable_stream::resolved_promise(scope, adapter.into())
    {
        result.set(promise.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuStore>() {
        store.constructor.remove(realm_id);
    }
}
