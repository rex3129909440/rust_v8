use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct GpuCompilationInfoStore {
    constructor: crate::webidl::RealmConstructor,
    messages: HashMap<i32, v8::Global<v8::Array>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuCompilationInfoStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUCompilationInfo", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<GpuCompilationInfoStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUCompilationInfo",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "messages", get_messages)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuCompilationInfoStore>()
        .ok_or_else(|| "GPUCompilationInfo state was not prepared".to_owned())?
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
    messages: Vec<super::gpu_compilation_message::CompilationMessageRecord>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPUCompilationInfo".to_owned());
    }
    let array = v8::Array::new(scope, messages.len() as i32);
    for (index, message) in messages.into_iter().enumerate() {
        let item = super::gpu_compilation_message::create(scope, message)?;
        let _ = array.set_index(scope, index as u32, item.into());
    }
    let persistent = v8::Global::new(scope, array);
    scope
        .get_slot_mut::<GpuCompilationInfoStore>()
        .ok_or_else(|| "GPUCompilationInfo state was not prepared".to_owned())?
        .messages
        .insert(object.get_identity_hash().get(), persistent);
    Ok(object)
}

fn get_messages(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let messages = scope
        .get_slot::<GpuCompilationInfoStore>()
        .and_then(|store| {
            store
                .messages
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(messages) = messages {
        result.set(v8::Local::new(scope, &messages).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuCompilationInfoStore>() {
        store.constructor.remove(realm_id);
    }
}
