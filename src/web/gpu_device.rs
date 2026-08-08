use std::collections::HashMap;

#[derive(Clone)]
struct DeviceRecord {
    features: v8::Global<v8::Object>,
    limits: v8::Global<v8::Object>,
    adapter_info: v8::Global<v8::Object>,
    lost: v8::Global<v8::Promise>,
    lost_resolver: v8::Global<v8::PromiseResolver>,
    queue: v8::Global<v8::Object>,
    on_uncaptured_error: Option<v8::Global<v8::Value>>,
    destroyed: bool,
    error_scopes: Vec<String>,
}

#[derive(Default)]
pub(crate) struct GpuDeviceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DeviceRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuDeviceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUDevice", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuDeviceStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUDevice",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "features", get_features)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "limits", get_limits)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "adapterInfo", get_adapter_info)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lost", get_lost)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "queue", get_queue)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onuncapturederror",
        get_on_uncaptured_error,
        set_on_uncaptured_error,
    )?;
    crate::webidl::define_accessor(scope, prototype, "label", get_label, set_label)?;
    crate::webidl::define_method(scope, prototype, "createBindGroup", 1, create_bind_group)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createBindGroupLayout",
        1,
        create_bind_group_layout,
    )?;
    crate::webidl::define_method(scope, prototype, "createBuffer", 1, create_buffer)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createCommandEncoder",
        0,
        create_command_encoder,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createComputePipeline",
        1,
        create_compute_pipeline,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createComputePipelineAsync",
        1,
        create_compute_pipeline_async,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createPipelineLayout",
        1,
        create_pipeline_layout,
    )?;
    crate::webidl::define_method(scope, prototype, "createQuerySet", 1, create_query_set)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createRenderBundleEncoder",
        1,
        create_render_bundle_encoder,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createRenderPipeline",
        1,
        create_render_pipeline,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createRenderPipelineAsync",
        1,
        create_render_pipeline_async,
    )?;
    crate::webidl::define_method(scope, prototype, "createSampler", 0, create_sampler)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createShaderModule",
        1,
        create_shader_module,
    )?;
    crate::webidl::define_method(scope, prototype, "createTexture", 1, create_texture)?;
    crate::webidl::define_method(scope, prototype, "destroy", 0, destroy)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "importExternalTexture",
        1,
        import_external_texture,
    )?;
    crate::webidl::define_method(scope, prototype, "popErrorScope", 0, pop_error_scope)?;
    crate::webidl::define_method(scope, prototype, "pushErrorScope", 1, push_error_scope)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuDeviceStore>()
        .ok_or_else(|| "GPUDevice state was not prepared".to_owned())?
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
        return Err("cannot create GPUDevice".to_owned());
    }
    super::event_target::attach(scope, object);
    super::gpu_label_support::attach(scope, object, String::new());
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
    let adapter_info = super::gpu_adapter_info::create(scope)?;
    let queue = super::gpu_queue::create(scope, String::new())?;
    let resolver =
        v8::PromiseResolver::new(scope).ok_or_else(|| "cannot create GPUDevice.lost".to_owned())?;
    let lost = resolver.get_promise(scope);
    let record = DeviceRecord {
        features: v8::Global::new(scope, features),
        limits: v8::Global::new(scope, limits),
        adapter_info: v8::Global::new(scope, adapter_info),
        lost: v8::Global::new(scope, lost),
        lost_resolver: v8::Global::new(scope, resolver),
        queue: v8::Global::new(scope, queue),
        on_uncaptured_error: None,
        destroyed: false,
        error_scopes: Vec::new(),
    };
    scope
        .get_slot_mut::<GpuDeviceStore>()
        .ok_or_else(|| "GPUDevice state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<DeviceRecord> {
    scope
        .get_slot::<GpuDeviceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn require_active(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    match record(scope, object) {
        Some(value) if !value.destroyed => true,
        Some(_) => {
            crate::webidl::throw_type_error(scope, "GPUDevice is destroyed");
            false
        }
        None => {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            false
        }
    }
}

fn descriptor_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: &v8::FunctionCallbackArguments<'s>,
) -> Option<v8::Local<'s, v8::Object>> {
    match v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        Ok(value) => Some(value),
        Err(_) => {
            crate::webidl::throw_type_error(scope, "A descriptor object is required");
            None
        }
    }
}

fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> v8::Local<'s, v8::Value> {
    crate::webidl::string(scope, name)
        .ok()
        .and_then(|key| object.get(scope, key.into()))
        .unwrap_or_else(|| v8::undefined(scope).into())
}

fn label(scope: &v8::PinScope<'_, '_>, descriptor: v8::Local<'_, v8::Object>) -> String {
    let value = member(scope, descriptor, "label");
    if value.is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, value)
    }
}

fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    value: Result<v8::Local<'_, v8::Object>, String>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_features(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.features).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_limits(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.limits).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_adapter_info(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.adapter_info).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_lost(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.lost).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_queue(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.queue).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_on_uncaptured_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|value| value.on_uncaptured_error);
    super::window_event_handler_support::return_handler(scope, handler, result);
}
fn set_on_uncaptured_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<GpuDeviceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.on_uncaptured_error = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = super::gpu_label_support::get(s, a.this()) {
        if let Some(v) = v8::String::new(s, &v) {
            r.set(v.into());
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if !super::gpu_label_support::set(s, a.this(), v) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn create_bind_group<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let value = super::gpu_bind_group::create(s, label);
    return_object(s, value, r);
}
fn create_bind_group_layout<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let value = super::gpu_bind_group_layout::create(s, label);
    return_object(s, value, r);
}
fn create_buffer<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let size = member(s, d, "size").integer_value(s).unwrap_or(0).max(0) as usize;
    let usage = member(s, d, "usage").uint32_value(s).unwrap_or(0);
    let mapped = member(s, d, "mappedAtCreation").boolean_value(s);
    let label = label(s, d);
    let value = super::gpu_buffer::create(s, size, usage, mapped, label);
    return_object(s, value, r);
}
fn create_command_encoder(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let label = v8::Local::<v8::Object>::try_from(a.get(0))
        .map(|d| label(s, d))
        .unwrap_or_default();
    let value = super::gpu_command_encoder::create(s, label);
    return_object(s, value, r);
}
fn create_compute_pipeline<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let value = super::gpu_compute_pipeline::create(s, label);
    return_object(s, value, r);
}
fn create_compute_pipeline_async<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    mut r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    if let Ok(value) = super::gpu_compute_pipeline::create(s, label)
        && let Ok(promise) = super::writable_stream::resolved_promise(s, value.into())
    {
        r.set(promise.into());
    }
}
fn create_pipeline_layout<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let value = super::gpu_pipeline_layout::create(s, label);
    return_object(s, value, r);
}
fn create_query_set<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let count = member(s, d, "count").uint32_value(s).unwrap_or(0);
    let kind = crate::webidl::value_to_string(s, member(s, d, "type"));
    let label = label(s, d);
    let value = super::gpu_query_set::create(s, count, kind, label);
    return_object(s, value, r);
}
fn create_render_bundle_encoder<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let value = super::gpu_render_bundle_encoder::create(s, label);
    return_object(s, value, r);
}
fn create_render_pipeline<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let value = super::gpu_render_pipeline::create(s, label);
    return_object(s, value, r);
}
fn create_render_pipeline_async<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    mut r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    if let Ok(value) = super::gpu_render_pipeline::create(s, label)
        && let Ok(promise) = super::writable_stream::resolved_promise(s, value.into())
    {
        r.set(promise.into());
    }
}
fn create_sampler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let label = v8::Local::<v8::Object>::try_from(a.get(0))
        .map(|d| label(s, d))
        .unwrap_or_default();
    let value = super::gpu_sampler::create(s, label);
    return_object(s, value, r);
}
fn create_shader_module<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let code = crate::webidl::value_to_string(s, member(s, d, "code"));
    let value = super::gpu_shader_module::create(s, label, code);
    return_object(s, value, r);
}
fn create_texture<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let usage = member(s, d, "usage").uint32_value(s).unwrap_or(0);
    let label = label(s, d);
    let value = super::gpu_texture::create(s, 1, 1, 1, usage, label);
    return_object(s, value, r);
}
fn import_external_texture<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    r: v8::ReturnValue<'s>,
) {
    if !require_active(s, a.this()) {
        return;
    }
    let Some(d) = descriptor_object(s, &a) else {
        return;
    };
    let label = label(s, d);
    let value = super::gpu_external_texture::create(s, label);
    return_object(s, value, r);
}

fn push_error_scope(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let filter = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<GpuDeviceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.error_scopes.push(filter);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn pop_error_scope(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope.get_slot_mut::<GpuDeviceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.error_scopes.pop().is_none() {
        let message = v8::String::new(scope, "No error scope to pop").unwrap();
        if let Ok(promise) = super::writable_stream::rejected_promise(scope, message.into()) {
            result.set(promise.into());
        }
    } else if let Ok(promise) =
        super::writable_stream::resolved_promise(scope, v8::null(scope).into())
    {
        result.set(promise.into());
    }
}
fn destroy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let resolver = {
        let Some(record) = scope.get_slot_mut::<GpuDeviceStore>().and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        }) else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        };
        if record.destroyed {
            return;
        }
        record.destroyed = true;
        record.lost_resolver.clone()
    };
    if let Ok(info) = super::gpu_device_lost_info::create(
        scope,
        "destroyed".to_owned(),
        "GPUDevice was destroyed".to_owned(),
    ) {
        let resolver = v8::Local::new(scope, &resolver);
        let _ = resolver.resolve(scope, info.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuDeviceStore>() {
        store.constructor.remove(realm_id);
    }
}
