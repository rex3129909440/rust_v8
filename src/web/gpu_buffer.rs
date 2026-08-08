use std::collections::HashMap;

#[derive(Clone)]
struct BufferRecord {
    bytes: Vec<u8>,
    usage: u32,
    map_state: String,
    destroyed: bool,
}

#[derive(Default)]
pub(crate) struct GpuBufferStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BufferRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuBufferStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUBuffer", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<GpuBufferStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUBuffer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "usage", get_usage)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mapState", get_map_state)?;
    crate::webidl::define_accessor(scope, prototype, "label", get_label, set_label)?;
    crate::webidl::define_method(scope, prototype, "destroy", 0, destroy)?;
    crate::webidl::define_method(scope, prototype, "getMappedRange", 0, get_mapped_range)?;
    crate::webidl::define_method(scope, prototype, "mapAsync", 1, map_async)?;
    crate::webidl::define_method(scope, prototype, "unmap", 0, unmap)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuBufferStore>()
        .ok_or_else(|| "GPUBuffer state was not prepared".to_owned())?
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
    size: usize,
    usage: u32,
    mapped_at_creation: bool,
    label: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if size % 4 != 0 {
        return Err("GPUBuffer size must be a multiple of 4".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPUBuffer".to_owned());
    }
    super::gpu_label_support::attach(scope, object, label);
    scope
        .get_slot_mut::<GpuBufferStore>()
        .ok_or_else(|| "GPUBuffer state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            BufferRecord {
                bytes: vec![0; size],
                usage,
                map_state: if mapped_at_creation {
                    "mapped".to_owned()
                } else {
                    "unmapped".to_owned()
                },
                destroyed: false,
            },
        );
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<BufferRecord> {
    scope
        .get_slot::<GpuBufferStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.bytes.len() as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_usage(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.usage).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_map_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &record.map_state)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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
    let label = crate::webidl::value_to_string(scope, arguments.get(0));
    if !super::gpu_label_support::set(scope, arguments.this(), label) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn destroy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = scope.get_slot_mut::<GpuBufferStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.destroyed = true;
    record.map_state = "unmapped".to_owned();
    record.bytes.clear();
}

fn map_async(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope.get_slot_mut::<GpuBufferStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.destroyed {
        let message = v8::String::new(scope, "GPUBuffer is destroyed").unwrap();
        if let Ok(promise) = super::writable_stream::rejected_promise(scope, message.into()) {
            result.set(promise.into());
        }
        return;
    }
    record.map_state = "mapped".to_owned();
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn get_mapped_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.map_state != "mapped" || record.destroyed {
        crate::webidl::throw_type_error(scope, "GPUBuffer is not mapped");
        return;
    }
    let offset = arguments.get(0).uint32_value(scope).unwrap_or(0) as usize;
    let length = arguments
        .get(1)
        .uint32_value(scope)
        .map(|value| value as usize)
        .unwrap_or_else(|| record.bytes.len().saturating_sub(offset));
    let end = offset.saturating_add(length).min(record.bytes.len());
    if offset > end {
        crate::webidl::throw_type_error(scope, "Mapped range is out of bounds");
        return;
    }
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(record.bytes[offset..end].to_vec())
        .make_shared();
    result.set(v8::ArrayBuffer::with_backing_store(scope, &backing).into());
}

fn unmap(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<GpuBufferStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.map_state = "unmapped".to_owned();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuBufferStore>() {
        store.constructor.remove(realm_id);
    }
}
