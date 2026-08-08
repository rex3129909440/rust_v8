use std::collections::HashMap;

#[derive(Clone)]
struct LostInfoRecord {
    reason: String,
    message: String,
}

#[derive(Default)]
pub(crate) struct GpuDeviceLostInfoStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, LostInfoRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuDeviceLostInfoStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUDeviceLostInfo", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<GpuDeviceLostInfoStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUDeviceLostInfo",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reason", get_reason)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "message", get_message)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuDeviceLostInfoStore>()
        .ok_or_else(|| "GPUDeviceLostInfo state was not prepared".to_owned())?
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
    reason: String,
    message: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPUDeviceLostInfo".to_owned());
    }
    scope
        .get_slot_mut::<GpuDeviceLostInfoStore>()
        .ok_or_else(|| "GPUDeviceLostInfo state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            LostInfoRecord { reason, message },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<LostInfoRecord> {
    scope
        .get_slot::<GpuDeviceLostInfoStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(LostInfoRecord) -> String,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &select(record)) {
        result.set(value.into());
    }
}

fn get_reason(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |v| v.reason);
}
fn get_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |v| v.message);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuDeviceLostInfoStore>() {
        store.constructor.remove(realm_id);
    }
}
