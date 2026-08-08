use std::collections::HashMap;

#[derive(Clone, Default)]
struct QueueRecord {
    submissions: u64,
    bytes_written: u64,
}
#[derive(Default)]
pub(crate) struct GpuQueueStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, QueueRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuQueueStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUQueue", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuQueueStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUQueue",
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
        "copyExternalImageToTexture",
        3,
        copy_external_image_to_texture,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "onSubmittedWorkDone",
        0,
        on_submitted_work_done,
    )?;
    crate::webidl::define_method(scope, prototype, "submit", 1, submit)?;
    crate::webidl::define_method(scope, prototype, "writeBuffer", 3, write_buffer)?;
    crate::webidl::define_method(scope, prototype, "writeTexture", 4, write_texture)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuQueueStore>()
        .ok_or_else(|| "GPUQueue state was not prepared".to_owned())?
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
        return Err("cannot create GPUQueue".to_owned());
    }
    super::gpu_label_support::attach(scope, object, label);
    scope
        .get_slot_mut::<GpuQueueStore>()
        .ok_or_else(|| "GPUQueue state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), QueueRecord::default());
    Ok(object)
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut QueueRecord),
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<GpuQueueStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
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
fn submit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let count = v8::Local::<v8::Array>::try_from(arguments.get(0))
        .map(|v| v.length() as u64)
        .unwrap_or(0);
    let _ = update(scope, arguments.this(), |record| {
        record.submissions += count
    });
}
fn write_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let amount = arguments.get(4).uint32_value(scope).unwrap_or(0) as u64;
    let _ = update(scope, arguments.this(), |record| {
        record.bytes_written += amount
    });
}
fn write_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = update(scope, arguments.this(), |record| record.bytes_written += 1);
}
fn copy_external_image_to_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let _ = update(scope, arguments.this(), |record| record.submissions += 1);
}
fn on_submitted_work_done(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if update(scope, arguments.this(), |_| {}) {
        let value = v8::undefined(scope);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
            result.set(promise.into())
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuQueueStore>() {
        store.constructor.remove(realm_id);
    }
}
