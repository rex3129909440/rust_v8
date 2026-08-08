use std::collections::HashMap;
#[derive(Clone, Default)]
struct RenderPassRecord {
    ended: bool,
    commands: u64,
    occlusion_active: bool,
    debug_depth: u32,
}
#[derive(Default)]
pub(crate) struct GpuRenderPassEncoderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RenderPassRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuRenderPassEncoderStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPURenderPassEncoder", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuRenderPassEncoderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPURenderPassEncoder",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "label", get_label, set_label)?;
    crate::webidl::define_method(scope, prototype, "executeBundles", 1, execute_bundles)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "insertDebugMarker",
        1,
        insert_debug_marker,
    )?;
    crate::webidl::define_method(scope, prototype, "pushDebugGroup", 1, push_debug_group)?;
    crate::webidl::define_method(scope, prototype, "setBlendConstant", 1, set_blend_constant)?;
    crate::webidl::define_method(scope, prototype, "setIndexBuffer", 2, set_index_buffer)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "beginOcclusionQuery",
        1,
        begin_occlusion_query,
    )?;
    crate::webidl::define_method(scope, prototype, "draw", 1, draw)?;
    crate::webidl::define_method(scope, prototype, "drawIndexed", 1, draw_indexed)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "drawIndexedIndirect",
        2,
        draw_indexed_indirect,
    )?;
    crate::webidl::define_method(scope, prototype, "drawIndirect", 2, draw_indirect)?;
    crate::webidl::define_method(scope, prototype, "end", 0, end)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "endOcclusionQuery",
        0,
        end_occlusion_query,
    )?;
    crate::webidl::define_method(scope, prototype, "popDebugGroup", 0, pop_debug_group)?;
    crate::webidl::define_method(scope, prototype, "setBindGroup", 2, set_bind_group)?;
    crate::webidl::define_method(scope, prototype, "setPipeline", 1, set_pipeline)?;
    crate::webidl::define_method(scope, prototype, "setScissorRect", 4, set_scissor_rect)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setStencilReference",
        1,
        set_stencil_reference,
    )?;
    crate::webidl::define_method(scope, prototype, "setVertexBuffer", 2, set_vertex_buffer)?;
    crate::webidl::define_method(scope, prototype, "setViewport", 6, set_viewport)?;
    crate::webidl::define_method(scope, prototype, "writeTimestamp", 2, write_timestamp)?;
    crate::webidl::define_method(scope, prototype, "setImmediates", 2, set_immediates)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuRenderPassEncoderStore>()
        .ok_or_else(|| "GPURenderPassEncoder state was not prepared".to_owned())?
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
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPURenderPassEncoder".to_owned());
    }
    super::gpu_label_support::attach(scope, object, label);
    scope
        .get_slot_mut::<GpuRenderPassEncoderStore>()
        .ok_or_else(|| "GPURenderPassEncoder state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            RenderPassRecord::default(),
        );
    Ok(object)
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut RenderPassRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<GpuRenderPassEncoderStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    if record.ended {
        crate::webidl::throw_type_error(scope, "GPURenderPassEncoder has ended");
        return false;
    }
    change(record);
    true
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
fn command(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>) {
    let _ = update(s, a.this(), |v| v.commands += 1);
}
fn execute_bundles(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn insert_debug_marker(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_blend_constant(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_index_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn draw(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn draw_indexed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn draw_indexed_indirect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn draw_indirect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_bind_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_pipeline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_scissor_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_stencil_reference(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_vertex_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_viewport(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn write_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn set_immediates(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    command(s, a)
}
fn push_debug_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = update(s, a.this(), |v| v.debug_depth += 1);
}
fn pop_debug_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = update(s, a.this(), |v| {
        v.debug_depth = v.debug_depth.saturating_sub(1)
    });
}
fn begin_occlusion_query(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = update(s, a.this(), |v| v.occlusion_active = true);
}
fn end_occlusion_query(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = update(s, a.this(), |v| v.occlusion_active = false);
}
fn end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<GpuRenderPassEncoderStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        if record.ended {
            crate::webidl::throw_type_error(scope, "GPURenderPassEncoder has ended")
        } else {
            record.ended = true
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuRenderPassEncoderStore>() {
        store.constructor.remove(realm_id);
    }
}
