use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct GpuPipelineErrorStore {
    constructor: crate::webidl::RealmConstructor,
    reasons: HashMap<i32, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuPipelineErrorStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUPipelineError", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuPipelineErrorStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUPipelineError",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reason", get_reason)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::dom_exception::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuPipelineErrorStore>()
        .ok_or_else(|| "GPUPipelineError state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Please use the 'new' operator");
        return;
    }
    let Some(message) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    if arguments.get(1).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'GPUPipelineError': The provided value is not of type 'GPUPipelineErrorInit'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let reason = options
        .and_then(|object| {
            crate::webidl::string(scope, "reason")
                .ok()
                .and_then(|key| object.get(scope, key.into()))
        })
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_else(|| "internal".to_owned());
    let object = arguments.this();
    super::dom_exception::attach(scope, object, "OperationError".to_owned(), message, 0);
    if let Some(store) = scope.get_slot_mut::<GpuPipelineErrorStore>() {
        store
            .reasons
            .insert(object.get_identity_hash().get(), reason);
    }
    result.set(object.into())
}
fn get_reason(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let reason = scope
        .get_slot::<GpuPipelineErrorStore>()
        .and_then(|store| {
            store
                .reasons
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(reason) = reason
        && let Some(value) = v8::String::new(scope, &reason)
    {
        result.set(value.into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuPipelineErrorStore>() {
        store.constructor.remove(realm_id);
    }
}
