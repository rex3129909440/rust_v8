use std::collections::HashMap;

#[derive(Clone)]
struct LaunchParamsRecord {
    target_url: String,
    files: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct LaunchParamsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, LaunchParamsRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LaunchParamsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "LaunchParams", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = scope
        .get_slot::<LaunchParamsStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let c = crate::webidl::create_function(
        scope,
        "LaunchParams",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "targetURL", get_target_url)?;
    crate::webidl::define_readonly_accessor(scope, p, "files", get_files)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<LaunchParamsStore>()
        .ok_or_else(|| "LaunchParams state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target_url: String,
    files: v8::Local<'_, v8::Array>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let value = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, value, p.into()) != Some(true) {
        return Err("cannot create LaunchParams".to_owned());
    }
    let record = LaunchParamsRecord {
        target_url,
        files: v8::Global::new(scope, files),
    };
    scope
        .get_slot_mut::<LaunchParamsStore>()
        .ok_or_else(|| "LaunchParams state was not prepared".to_owned())?
        .records
        .insert(value.get_identity_hash().get(), record);
    Ok(value)
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<LaunchParamsRecord> {
    scope
        .get_slot::<LaunchParamsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'LaunchParams': Illegal constructor",
    )
}
fn get_target_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.target_url) {
        result.set(value.into())
    }
}
fn get_files(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.files).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
