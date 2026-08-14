use std::collections::HashMap;

#[derive(Clone)]
struct FileRecord {
    name: String,
    last_modified: f64,
    webkit_relative_path: String,
}

#[derive(Default)]
pub(crate) struct FileStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, FileRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FileStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "File", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FileStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "File",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::blob::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lastModified", get_last_modified)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "lastModifiedDate",
        get_last_modified_date,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "webkitRelativePath",
        get_webkit_relative_path,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FileStore>()
        .ok_or_else(|| "File state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "Failed to construct 'File': 2 arguments required");
        return;
    }
    if !arguments.get(0).is_object() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'File': The provided value cannot be converted to a sequence.",
        );
        return;
    }
    let parts = match crate::webidl::sequence_values(scope, arguments.get(0)) {
        Ok(parts) => parts,
        Err(_) => {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'File': The object must have a callable @@iterator property.",
            );
            return;
        }
    };
    let mut bytes = Vec::new();
    for part in parts {
        append_part(scope, v8::Local::new(scope, &part), &mut bytes);
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(1));
    let options = v8::Local::<v8::Object>::try_from(arguments.get(2)).ok();
    let media_type = options
        .and_then(|object| property(scope, object, "type"))
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value).to_ascii_lowercase())
        .unwrap_or_default();
    let last_modified = options
        .and_then(|object| property(scope, object, "lastModified"))
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .map(webidl_long_long)
        .unwrap_or_else(|| crate::determinism::date_epoch_milliseconds(scope));
    attach(
        scope,
        arguments.this(),
        bytes,
        media_type,
        FileRecord {
            name,
            last_modified,
            webkit_relative_path: String::new(),
        },
    );
    result.set(arguments.this().into());
}

fn append_part(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    output: &mut Vec<u8>,
) {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value)
        && let Some((bytes, _)) = super::blob::byte_snapshot(scope, object)
    {
        output.extend_from_slice(&bytes);
        return;
    }
    if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
        let store = buffer.get_backing_store();
        if let Some(data) = store.data() {
            let bytes = unsafe {
                std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), store.byte_length())
            };
            output.extend_from_slice(bytes);
        }
        return;
    }
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let mut bytes = vec![0_u8; view.byte_length()];
        let copied = view.copy_contents(&mut bytes);
        output.extend_from_slice(&bytes[..copied]);
        return;
    }
    output.extend_from_slice(crate::webidl::value_to_string(scope, value).as_bytes());
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    bytes: Vec<u8>,
    media_type: String,
    record: FileRecord,
) {
    super::blob::attach(scope, object, bytes, media_type);
    if let Some(store) = scope.get_slot_mut::<FileStore>() {
        store
            .records
            .insert(object.get_identity_hash().get(), record);
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    bytes: Vec<u8>,
    media_type: &str,
    last_modified: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let file = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, file, prototype.into()) != Some(true) {
        return Err("cannot create File".to_owned());
    }
    attach(
        scope,
        file,
        bytes,
        media_type.to_ascii_lowercase(),
        FileRecord {
            name: name.to_owned(),
            last_modified: webidl_long_long(last_modified),
            webkit_relative_path: String::new(),
        },
    );
    Ok(file)
}

fn webidl_long_long(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return 0.0;
    }
    const TWO_TO_63: f64 = 9_223_372_036_854_775_808.0;
    const TWO_TO_64: f64 = 18_446_744_073_709_551_616.0;
    let value = value.trunc();
    if (-TWO_TO_63..TWO_TO_63).contains(&value) {
        return value;
    }
    let value = value.rem_euclid(TWO_TO_64);
    if value >= TWO_TO_63 {
        value - TWO_TO_64
    } else {
        value
    }
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<FileRecord> {
    scope
        .get_slot::<FileStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.name) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_last_modified(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.last_modified).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_last_modified_date(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::Date::new(scope, record.last_modified) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_webkit_relative_path(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.webkit_relative_path) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileStore>() {
        store.constructors.remove(&realm_id);
    }
}
