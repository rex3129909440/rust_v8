use std::collections::HashMap;

#[derive(Clone)]
struct AdapterRecord {
    features: v8::Global<v8::Object>,
    limits: v8::Global<v8::Object>,
    info: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct GpuAdapterStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AdapterRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuAdapterStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUAdapter", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<GpuAdapterStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUAdapter",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "features", get_features)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "limits", get_limits)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "info", get_info)?;
    crate::webidl::define_method(scope, prototype, "requestDevice", 0, request_device)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuAdapterStore>()
        .ok_or_else(|| "GPUAdapter state was not prepared".to_owned())?
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
        return Err("cannot create GPUAdapter".to_owned());
    }
    let configured_features = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .features
        .clone();
    let feature_names = configured_features
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let features = super::gpu_supported_features::create(scope, &feature_names)?;
    let limits = super::gpu_supported_limits::create(scope)?;
    let info = super::gpu_adapter_info::create(scope)?;
    let record = AdapterRecord {
        features: v8::Global::new(scope, features),
        limits: v8::Global::new(scope, limits),
        info: v8::Global::new(scope, info),
    };
    scope
        .get_slot_mut::<GpuAdapterStore>()
        .ok_or_else(|| "GPUAdapter state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AdapterRecord> {
    scope
        .get_slot::<GpuAdapterStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn object_field(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(AdapterRecord) -> v8::Global<v8::Object>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, &select(record)).into());
}

fn get_features(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_field(s, a, r, |v| v.features);
}
fn get_limits(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_field(s, a, r, |v| v.limits);
}
fn get_info(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_field(s, a, r, |v| v.info);
}

fn request_device(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(device) = super::gpu_device::create(scope)
        && let Ok(promise) = super::writable_stream::resolved_promise(scope, device.into())
    {
        result.set(promise.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuAdapterStore>() {
        store.constructor.remove(realm_id);
    }
}
