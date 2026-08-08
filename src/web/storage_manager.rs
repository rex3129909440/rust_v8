#[derive(Default)]
pub(crate) struct StorageManagerStore {
    constructor: crate::webidl::RealmConstructor,
    instances: std::collections::HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(StorageManagerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "StorageManager", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<StorageManagerStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "StorageManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "estimate", 0, estimate)?;
    crate::webidl::define_method(s, p, "persisted", 0, persisted)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_method(s, p, "getDirectory", 0, get_directory)?;
    crate::webidl::define_method(s, p, "persist", 0, persist)?;
    let g = v8::Global::new(s, c);
    s.get_slot_mut::<StorageManagerStore>()
        .ok_or_else(|| "StorageManager state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
    Ok(c)
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
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create StorageManager".to_owned());
    }
    s.get_slot_mut::<StorageManagerStore>()
        .unwrap()
        .instances
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<StorageManagerStore>()
        .is_some_and(|x| x.instances.contains(&o.get_identity_hash().get()))
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn estimate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let o = v8::Object::new(s);
    let usage = v8::String::new(s, "usage").unwrap();
    let quota = v8::String::new(s, "quota").unwrap();
    let storage = crate::fingerprint::edge(s).storage.clone();
    let _ = o.set(
        s,
        usage.into(),
        v8::Number::new(s, storage.usage_bytes as f64).into(),
    );
    let _ = o.set(
        s,
        quota.into(),
        v8::Number::new(s, storage.quota_bytes as f64).into(),
    );
    resolve(s, o.into(), r)
}
fn persisted(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        let persisted = crate::fingerprint::edge(s).storage.persisted;
        resolve(s, v8::Boolean::new(s, persisted).into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn persist(
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
fn get_directory(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if valid(s, a.this()) {
        match super::file_system_directory_handle::create(s, String::new()) {
            Ok(directory) => resolve(s, directory.into(), r),
            Err(message) => crate::webidl::throw_type_error(s, &message),
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<StorageManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
