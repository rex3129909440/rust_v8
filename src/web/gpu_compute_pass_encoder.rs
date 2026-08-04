use std::collections::HashMap;

#[derive(Clone, Default)]
struct ComputePassRecord {
    ended: bool,
    dispatches: u64,
    debug_depth: u32,
}

#[derive(Default)]
pub(crate) struct GpuComputePassEncoderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ComputePassRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuComputePassEncoderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUComputePassEncoder", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuComputePassEncoderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUComputePassEncoder",
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
        "insertDebugMarker",
        1,
        insert_debug_marker,
    )?;
    crate::webidl::define_method(scope, prototype, "pushDebugGroup", 1, push_debug_group)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "dispatchWorkgroups",
        1,
        dispatch_workgroups,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "dispatchWorkgroupsIndirect",
        2,
        dispatch_workgroups_indirect,
    )?;
    crate::webidl::define_method(scope, prototype, "end", 0, end)?;
    crate::webidl::define_method(scope, prototype, "popDebugGroup", 0, pop_debug_group)?;
    crate::webidl::define_method(scope, prototype, "setBindGroup", 2, set_bind_group)?;
    crate::webidl::define_method(scope, prototype, "setPipeline", 1, set_pipeline)?;
    crate::webidl::define_method(scope, prototype, "writeTimestamp", 2, write_timestamp)?;
    crate::webidl::define_method(scope, prototype, "setImmediates", 2, set_immediates)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuComputePassEncoderStore>()
        .ok_or_else(|| "GPUComputePassEncoder state was not prepared".to_owned())?
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
    label: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPUComputePassEncoder".to_owned());
    }
    super::gpu_label_support::attach(scope, object, label);
    scope
        .get_slot_mut::<GpuComputePassEncoderStore>()
        .ok_or_else(|| "GPUComputePassEncoder state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ComputePassRecord::default(),
        );
    Ok(object)
}

fn with_record(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut ComputePassRecord),
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<GpuComputePassEncoderStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        if record.ended {
            crate::webidl::throw_type_error(scope, "GPUComputePassEncoder has ended");
            return false;
        }
        change(record);
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

fn get_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(label) = super::gpu_label_support::get(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &label)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::gpu_label_support::set(scope, arguments.this(), value) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn insert_debug_marker(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |_| {});
}
fn push_debug_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |v| v.debug_depth += 1);
}
fn pop_debug_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |v| {
        v.debug_depth = v.debug_depth.saturating_sub(1)
    });
}
fn dispatch_workgroups(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |v| v.dispatches += 1);
}
fn dispatch_workgroups_indirect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |v| v.dispatches += 1);
}
fn set_bind_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |_| {});
}
fn set_pipeline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |_| {});
}
fn write_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |_| {});
}
fn set_immediates(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = with_record(s, a.this(), |_| {});
}
fn end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<GpuComputePassEncoderStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        if record.ended {
            crate::webidl::throw_type_error(scope, "GPUComputePassEncoder has already ended");
        } else {
            record.ended = true;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuComputePassEncoderStore>() {
        store.constructor.remove(realm_id);
    }
}
