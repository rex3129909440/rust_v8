use std::collections::HashMap;
#[derive(Clone)]
struct AccessRecord {
    key_system: String,
    configuration: v8::Global<v8::Object>,
}
#[derive(Default)]
pub(crate) struct MediaKeySystemAccessStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AccessRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaKeySystemAccessStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaKeySystemAccess", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MediaKeySystemAccessStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaKeySystemAccess",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "keySystem", get_key_system)?;
    crate::webidl::define_method(scope, prototype, "createMediaKeys", 0, create_media_keys)?;
    crate::webidl::define_method(scope, prototype, "getConfiguration", 0, get_configuration)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MediaKeySystemAccessStore>()
        .ok_or_else(|| "MediaKeySystemAccess state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    key_system: String,
    configuration: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(s)?;
    let prototype = crate::webidl::prototype(s, constructor)?;
    let object = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaKeySystemAccess".to_owned());
    }
    let record = AccessRecord {
        key_system,
        configuration: v8::Global::new(s, configuration),
    };
    s.get_slot_mut::<MediaKeySystemAccessStore>()
        .ok_or_else(|| "MediaKeySystemAccess state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<AccessRecord> {
    s.get_slot::<MediaKeySystemAccessStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_key_system(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(value) = v8::String::new(s, &v.key_system)
    {
        r.set(value.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_configuration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.configuration).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn create_media_keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "MediaKeySystemAccess",
            "createMediaKeys",
            r,
        );
        return;
    }
    if let Ok(keys) = super::media_keys::create(s)
        && let Ok(promise) = super::writable_stream::resolved_promise(s, keys.into())
    {
        r.set(promise.into())
    }
}
