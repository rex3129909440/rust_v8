use std::collections::HashMap;
#[derive(Clone)]
struct BucketData {
    name: String,
    expires: Option<f64>,
    directory: v8::Global<v8::Object>,
}
#[derive(Default)]
pub(crate) struct StorageBucketStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BucketData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(StorageBucketStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "StorageBucket", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<StorageBucketStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "StorageBucket",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "name", name)?;
    crate::webidl::define_readonly_accessor(s, p, "indexedDB", indexed_db)?;
    crate::webidl::define_readonly_accessor(s, p, "caches", caches)?;
    crate::webidl::define_method(s, p, "estimate", 0, estimate)?;
    crate::webidl::define_method(s, p, "expires", 0, expires)?;
    crate::webidl::define_method(s, p, "getDirectory", 0, get_directory)?;
    crate::webidl::define_method(s, p, "persisted", 0, persisted)?;
    crate::webidl::define_method(s, p, "setExpires", 1, set_expires)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let tag = v8::Symbol::get_to_string_tag(s);
    let _ = p.delete(s, tag.into());
    crate::webidl::define_method(s, p, "persist", 0, persist)?;
    crate::webidl::define_to_string_tag(s, p, "StorageBucket")?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<StorageBucketStore>()
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
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create StorageBucket".to_owned());
    }
    let directory_object = super::file_system_directory_handle::create(s, name.clone())?;
    let directory = v8::Global::new(s, directory_object);
    s.get_slot_mut::<StorageBucketStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            BucketData {
                name,
                expires: None,
                directory,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<BucketData> {
    s.get_slot::<StorageBucketStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
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
fn global_value(s: &mut v8::PinScope<'_, '_>, name: &str, mut r: v8::ReturnValue<'_>) {
    let g = s.get_current_context().global(s);
    if let Some(k) = v8::String::new(s, name)
        && let Some(v) = g.get(s, k.into())
    {
        r.set(v)
    }
}
fn indexed_db(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        global_value(s, "indexedDB", r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn caches(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        global_value(s, "caches", r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn estimate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucket", "estimate", r);
        return;
    }
    let o = v8::Object::new(s);
    if let Some(k) = v8::String::new(s, "usage") {
        let _ = o.set(s, k.into(), v8::Number::new(s, 0.0).into());
    }
    if let Some(k) = v8::String::new(s, "quota") {
        let _ = o.set(s, k.into(), v8::Number::new(s, 1_073_741_824.0).into());
    }
    promise(s, o.into(), r)
}
fn expires(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let x = if let Some(n) = v.expires {
            v8::Number::new(s, n).into()
        } else {
            v8::null(s).into()
        };
        promise(s, x, r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucket", "expires", r)
    }
}
fn get_directory(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let x = v8::Local::new(s, &v.directory);
        promise(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucket", "getDirectory", r)
    }
}
fn persisted(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        let x = v8::Boolean::new(s, true);
        promise(s, x.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucket", "persisted", r)
    }
}
fn persist(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        let value = v8::Boolean::new(s, true);
        promise(s, value.into(), r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucket", "persist", r)
    }
}
fn set_expires(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let identity = a.this().get_identity_hash().get();
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "StorageBucket", "setExpires", r);
        return;
    }
    let expires = if a.get(0).is_null() {
        None
    } else {
        a.get(0).number_value(s)
    };
    if let Some(v) = s
        .get_slot_mut::<StorageBucketStore>()
        .and_then(|x| x.records.get_mut(&identity))
    {
        v.expires = expires;
        let x = v8::undefined(s);
        promise(s, x.into(), r)
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<StorageBucketStore>() {
        store.constructor.remove(realm_id);
    }
}
