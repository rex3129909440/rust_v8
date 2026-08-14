use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct FileHandleRecord {
    bytes: Arc<Mutex<Vec<u8>>>,
}
#[derive(Default)]
pub(crate) struct FileSystemFileHandleStore {
    constructor: crate::webidl::RealmConstructor,
    worker_constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, FileHandleRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(FileSystemFileHandleStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "FileSystemFileHandle", c.into())
}

pub(crate) fn install_in_worker_realm(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(c) = s
        .get_slot::<FileSystemFileHandleStore>()
        .and_then(|store| store.worker_constructors.get(&realm_id))
        .cloned()
    {
        return crate::webidl::define_global(
            s,
            "FileSystemFileHandle",
            v8::Local::new(s, &c).into(),
        );
    }
    let c = build_constructor(s, true)?;
    let saved_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<FileSystemFileHandleStore>()
        .ok_or_else(|| "FileSystemFileHandle state was not prepared".to_owned())?
        .worker_constructors
        .insert(realm_id, saved_constructor);
    crate::webidl::define_global(s, "FileSystemFileHandle", c.into())
}

fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<FileSystemFileHandleStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = build_constructor(s, false)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<FileSystemFileHandleStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}

fn build_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
    worker: bool,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let c = crate::webidl::create_function(
        s,
        "FileSystemFileHandle",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "createWritable", 0, create_writable)?;
    crate::webidl::define_method(s, p, "getFile", 0, get_file)?;
    crate::webidl::define_method(s, p, "move", 1, move_file)?;
    if worker {
        super::file_system_file_handle_create_sync_access_handle::define(s, p)?;
    }
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = if worker {
        current_constructor(s, "FileSystemHandle")?
    } else {
        super::file_system_handle::ensure_constructor(s)?
    };
    crate::webidl::inherit(s, c, parent)?;
    Ok(c)
}

fn current_constructor<'s>(
    s: &v8::PinScope<'s, '_>,
    name: &str,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let global = s.get_current_context().global(s);
    let key = crate::webidl::string(s, name)?;
    let value = global
        .get(s, key.into())
        .ok_or_else(|| format!("{name} is unavailable in this realm"))?;
    v8::Local::<v8::Function>::try_from(value).map_err(|_| format!("{name} is not a constructor"))
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
    let current_realm_id = crate::webidl::realm_id(s);
    let entered_realm_id = {
        let entered = {
            let context = s.get_entered_or_microtask_context();
            v8::Global::new(s, context)
        };
        let entered = v8::Local::new(s, &entered);
        let entered_scope = &mut v8::ContextScope::new(s, entered);
        crate::webidl::realm_id(entered_scope)
    };
    let worker_constructor = s.get_slot::<FileSystemFileHandleStore>().and_then(|store| {
        store
            .worker_constructors
            .get(&current_realm_id)
            .or_else(|| store.worker_constructors.get(&entered_realm_id))
            .cloned()
    });
    let c = if let Some(constructor) = worker_constructor {
        v8::Local::new(s, &constructor)
    } else {
        ensure(s)?
    };
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create FileSystemFileHandle".to_owned());
    }
    super::file_system_handle::attach(s, o, "file".to_owned(), name);
    s.get_slot_mut::<FileSystemFileHandleStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), FileHandleRecord::default());
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<FileHandleRecord> {
    s.get_slot::<FileSystemFileHandleStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}

pub(crate) fn shared_bytes(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Result<Arc<Mutex<Vec<u8>>>, String> {
    record(s, o)
        .map(|record| record.bytes)
        .ok_or_else(|| "Illegal invocation".to_owned())
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}

pub(crate) fn cleanup_worker_realm(s: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = s.get_slot_mut::<FileSystemFileHandleStore>() {
        store.worker_constructors.remove(&realm_id);
    }
}
fn create_writable(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemFileHandle",
            "createWritable",
            r,
        );
        return;
    };
    let bytes = v
        .bytes
        .lock()
        .map(|bytes| bytes.clone())
        .unwrap_or_default();
    match super::file_system_writable_file_stream::create(s, bytes) {
        Ok(x) => resolve(s, x.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn get_file(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::reject_illegal_invocation_promise(s, "FileSystemFileHandle", "getFile", r);
        return;
    };
    let name = super::file_system_handle::record(s, a.this())
        .map(|x| x.name)
        .unwrap_or_else(|| "file".to_owned());
    let bytes = v
        .bytes
        .lock()
        .map(|bytes| bytes.clone())
        .unwrap_or_default();
    match super::file::create(s, &name, bytes, "application/octet-stream", 0.0) {
        Ok(x) => resolve(s, x.into(), r),
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}
fn move_file(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(s, "FileSystemFileHandle", "move", r);
        return;
    }
    let new_name = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<super::file_system_handle::FileSystemHandleStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.name = new_name;
        let x = v8::undefined(s);
        resolve(s, x.into(), r)
    }
}
