use std::collections::HashMap;

#[derive(Clone)]
struct AdapterInfoRecord {
    vendor: String,
    architecture: String,
    device: String,
    description: String,
    subgroup_min_size: i32,
    subgroup_max_size: i32,
    fallback: bool,
}

#[derive(Default)]
pub(crate) struct GpuAdapterInfoStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AdapterInfoRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuAdapterInfoStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUAdapterInfo", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<GpuAdapterInfoStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUAdapterInfo",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "vendor", get_vendor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "architecture", get_architecture)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "device", get_device)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "description", get_description)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "subgroupMinSize",
        get_subgroup_min_size,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "subgroupMaxSize",
        get_subgroup_max_size,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "isFallbackAdapter",
        get_is_fallback_adapter,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuAdapterInfoStore>()
        .ok_or_else(|| "GPUAdapterInfo state was not prepared".to_owned())?
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
        return Err("cannot create GPUAdapterInfo".to_owned());
    }
    let profile = crate::fingerprint::edge(scope).rendering.webgpu.clone();
    scope
        .get_slot_mut::<GpuAdapterInfoStore>()
        .ok_or_else(|| "GPUAdapterInfo state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AdapterInfoRecord {
                vendor: profile.vendor,
                architecture: profile.architecture,
                device: if profile.developer_features {
                    profile.device
                } else {
                    String::new()
                },
                description: if profile.developer_features {
                    profile.description
                } else {
                    String::new()
                },
                subgroup_min_size: profile.subgroup_min_size as i32,
                subgroup_max_size: profile.subgroup_max_size as i32,
                fallback: profile.is_fallback_adapter,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AdapterInfoRecord> {
    scope
        .get_slot::<GpuAdapterInfoStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn string_field(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(AdapterInfoRecord) -> String,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &select(record)) {
        result.set(value.into());
    }
}

fn get_vendor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_field(s, a, r, |v| v.vendor);
}
fn get_architecture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_field(s, a, r, |v| v.architecture);
}
fn get_device(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_field(s, a, r, |v| v.device);
}
fn get_description(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_field(s, a, r, |v| v.description);
}
fn get_subgroup_min_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.subgroup_min_size).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_subgroup_max_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.subgroup_max_size).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_is_fallback_adapter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.fallback).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuAdapterInfoStore>() {
        store.constructor.remove(realm_id);
    }
}
