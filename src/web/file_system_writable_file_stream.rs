use std::collections::HashMap;
#[derive(Clone)]
struct WritableFileRecord {
    bytes: Vec<u8>,
    position: usize,
    mode: String,
}
#[derive(Default)]
pub(crate) struct FileSystemWritableFileStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WritableFileRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(FileSystemWritableFileStreamStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "FileSystemWritableFileStream", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<FileSystemWritableFileStreamStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "FileSystemWritableFileStream",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "seek", 1, seek)?;
    crate::webidl::define_method(s, p, "truncate", 1, truncate)?;
    crate::webidl::define_method(s, p, "write", 1, write)?;
    crate::webidl::define_readonly_accessor(s, p, "mode", mode)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::writable_stream::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<FileSystemWritableFileStreamStore>()
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
    bytes: Vec<u8>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = super::writable_stream::create_empty(s)?;
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create FileSystemWritableFileStream".to_owned());
    }
    s.get_slot_mut::<FileSystemWritableFileStreamStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            WritableFileRecord {
                bytes,
                position: 0,
                mode: "exclusive".to_owned(),
            },
        );
    Ok(o)
}
fn resolve(s: &mut v8::PinScope<'_, '_>, mut r: v8::ReturnValue<'_>) {
    let x = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
        r.set(p.into())
    }
}
fn mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot::<FileSystemWritableFileStreamStore>()
        .and_then(|x| x.records.get(&a.this().get_identity_hash().get()))
        && let Some(x) = v8::String::new(s, &v.mode)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn seek(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let identity = a.this().get_identity_hash().get();
    if s.get_slot::<FileSystemWritableFileStreamStore>()
        .is_none_or(|x| !x.records.contains_key(&identity))
    {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemWritableFileStream",
            "seek",
            r,
        );
        return;
    }
    let position = a.get(0).integer_value(s).unwrap_or(0).max(0) as usize;
    if let Some(v) = s
        .get_slot_mut::<FileSystemWritableFileStreamStore>()
        .and_then(|x| x.records.get_mut(&identity))
    {
        v.position = position;
        resolve(s, r)
    }
}
fn truncate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let identity = a.this().get_identity_hash().get();
    if s.get_slot::<FileSystemWritableFileStreamStore>()
        .is_none_or(|x| !x.records.contains_key(&identity))
    {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemWritableFileStream",
            "truncate",
            r,
        );
        return;
    }
    let size = a.get(0).integer_value(s).unwrap_or(0).max(0) as usize;
    if let Some(v) = s
        .get_slot_mut::<FileSystemWritableFileStreamStore>()
        .and_then(|x| x.records.get_mut(&identity))
    {
        v.bytes.resize(size, 0);
        v.position = v.position.min(size);
        resolve(s, r)
    }
}
fn write(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let identity = a.this().get_identity_hash().get();
    if s.get_slot::<FileSystemWritableFileStreamStore>()
        .is_none_or(|x| !x.records.contains_key(&identity))
    {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemWritableFileStream",
            "write",
            r,
        );
        return;
    }
    let bytes = crate::webidl::value_to_string(s, a.get(0)).into_bytes();
    if let Some(v) = s
        .get_slot_mut::<FileSystemWritableFileStreamStore>()
        .and_then(|x| x.records.get_mut(&identity))
    {
        let end = v.position + bytes.len();
        if v.bytes.len() < end {
            v.bytes.resize(end, 0)
        }
        v.bytes[v.position..end].copy_from_slice(&bytes);
        v.position = end;
        resolve(s, r)
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileSystemWritableFileStreamStore>() {
        store.constructor.remove(realm_id);
    }
}
