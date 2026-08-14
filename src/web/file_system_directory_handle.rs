use std::collections::HashMap;
#[derive(Clone, Default)]
struct DirectoryRecord {
    directories: Vec<String>,
    files: Vec<String>,
}
#[derive(Default)]
pub(crate) struct FileSystemDirectoryHandleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DirectoryRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(FileSystemDirectoryHandleStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "FileSystemDirectoryHandle", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<FileSystemDirectoryHandleStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "FileSystemDirectoryHandle",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "getDirectoryHandle", 1, get_directory_handle)?;
    crate::webidl::define_method(s, p, "getFileHandle", 1, get_file_handle)?;
    crate::webidl::define_method(s, p, "removeEntry", 1, remove_entry)?;
    crate::webidl::define_method(s, p, "resolve", 1, resolve_path)?;
    crate::webidl::define_method(s, p, "entries", 0, entries)?;
    crate::webidl::define_method(s, p, "keys", 0, keys)?;
    crate::webidl::define_method(s, p, "values", 0, values)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_async_iterator_alias(s, p, "entries")?;
    let parent = super::file_system_handle::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<FileSystemDirectoryHandleStore>()
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
        return Err("cannot create FileSystemDirectoryHandle".to_owned());
    }
    super::file_system_handle::attach(s, o, "directory".to_owned(), name);
    s.get_slot_mut::<FileSystemDirectoryHandleStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), DirectoryRecord::default());
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<DirectoryRecord> {
    s.get_slot::<FileSystemDirectoryHandleStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn get_directory_handle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemDirectoryHandle",
            "getDirectoryHandle",
            r,
        );
        return;
    }
    let name = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<FileSystemDirectoryHandleStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        if !v.directories.contains(&name) {
            v.directories.push(name.clone())
        }
        match create(s, name) {
            Ok(x) => resolve(s, x.into(), r),
            Err(e) => crate::webidl::throw_type_error(s, &e),
        }
    }
}
fn get_file_handle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemDirectoryHandle",
            "getFileHandle",
            r,
        );
        return;
    }
    let name = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<FileSystemDirectoryHandleStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        if !v.files.contains(&name) {
            v.files.push(name.clone())
        }
        match super::file_system_file_handle::create(s, name) {
            Ok(x) => resolve(s, x.into(), r),
            Err(e) => crate::webidl::throw_type_error(s, &e),
        }
    }
}
fn remove_entry(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemDirectoryHandle",
            "removeEntry",
            r,
        );
        return;
    }
    let name = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<FileSystemDirectoryHandleStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.directories.retain(|x| x != &name);
        v.files.retain(|x| x != &name);
        let x = v8::undefined(s);
        resolve(s, x.into(), r)
    }
}
fn resolve_path(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "FileSystemDirectoryHandle",
            "resolve",
            r,
        );
        return;
    }
    let array = v8::Array::new(s, 0);
    resolve(s, array.into(), r)
}
fn names_array<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    pairs: bool,
    handles: bool,
) -> Option<v8::Local<'s, v8::Array>> {
    let v = record(s, a.this())?;
    let length = v.directories.len() + v.files.len();
    let array = v8::Array::new(s, length as i32);
    let mut index = 0;
    for name in v.directories.iter().chain(v.files.iter()) {
        let value = v8::String::new(s, name)?;
        if pairs {
            let pair = v8::Array::new(s, 2);
            let _ = pair.set_index(s, 0, value.into());
            let _ = pair.set_index(s, 1, value.into());
            let _ = array.set_index(s, index, pair.into());
        } else if handles {
            let _ = array.set_index(s, index, value.into());
        } else {
            let _ = array.set_index(s, index, value.into());
        }
        index += 1;
    }
    Some(array)
}
fn entries<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    mut r: v8::ReturnValue<'s>,
) {
    if let Some(v) = names_array(s, a, true, false) {
        r.set(v.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn keys<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    mut r: v8::ReturnValue<'s>,
) {
    if let Some(v) = names_array(s, a, false, false) {
        r.set(v.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn values<'s>(
    s: &mut v8::PinScope<'s, '_>,
    a: v8::FunctionCallbackArguments<'s>,
    mut r: v8::ReturnValue<'s>,
) {
    if let Some(v) = names_array(s, a, false, true) {
        r.set(v.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileSystemDirectoryHandleStore>() {
        store.constructor.remove(realm_id);
    }
}
