use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdbDatabaseStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbDatabaseRecord>,
}

#[derive(Clone)]
struct IdbDatabaseRecord {
    name: String,
    version: u64,
    closed: bool,
    version_change: Option<v8::Global<v8::Object>>,
    onabort: Option<v8::Global<v8::Function>>,
    onclose: Option<v8::Global<v8::Function>>,
    onerror: Option<v8::Global<v8::Function>>,
    onversionchange: Option<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbDatabaseStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBDatabase", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbDatabaseStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBDatabase",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "version", get_version)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "objectStoreNames",
        get_object_store_names,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onabort", get_onabort, set_onabort)?;
    crate::webidl::define_accessor(scope, prototype, "onclose", get_onclose, set_onclose)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onversionchange",
        get_onversionchange,
        set_onversionchange,
    )?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createObjectStore",
        1,
        create_object_store,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "deleteObjectStore",
        1,
        delete_object_store,
    )?;
    crate::webidl::define_method(scope, prototype, "transaction", 1, transaction)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbDatabaseStore>()
        .ok_or_else(|| "IDBDatabase state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    version: u64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBDatabase".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<IdbDatabaseStore>()
        .ok_or_else(|| "IDBDatabase state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdbDatabaseRecord {
                name,
                version,
                closed: false,
                version_change: None,
                onabort: None,
                onclose: None,
                onerror: None,
                onversionchange: None,
            },
        );
    Ok(object)
}

pub(crate) fn set_version_change(
    scope: &mut v8::PinScope<'_, '_>,
    database: v8::Local<'_, v8::Object>,
    transaction: v8::Local<'_, v8::Object>,
) {
    let transaction = v8::Global::new(scope, transaction);
    if let Some(record) = scope
        .get_slot_mut::<IdbDatabaseStore>()
        .and_then(|store| store.records.get_mut(&database.get_identity_hash().get()))
    {
        record.version_change = Some(transaction);
    }
}

pub(crate) fn finish_version_change(
    scope: &mut v8::PinScope<'_, '_>,
    database: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = scope
        .get_slot_mut::<IdbDatabaseStore>()
        .and_then(|store| store.records.get_mut(&database.get_identity_hash().get()))
    {
        record.version_change = None;
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbDatabaseRecord> {
    scope
        .get_slot::<IdbDatabaseStore>()?
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
fn get_version(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.version as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_object_store_names(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let names = super::idb_factory::object_store_names(scope, &record.name);
    match super::dom_string_list::create(scope, names) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    if let Some(record) = scope
        .get_slot_mut::<IdbDatabaseStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.closed = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn create_object_store(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(transaction) = record.version_change else {
        throw_dom(
            scope,
            "InvalidStateError",
            "No version change transaction is running.",
        );
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let key_path = options.and_then(|options| optional_string(scope, options, "keyPath"));
    let auto_increment = options
        .is_some_and(|options| super::event::boolean_property(scope, options, "autoIncrement"));
    if let Err(error) = super::idb_factory::create_object_store(
        scope,
        &record.name,
        name.clone(),
        key_path,
        auto_increment,
    ) {
        super::idb_factory::throw_operation_error(scope, error);
        return;
    }
    let transaction = v8::Local::new(scope, &transaction);
    match super::idb_object_store::create(scope, record.name, name, transaction) {
        Ok(store) => result.set(store.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn delete_object_store(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.version_change.is_none() {
        throw_dom(
            scope,
            "InvalidStateError",
            "No version change transaction is running.",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Err(error) = super::idb_factory::delete_object_store(scope, &record.name, &name) {
        super::idb_factory::throw_operation_error(scope, error);
    }
}

fn transaction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        throw_dom(
            scope,
            "InvalidStateError",
            "The database connection is closed.",
        );
        return;
    }
    let names = read_names(scope, arguments.get(0));
    if names.is_empty() {
        throw_dom(
            scope,
            "InvalidAccessError",
            "No object stores were specified.",
        );
        return;
    }
    let available = super::idb_factory::object_store_names(scope, &record.name);
    if let Some(missing) = names.iter().find(|name| !available.contains(name)) {
        throw_dom(
            scope,
            "NotFoundError",
            &format!("The object store '{missing}' was not found."),
        );
        return;
    }
    let mode = if arguments.get(1).is_undefined() {
        "readonly".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    if mode != "readonly" && mode != "readwrite" {
        crate::webidl::throw_type_error(scope, "The transaction mode is invalid");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(2)).ok();
    let durability = options
        .and_then(|options| optional_string(scope, options, "durability"))
        .unwrap_or_else(|| "default".to_owned());
    match super::idb_transaction::create(
        scope,
        arguments.this(),
        record.name,
        names,
        &mode,
        &durability,
    ) {
        Ok(transaction) => result.set(transaction.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn read_names(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Vec<String> {
    if value.is_string() || value.is_string_object() {
        return vec![crate::webidl::value_to_string(scope, value)];
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return Vec::new();
    };
    let Some(length) = object
        .get(scope, v8::String::new(scope, "length").unwrap().into())
        .and_then(|value| value.uint32_value(scope))
    else {
        return Vec::new();
    };
    let mut names = Vec::with_capacity(length as usize);
    for index in 0..length {
        if let Some(value) = object.get_index(scope, index) {
            names.push(crate::webidl::value_to_string(scope, value));
        }
    }
    names
}

fn optional_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let value = object.get(scope, v8::String::new(scope, name)?.into())?;
    if value.is_null_or_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbDatabaseRecord) -> Option<v8::Global<v8::Function>>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match select(&record) {
            Some(value) => result.set(v8::Local::new(scope, &value).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    select: impl FnOnce(&mut IdbDatabaseRecord) -> &mut Option<v8::Global<v8::Function>>,
) {
    let value = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|value| v8::Global::new(scope, value));
    if let Some(record) = scope.get_slot_mut::<IdbDatabaseStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        *select(record) = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_onabort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |x| x.onabort.clone())
}
fn set_onabort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x| &mut x.onabort)
}
fn get_onclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |x| x.onclose.clone())
}
fn set_onclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x| &mut x.onclose)
}
fn get_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |x| x.onerror.clone())
}
fn set_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x| &mut x.onerror)
}
fn get_onversionchange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |x| x.onversionchange.clone())
}
fn set_onversionchange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x| &mut x.onversionchange)
}

fn throw_dom(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    if let Ok(exception) = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbDatabaseStore>() {
        store.constructor.remove(realm_id);
    }
}
