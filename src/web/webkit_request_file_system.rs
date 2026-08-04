use std::collections::HashMap;

#[derive(Clone)]
struct EntryRecord {
    name: String,
    full_path: String,
    url: String,
    is_file: bool,
    filesystem: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct WebkitFileSystemStore {
    filesystems: HashMap<(String, i32), v8::Global<v8::Object>>,
    entries: HashMap<i32, EntryRecord>,
    entries_by_url: HashMap<String, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WebkitFileSystemStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "webkitRequestFileSystem",
        3,
        v8::ConstructorBehavior::Throw,
        execute,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "webkitRequestFileSystem")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.webkitRequestFileSystem".to_owned())
    }
}

pub(crate) fn execute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(2)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'webkitRequestFileSystem': the success callback is required.",
        );
        return;
    };
    let error_callback = v8::Local::<v8::Function>::try_from(arguments.get(3)).ok();
    let Some(origin) = secure_origin(scope) else {
        if let Some(error_callback) = error_callback
            && let Ok(error) = security_error(scope)
        {
            schedule_callback(scope, error_callback, error.into());
        }
        return;
    };
    let file_system_type = arguments.get(0).int32_value(scope).unwrap_or(0);
    if file_system_type != 0 && file_system_type != 1 {
        if let Some(error_callback) = error_callback
            && let Ok(error) = super::dom_exception::create(
                scope,
                "The requested file system type is not supported.".to_owned(),
                "NotSupportedError".to_owned(),
            )
        {
            schedule_callback(scope, error_callback, error.into());
        }
        return;
    }
    match get_or_create_file_system(scope, &origin, file_system_type) {
        Ok(file_system) => schedule_callback(scope, success_callback, file_system.into()),
        Err(message) => {
            if let Some(error_callback) = error_callback
                && let Ok(error) =
                    super::dom_exception::create(scope, message, "InvalidStateError".to_owned())
            {
                schedule_callback(scope, error_callback, error.into());
            }
        }
    }
}

pub(crate) fn secure_origin(scope: &mut v8::PinScope<'_, '_>) -> Option<url::Url> {
    if let Some(origin) = super::worker_global_scope::current_origin(scope) {
        let origin = url::Url::parse(&format!("{origin}/")).ok()?;
        return matches!(origin.scheme(), "http" | "https").then_some(origin);
    }
    let global = scope.get_current_context().global(scope);
    let location_key = v8::String::new(scope, "location")?;
    let location = global.get(scope, location_key.into())?;
    let location = v8::Local::<v8::Object>::try_from(location).ok()?;
    let href_key = v8::String::new(scope, "href")?;
    let href = location.get(scope, href_key.into())?;
    let href = crate::webidl::value_to_string(scope, href);
    let origin = url::Url::parse(&href).ok()?;
    match origin.scheme() {
        "http" | "https" => Some(origin),
        _ => None,
    }
}

pub(crate) fn get_or_create_file_system<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    origin: &url::Url,
    file_system_type: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let origin_text = origin.origin().ascii_serialization();
    let key = (origin_text.clone(), file_system_type);
    if let Some(existing) = scope
        .get_slot::<WebkitFileSystemStore>()
        .and_then(|store| store.filesystems.get(&key))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &existing));
    }

    let file_system = v8::Object::new(scope);
    let root = v8::Object::new(scope);
    let storage_name = if file_system_type == 0 {
        "Temporary"
    } else {
        "Persistent"
    };
    let host = origin.host_str().unwrap_or_default();
    let name = format!(
        "{}_{}_{}:{storage_name}",
        origin.scheme(),
        host,
        origin.port().unwrap_or(0)
    );
    let storage_path = if file_system_type == 0 {
        "temporary"
    } else {
        "persistent"
    };
    let root_url = format!("filesystem:{origin_text}/{storage_path}/");

    define_readonly_text(scope, file_system, "name", &name)?;
    define_readonly_value(scope, file_system, "root", root.into())?;
    define_tag(scope, file_system, "DOMFileSystem")?;

    let file_system_global = v8::Global::new(scope, file_system);
    define_readonly_value(scope, root, "filesystem", file_system.into())?;
    define_readonly_value(scope, root, "isFile", v8::Boolean::new(scope, false).into())?;
    define_readonly_value(
        scope,
        root,
        "isDirectory",
        v8::Boolean::new(scope, true).into(),
    )?;
    define_readonly_text(scope, root, "name", "")?;
    define_readonly_text(scope, root, "fullPath", "/")?;
    define_entry_methods(scope, root, true)?;
    define_tag(scope, root, "DirectoryEntry")?;

    let root_record = EntryRecord {
        name: String::new(),
        full_path: "/".to_owned(),
        url: root_url.clone(),
        is_file: false,
        filesystem: file_system_global.clone(),
    };
    let root_global = v8::Global::new(scope, root);
    let store = scope
        .get_slot_mut::<WebkitFileSystemStore>()
        .ok_or_else(|| "webkit file system state was not prepared".to_owned())?;
    store.filesystems.insert(key, file_system_global);
    store
        .entries
        .insert(root.get_identity_hash().get(), root_record);
    store.entries_by_url.insert(root_url, root_global);
    Ok(file_system)
}

fn define_entry_methods(
    scope: &mut v8::PinScope<'_, '_>,
    entry: v8::Local<'_, v8::Object>,
    is_directory: bool,
) -> Result<(), String> {
    define_method(scope, entry, "toURL", 0, entry_to_url)?;
    define_method(scope, entry, "getParent", 2, get_parent)?;
    define_method(scope, entry, "getMetadata", 2, get_metadata)?;
    define_method(scope, entry, "remove", 2, remove_entry)?;
    if is_directory {
        define_method(scope, entry, "getFile", 3, get_file)?;
        define_method(scope, entry, "getDirectory", 3, get_directory)?;
        define_method(scope, entry, "createReader", 0, create_reader)?;
        define_method(scope, entry, "removeRecursively", 2, remove_entry)?;
    } else {
        define_method(scope, entry, "file", 2, read_file)?;
    }
    Ok(())
}

fn entry_record(
    scope: &v8::PinScope<'_, '_>,
    entry: v8::Local<'_, v8::Object>,
) -> Option<EntryRecord> {
    scope
        .get_slot::<WebkitFileSystemStore>()?
        .entries
        .get(&entry.get_identity_hash().get())
        .cloned()
}

fn entry_to_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = entry_record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(url) = v8::String::new(scope, &record.url) {
        result.set(url.into());
    }
}

fn get_parent(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = entry_record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        return;
    };
    let parent_url = if record.full_path == "/" {
        record.url
    } else {
        let mut parent = record.url.trim_end_matches('/').to_owned();
        if let Some(index) = parent.rfind('/') {
            parent.truncate(index + 1);
        }
        parent
    };
    if let Some(parent) = resolve_entry(scope, &parent_url) {
        schedule_callback(scope, success_callback, parent.into());
    }
}

fn get_metadata(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if entry_record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        return;
    };
    let metadata = v8::Object::new(scope);
    let _ = define_readonly_value(scope, metadata, "size", v8::Number::new(scope, 0.0).into());
    schedule_callback(scope, success_callback, metadata.into());
}

fn get_file(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    create_child(scope, &arguments, true);
}

fn get_directory(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    create_child(scope, &arguments, false);
}

fn create_child(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    is_file: bool,
) {
    let Some(parent) = entry_record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if parent.is_file {
        crate::webidl::throw_type_error(scope, "A FileEntry cannot contain child entries");
        return;
    }
    let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(2)) else {
        return;
    };
    let requested = crate::webidl::value_to_string(scope, arguments.get(0));
    let name = requested
        .trim_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_owned();
    if name.is_empty() || name == "." || name == ".." {
        if let Ok(error_callback) = v8::Local::<v8::Function>::try_from(arguments.get(3))
            && let Ok(error) = super::dom_exception::create(
                scope,
                "The supplied path is invalid.".to_owned(),
                "EncodingError".to_owned(),
            )
        {
            schedule_callback(scope, error_callback, error.into());
        }
        return;
    }
    let mut full_path = parent.full_path.trim_end_matches('/').to_owned();
    full_path.push('/');
    full_path.push_str(requested.trim_matches('/'));
    if !is_file {
        full_path.push('/');
    }
    let mut url = parent.url.trim_end_matches('/').to_owned();
    url.push('/');
    url.push_str(requested.trim_matches('/'));
    if !is_file {
        url.push('/');
    }
    if let Some(existing) = resolve_entry(scope, &url) {
        schedule_callback(scope, success_callback, existing.into());
        return;
    }

    let child = v8::Object::new(scope);
    let filesystem = v8::Local::new(scope, &parent.filesystem);
    let _ = define_readonly_value(scope, child, "filesystem", filesystem.into());
    let _ = define_readonly_value(
        scope,
        child,
        "isFile",
        v8::Boolean::new(scope, is_file).into(),
    );
    let _ = define_readonly_value(
        scope,
        child,
        "isDirectory",
        v8::Boolean::new(scope, !is_file).into(),
    );
    let _ = define_readonly_text(scope, child, "name", &name);
    let _ = define_readonly_text(scope, child, "fullPath", &full_path);
    let _ = define_entry_methods(scope, child, !is_file);
    let _ = define_tag(
        scope,
        child,
        if is_file {
            "FileEntry"
        } else {
            "DirectoryEntry"
        },
    );
    let record = EntryRecord {
        name,
        full_path,
        url: url.clone(),
        is_file,
        filesystem: parent.filesystem,
    };
    let child_global = v8::Global::new(scope, child);
    if let Some(store) = scope.get_slot_mut::<WebkitFileSystemStore>() {
        store
            .entries
            .insert(child.get_identity_hash().get(), record);
        store.entries_by_url.insert(url, child_global);
    }
    schedule_callback(scope, success_callback, child.into());
}

fn create_reader(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if entry_record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let reader = v8::Object::new(scope);
    if define_method(scope, reader, "readEntries", 2, read_entries).is_ok() {
        let _ = define_tag(scope, reader, "DirectoryReader");
        result.set(reader.into());
    }
}

fn read_entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) {
        let entries = v8::Array::new(scope, 0);
        schedule_callback(scope, success_callback, entries.into());
    }
}

fn remove_entry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = entry_record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.full_path == "/" {
        if let Ok(error_callback) = v8::Local::<v8::Function>::try_from(arguments.get(1))
            && let Ok(error) = super::dom_exception::create(
                scope,
                "The root directory cannot be removed.".to_owned(),
                "NoModificationAllowedError".to_owned(),
            )
        {
            schedule_callback(scope, error_callback, error.into());
        }
        return;
    }
    if let Some(store) = scope.get_slot_mut::<WebkitFileSystemStore>() {
        store
            .entries
            .remove(&arguments.this().get_identity_hash().get());
        store.entries_by_url.remove(&record.url);
    }
    if let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) {
        schedule_callback(scope, success_callback, v8::undefined(scope).into());
    }
}

fn read_file(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = entry_record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(success_callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        return;
    };
    let Ok(file) = super::file::create(scope, &record.name, Vec::new(), "", 0.0) else {
        return;
    };
    schedule_callback(scope, success_callback, file.into());
}

pub(crate) fn resolve_entry<'s>(
    scope: &v8::PinScope<'s, '_>,
    input: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let store = scope.get_slot::<WebkitFileSystemStore>()?;
    if let Some(entry) = store.entries_by_url.get(input) {
        return Some(v8::Local::new(scope, entry));
    }
    if input.ends_with('/') {
        None
    } else {
        let alternate = format!("{input}/");
        store
            .entries_by_url
            .get(&alternate)
            .map(|entry| v8::Local::new(scope, entry))
    }
}

pub(crate) fn security_error<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    super::dom_exception::create(
        scope,
        "It is unsafe to access the file system from this context.".to_owned(),
        "SecurityError".to_owned(),
    )
}

pub(crate) fn schedule_callback(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
    value: v8::Local<'_, v8::Value>,
) {
    let data = v8::Object::new(scope);
    let Some(callback_key) = v8::String::new(scope, "callback") else {
        return;
    };
    let Some(value_key) = v8::String::new(scope, "value") else {
        return;
    };
    let _ = data.set(scope, callback_key.into(), callback.into());
    let _ = data.set(scope, value_key.into(), value);
    if let Some(task) = v8::Function::builder(deliver_callback)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        scope.enqueue_microtask(task);
    }
}

fn deliver_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(data) = v8::Local::<v8::Object>::try_from(arguments.data()) else {
        return;
    };
    let Some(callback_key) = v8::String::new(scope, "callback") else {
        return;
    };
    let Some(value_key) = v8::String::new(scope, "value") else {
        return;
    };
    let Some(callback) = data
        .get(scope, callback_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    let Some(value) = data.get(scope, value_key.into()) else {
        return;
    };
    let receiver = v8::undefined(scope);
    let _ = callback.call(scope, receiver.into(), &[value]);
}

fn define_method(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    length: i32,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        name,
        length,
        v8::ConstructorBehavior::Throw,
        callback,
    )?;
    let key = crate::webidl::string(scope, name)?;
    if object.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define file system method {name}"))
    }
}

fn define_readonly_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    text: &str,
) -> Result<(), String> {
    let value = crate::webidl::string(scope, text)?;
    define_readonly_value(scope, object, name, value.into())
}

fn define_readonly_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if object.define_own_property(
        scope,
        key.into(),
        value,
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define file system property {name}"))
    }
}

fn define_tag(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: &str,
) -> Result<(), String> {
    let tag = v8::Symbol::get_to_string_tag(scope);
    let value = crate::webidl::string(scope, value)?;
    if object.define_own_property(
        scope,
        tag.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define file system object tag".to_owned())
    }
}
