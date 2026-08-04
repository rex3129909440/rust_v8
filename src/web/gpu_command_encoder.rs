use std::collections::HashMap;

#[derive(Clone, Default)]
struct CommandEncoderRecord {
    finished: bool,
    command_count: u64,
    debug_depth: u32,
}

#[derive(Default)]
pub(crate) struct GpuCommandEncoderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CommandEncoderRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuCommandEncoderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUCommandEncoder", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuCommandEncoderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUCommandEncoder",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "label", get_label, set_label)?;
    crate::webidl::define_method(scope, prototype, "beginComputePass", 0, begin_compute_pass)?;
    crate::webidl::define_method(scope, prototype, "beginRenderPass", 1, begin_render_pass)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "copyBufferToTexture",
        3,
        copy_buffer_to_texture,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "copyTextureToBuffer",
        3,
        copy_texture_to_buffer,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "copyTextureToTexture",
        3,
        copy_texture_to_texture,
    )?;
    crate::webidl::define_method(scope, prototype, "finish", 0, finish)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "insertDebugMarker",
        1,
        insert_debug_marker,
    )?;
    crate::webidl::define_method(scope, prototype, "pushDebugGroup", 1, push_debug_group)?;
    crate::webidl::define_method(scope, prototype, "clearBuffer", 1, clear_buffer)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "copyBufferToBuffer",
        2,
        copy_buffer_to_buffer,
    )?;
    crate::webidl::define_method(scope, prototype, "popDebugGroup", 0, pop_debug_group)?;
    crate::webidl::define_method(scope, prototype, "resolveQuerySet", 5, resolve_query_set)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuCommandEncoderStore>()
        .ok_or_else(|| "GPUCommandEncoder state was not prepared".to_owned())?
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
        return Err("cannot create GPUCommandEncoder".to_owned());
    }
    super::gpu_label_support::attach(scope, object, label);
    scope
        .get_slot_mut::<GpuCommandEncoderStore>()
        .ok_or_else(|| "GPUCommandEncoder state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CommandEncoderRecord::default(),
        );
    Ok(object)
}

fn change(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    update: impl FnOnce(&mut CommandEncoderRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<GpuCommandEncoderStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    if record.finished {
        crate::webidl::throw_type_error(scope, "GPUCommandEncoder has already finished");
        return false;
    }
    update(record);
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
    let label = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::gpu_label_support::set(scope, arguments.this(), label) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn begin_compute_pass(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !change(scope, arguments.this(), |v| v.command_count += 1) {
        return;
    }
    match super::gpu_compute_pass_encoder::create(scope, String::new()) {
        Ok(pass) => result.set(pass.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn begin_render_pass(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !change(scope, arguments.this(), |v| v.command_count += 1) {
        return;
    }
    match super::gpu_render_pass_encoder::create(scope, String::new()) {
        Ok(pass) => result.set(pass.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn finish(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot_mut::<GpuCommandEncoderStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.finished {
        crate::webidl::throw_type_error(scope, "GPUCommandEncoder has already finished");
        return;
    }
    record.finished = true;
    match super::gpu_command_buffer::create(scope, String::new()) {
        Ok(buffer) => result.set(buffer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn record_command(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>) {
    let _ = change(s, a.this(), |v| v.command_count += 1);
}
fn copy_buffer_to_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    record_command(s, a);
}
fn copy_texture_to_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    record_command(s, a);
}
fn copy_texture_to_texture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    record_command(s, a);
}
fn clear_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    record_command(s, a);
}
fn copy_buffer_to_buffer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    record_command(s, a);
}
fn resolve_query_set(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    record_command(s, a);
}
fn insert_debug_marker(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = change(s, a.this(), |_| {});
}
fn push_debug_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = change(s, a.this(), |v| v.debug_depth += 1);
}
fn pop_debug_group(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = change(s, a.this(), |v| {
        v.debug_depth = v.debug_depth.saturating_sub(1)
    });
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuCommandEncoderStore>() {
        store.constructor.remove(realm_id);
    }
}
