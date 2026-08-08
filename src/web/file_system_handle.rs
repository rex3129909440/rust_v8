use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct HandleRecord {
    pub kind: String,
    pub name: String,
    pub removed: bool,
}

#[derive(Default)]
pub(crate) struct FileSystemHandleStore {
    constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, HandleRecord>,
}

pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(FileSystemHandleStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "FileSystemHandle", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<FileSystemHandleStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "FileSystemHandle",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "kind", kind)?;
    crate::webidl::define_readonly_accessor(s, p, "name", name)?;
    crate::webidl::define_method(s, p, "isSameEntry", 1, is_same_entry)?;
    crate::webidl::define_method(s, p, "queryPermission", 0, query_permission)?;
    crate::webidl::define_method(s, p, "remove", 0, remove)?;
    crate::webidl::define_method(s, p, "requestPermission", 0, request_permission)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<FileSystemHandleStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn attach(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    kind: String,
    name: String,
) {
    s.get_slot_mut::<FileSystemHandleStore>()
        .expect("FileSystemHandle state")
        .records
        .insert(
            o.get_identity_hash().get(),
            HandleRecord {
                kind,
                name,
                removed: false,
            },
        );
}
pub(crate) fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<HandleRecord> {
    s.get_slot::<FileSystemHandleStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn kind(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.kind)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.name)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn is_same_entry(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let same = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .and_then(|o| record(s, o))
        .zip(record(s, a.this()))
        .is_some_and(|(x, y)| x.kind == y.kind && x.name == y.name);
    let v = v8::Boolean::new(s, same);
    resolve(s, v.into(), r)
}
fn permission(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let v = v8::String::new(s, "granted").unwrap();
    resolve(s, v.into(), r)
}
fn query_permission(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    permission(s, a, r)
}
fn request_permission(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    permission(s, a, r)
}
fn remove(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<FileSystemHandleStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.removed = true;
        let x = v8::undefined(s);
        resolve(s, x.into(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileSystemHandleStore>() {
        store.constructor.remove(realm_id);
    }
}
