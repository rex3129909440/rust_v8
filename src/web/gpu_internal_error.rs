#[derive(Default)]
pub(crate) struct GpuInternalErrorStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuInternalErrorStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUInternalError", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuInternalErrorStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUInternalError",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::gpu_error::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::define_to_string_tag(scope, prototype, "GPUInternalError")?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuInternalErrorStore>()
        .ok_or_else(|| "GPUInternalError state was not prepared".to_owned())?
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
    let message = crate::webidl::value_to_string(scope, arguments.get(0));
    let object = arguments.this();
    super::gpu_error::attach(scope, object, message);
    result.set(object.into())
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuInternalErrorStore>() {
        store.constructor.remove(realm_id);
    }
}
