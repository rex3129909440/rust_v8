use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct GpuSupportedLimitsStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuSupportedLimitsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUSupportedLimits", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuSupportedLimitsStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUSupportedLimits",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxTextureDimension1D",
        max_texture_dimension_1d,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxTextureDimension2D",
        max_texture_dimension_2d,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxTextureDimension3D",
        max_texture_dimension_3d,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxTextureArrayLayers",
        max_texture_array_layers,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "maxBindGroups", max_bind_groups)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxBindGroupsPlusVertexBuffers",
        max_bind_groups_plus_vertex_buffers,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxBindingsPerBindGroup",
        max_bindings_per_bind_group,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxDynamicUniformBuffersPerPipelineLayout",
        max_dynamic_uniform_buffers,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxDynamicStorageBuffersPerPipelineLayout",
        max_dynamic_storage_buffers,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxSampledTexturesPerShaderStage",
        max_sampled_textures,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxSamplersPerShaderStage",
        max_samplers,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxStorageBuffersPerShaderStage",
        max_storage_buffers,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxStorageTexturesPerShaderStage",
        max_storage_textures,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxUniformBuffersPerShaderStage",
        max_uniform_buffers,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxUniformBufferBindingSize",
        max_uniform_buffer_binding_size,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxStorageBufferBindingSize",
        max_storage_buffer_binding_size,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "minUniformBufferOffsetAlignment",
        min_uniform_alignment,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "minStorageBufferOffsetAlignment",
        min_storage_alignment,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxVertexBuffers",
        max_vertex_buffers,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "maxBufferSize", max_buffer_size)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxVertexAttributes",
        max_vertex_attributes,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxVertexBufferArrayStride",
        max_vertex_stride,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxInterStageShaderVariables",
        max_inter_stage_variables,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxColorAttachments",
        max_color_attachments,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxColorAttachmentBytesPerSample",
        max_color_bytes,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxComputeWorkgroupStorageSize",
        max_compute_storage,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxComputeInvocationsPerWorkgroup",
        max_compute_invocations,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxComputeWorkgroupSizeX",
        max_compute_size_x,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxComputeWorkgroupSizeY",
        max_compute_size_y,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxComputeWorkgroupSizeZ",
        max_compute_size_z,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxComputeWorkgroupsPerDimension",
        max_compute_workgroups,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxImmediateSize",
        max_immediate_size,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxStorageBuffersInFragmentStage",
        max_storage_buffers_fragment,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxStorageTexturesInFragmentStage",
        max_storage_textures_fragment,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxStorageBuffersInVertexStage",
        max_storage_buffers_vertex,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxStorageTexturesInVertexStage",
        max_storage_textures_vertex,
    )?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuSupportedLimitsStore>()
        .ok_or_else(|| "GPUSupportedLimits state was not prepared".to_owned())?
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
        return Err("cannot create GPUSupportedLimits".to_owned());
    }
    scope
        .get_slot_mut::<GpuSupportedLimitsStore>()
        .ok_or_else(|| "GPUSupportedLimits state was not prepared".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    limit: f64,
) {
    if scope
        .get_slot::<GpuSupportedLimitsStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Number::new(scope, limit).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn max_texture_dimension_1d(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_texture_dimension_1d;
    value(scope, arguments, result, configured as f64)
}

fn max_texture_dimension_2d(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_texture_dimension_2d;
    value(scope, arguments, result, configured as f64)
}

fn max_texture_dimension_3d(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_texture_dimension_3d;
    value(scope, arguments, result, configured as f64)
}

fn max_texture_array_layers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_texture_array_layers;
    value(scope, arguments, result, configured as f64)
}

fn max_bind_groups(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_bind_groups;
    value(scope, arguments, result, configured as f64)
}

fn max_bind_groups_plus_vertex_buffers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_bind_groups_plus_vertex_buffers;
    value(scope, arguments, result, configured as f64)
}

fn max_bindings_per_bind_group(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_bindings_per_bind_group;
    value(scope, arguments, result, configured as f64)
}

fn max_dynamic_uniform_buffers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_dynamic_uniform_buffers_per_pipeline_layout;
    value(scope, arguments, result, configured as f64)
}

fn max_dynamic_storage_buffers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_dynamic_storage_buffers_per_pipeline_layout;
    value(scope, arguments, result, configured as f64)
}

fn max_sampled_textures(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_sampled_textures_per_shader_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_samplers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_samplers_per_shader_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_storage_buffers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_storage_buffers_per_shader_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_storage_textures(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_storage_textures_per_shader_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_uniform_buffers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_uniform_buffers_per_shader_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_uniform_buffer_binding_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_uniform_buffer_binding_size;
    value(scope, arguments, result, configured as f64)
}

fn max_storage_buffer_binding_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_storage_buffer_binding_size;
    value(scope, arguments, result, configured as f64)
}

fn min_uniform_alignment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .min_uniform_buffer_offset_alignment;
    value(scope, arguments, result, configured as f64)
}

fn min_storage_alignment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .min_storage_buffer_offset_alignment;
    value(scope, arguments, result, configured as f64)
}

fn max_vertex_buffers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_vertex_buffers;
    value(scope, arguments, result, configured as f64)
}

fn max_buffer_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_buffer_size;
    value(scope, arguments, result, configured as f64)
}

fn max_vertex_attributes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_vertex_attributes;
    value(scope, arguments, result, configured as f64)
}

fn max_vertex_stride(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_vertex_buffer_array_stride;
    value(scope, arguments, result, configured as f64)
}

fn max_inter_stage_variables(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_inter_stage_shader_variables;
    value(scope, arguments, result, configured as f64)
}

fn max_color_attachments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_color_attachments;
    value(scope, arguments, result, configured as f64)
}

fn max_color_bytes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_color_attachment_bytes_per_sample;
    value(scope, arguments, result, configured as f64)
}

fn max_compute_storage(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_compute_workgroup_storage_size;
    value(scope, arguments, result, configured as f64)
}

fn max_compute_invocations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_compute_invocations_per_workgroup;
    value(scope, arguments, result, configured as f64)
}

fn max_compute_size_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_compute_workgroup_size_x;
    value(scope, arguments, result, configured as f64)
}

fn max_compute_size_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_compute_workgroup_size_y;
    value(scope, arguments, result, configured as f64)
}

fn max_compute_size_z(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_compute_workgroup_size_z;
    value(scope, arguments, result, configured as f64)
}

fn max_compute_workgroups(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_compute_workgroups_per_dimension;
    value(scope, arguments, result, configured as f64)
}

fn max_immediate_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_immediate_size;
    value(scope, arguments, result, configured as f64)
}

fn max_storage_buffers_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_storage_buffers_in_fragment_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_storage_textures_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_storage_textures_in_fragment_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_storage_buffers_vertex(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_storage_buffers_in_vertex_stage;
    value(scope, arguments, result, configured as f64)
}

fn max_storage_textures_vertex(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let configured = crate::fingerprint::edge(scope)
        .rendering
        .webgpu
        .max_storage_textures_in_vertex_stage;
    value(scope, arguments, result, configured as f64)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuSupportedLimitsStore>() {
        store.constructor.remove(realm_id);
    }
}
