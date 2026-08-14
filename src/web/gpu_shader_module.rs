use std::collections::HashMap;
#[derive(Clone)]
struct ShaderRecord {
    code: String,
}
#[derive(Default)]
pub(crate) struct GpuShaderModuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ShaderRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuShaderModuleStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUShaderModule", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuShaderModuleStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUShaderModule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "label", get_label, set_label)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getCompilationInfo",
        0,
        get_compilation_info,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuShaderModuleStore>()
        .ok_or_else(|| "GPUShaderModule state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
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
    label: String,
    code: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPUShaderModule".to_owned());
    }
    super::gpu_label_support::attach(scope, object, label);
    scope
        .get_slot_mut::<GpuShaderModuleStore>()
        .ok_or_else(|| "GPUShaderModule state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), ShaderRecord { code });
    Ok(object)
}
fn get_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(label) = super::gpu_label_support::get(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &label)
    {
        result.set(value.into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let label = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::gpu_label_support::set(scope, arguments.this(), label) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_compilation_info(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot::<GpuShaderModuleStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "GPUShaderModule",
            "getCompilationInfo",
            result,
        );
        return;
    };
    let messages = if record.code.trim().is_empty() {
        vec![super::gpu_compilation_message::CompilationMessageRecord {
            message: "WGSL source is empty".to_owned(),
            kind: "error".to_owned(),
            line_num: 1,
            line_pos: 1,
            offset: 0,
            length: 0,
        }]
    } else {
        Vec::new()
    };
    if let Ok(info) = super::gpu_compilation_info::create(scope, messages)
        && let Ok(promise) = super::writable_stream::resolved_promise(scope, info.into())
    {
        result.set(promise.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuShaderModuleStore>() {
        store.constructor.remove(realm_id);
    }
}
