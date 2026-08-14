use std::collections::HashMap;

#[derive(Clone)]
pub(crate) enum ItemPayload {
    String(String),
    File(v8::Global<v8::Object>),
}

#[derive(Clone)]
pub(crate) struct ItemRecord {
    pub media_type: String,
    pub payload: ItemPayload,
    pub(crate) created_via_set_data: bool,
}

#[derive(Default)]
pub(crate) struct DataTransferItemStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ItemRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DataTransferItemStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DataTransferItem", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DataTransferItemStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DataTransferItem",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "kind", get_kind)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_method(scope, prototype, "getAsFile", 0, get_as_file)?;
    crate::webidl::define_method(scope, prototype, "getAsString", 1, get_as_string)?;
    crate::webidl::define_method(scope, prototype, "webkitGetAsEntry", 0, webkit_get_as_entry)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getAsFileSystemHandle",
        0,
        get_as_file_system_handle,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DataTransferItemStore>()
        .ok_or_else(|| "DataTransferItem state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_string<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: String,
    media_type: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create(
        scope,
        ItemRecord {
            media_type: media_type.to_ascii_lowercase(),
            payload: ItemPayload::String(value),
            created_via_set_data: false,
        },
    )
}

pub(crate) fn create_string_from_set_data<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: String,
    media_type: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create(
        scope,
        ItemRecord {
            media_type: media_type.to_ascii_lowercase(),
            payload: ItemPayload::String(value),
            created_via_set_data: true,
        },
    )
}

pub(crate) fn create_file<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    file: v8::Local<'_, v8::Object>,
    media_type: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create(
        scope,
        ItemRecord {
            media_type: media_type.to_ascii_lowercase(),
            payload: ItemPayload::File(v8::Global::new(scope, file)),
            created_via_set_data: false,
        },
    )
}

fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: ItemRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let item = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, item, prototype.into()) != Some(true) {
        return Err("cannot create DataTransferItem".to_owned());
    }
    scope
        .get_slot_mut::<DataTransferItemStore>()
        .ok_or_else(|| "DataTransferItem state was not prepared".to_owned())?
        .records
        .insert(item.get_identity_hash().get(), record);
    Ok(item)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ItemRecord> {
    scope
        .get_slot::<DataTransferItemStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn get_kind(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let kind = match record.payload {
        ItemPayload::String(_) => "string",
        ItemPayload::File(_) => "file",
    };
    if let Some(kind) = v8::String::new(scope, kind) {
        result.set(kind.into());
    }
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(media_type) = v8::String::new(scope, &record.media_type) {
        result.set(media_type.into());
    }
}

fn get_as_file(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(ItemRecord {
            payload: ItemPayload::File(file),
            ..
        }) => result.set(v8::Local::new(scope, &file).into()),
        Some(_) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_as_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let ItemPayload::String(text) = record.payload else {
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        return;
    };
    let data = v8::Object::new(scope);
    define_data(scope, data, "callback", callback.into());
    if let Some(text) = v8::String::new(scope, &text) {
        define_data(scope, data, "text", text.into());
    }
    if let Some(task) = v8::Function::builder(run_string_callback)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    {
        scope.enqueue_microtask(task);
    }
}

fn run_string_callback(
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
    let Some(text_key) = v8::String::new(scope, "text") else {
        return;
    };
    let Some(callback) = data
        .get(scope, callback_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    let text = data
        .get(scope, text_key.into())
        .unwrap_or_else(|| v8::undefined(scope).into());
    let receiver = v8::undefined(scope);
    let _ = callback.call(scope, receiver.into(), &[text]);
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

fn webkit_get_as_entry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_as_file_system_handle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "DataTransferItem",
            "getAsFileSystemHandle",
            result,
        );
        return;
    }
    let null = v8::null(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, null.into()) {
        result.set(promise.into());
    }
}
