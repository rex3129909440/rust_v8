#[derive(Default)]
pub(crate) struct MediaKeysStore {
    constructor: crate::webidl::RealmConstructor,
    instances: std::collections::HashSet<i32>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaKeysStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaKeys", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MediaKeysStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaKeys",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "createSession", 0, create_session)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setServerCertificate",
        1,
        set_server_certificate,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getStatusForPolicy",
        0,
        get_status_for_policy,
    )?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MediaKeysStore>()
        .ok_or_else(|| "MediaKeys state was not prepared".to_owned())?
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
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(s)?;
    let prototype = crate::webidl::prototype(s, constructor)?;
    let object = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaKeys".to_owned());
    }
    s.get_slot_mut::<MediaKeysStore>()
        .ok_or_else(|| "MediaKeys state was not prepared".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<MediaKeysStore>()
        .is_some_and(|store| store.instances.contains(&o.get_identity_hash().get()))
}
fn create_session(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    match super::media_key_session::create(s) {
        Ok(value) => r.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
fn resolve(
    s: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(s, value) {
        r.set(promise.into())
    }
}
fn set_server_certificate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        resolve(s, v8::Boolean::new(s, true).into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_status_for_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        let value = v8::String::new(s, "usable").unwrap();
        resolve(s, value.into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
