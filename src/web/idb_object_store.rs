use std::collections::HashMap;

use super::idb_factory::{IdbOperationError, IdbStoredRecord};

#[derive(Default)]
pub(crate) struct IdbObjectStoreStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbObjectStoreRecord>,
}

#[derive(Clone)]
pub(crate) struct IdbObjectStoreRecord {
    pub database_name: String,
    pub name: String,
    pub transaction: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbObjectStoreStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBObjectStore", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbObjectStoreStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBObjectStore",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "name", get_name, set_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "keyPath", get_key_path)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "indexNames", get_index_names)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transaction", get_transaction)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "autoIncrement", get_auto_increment)?;
    crate::webidl::define_method(scope, prototype, "add", 1, add)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "count", 0, count)?;
    crate::webidl::define_method(scope, prototype, "createIndex", 2, create_index)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "deleteIndex", 1, delete_index)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getAll", 0, get_all)?;
    crate::webidl::define_method(scope, prototype, "getAllKeys", 0, get_all_keys)?;
    crate::webidl::define_method(scope, prototype, "getAllRecords", 0, get_all_records)?;
    crate::webidl::define_method(scope, prototype, "getKey", 1, get_key)?;
    crate::webidl::define_method(scope, prototype, "index", 1, index)?;
    crate::webidl::define_method(scope, prototype, "openCursor", 0, open_cursor)?;
    crate::webidl::define_method(scope, prototype, "openKeyCursor", 0, open_key_cursor)?;
    crate::webidl::define_method(scope, prototype, "put", 1, put)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbObjectStoreStore>()
        .ok_or_else(|| "IDBObjectStore state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    database_name: String,
    name: String,
    transaction: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBObjectStore".to_owned());
    }
    let transaction = v8::Global::new(scope, transaction);
    scope
        .get_slot_mut::<IdbObjectStoreStore>()
        .ok_or_else(|| "IDBObjectStore state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdbObjectStoreRecord {
                database_name,
                name,
                transaction,
            },
        );
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbObjectStoreRecord> {
    scope
        .get_slot::<IdbObjectStoreStore>()?
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
fn set_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let transaction = v8::Local::new(scope, &snapshot.transaction);
    if !super::idb_transaction::record(scope, transaction)
        .is_some_and(|transaction| transaction.active && transaction.mode == "versionchange")
    {
        throw_dom(
            scope,
            "InvalidStateError",
            "The name can only be changed during upgrade.",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Err(error) = super::idb_factory::rename_object_store(
        scope,
        &snapshot.database_name,
        &snapshot.name,
        name.clone(),
    ) {
        super::idb_factory::throw_operation_error(scope, error);
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<IdbObjectStoreStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.name = name;
    }
}

fn store_data(
    scope: &v8::PinScope<'_, '_>,
    record: &IdbObjectStoreRecord,
) -> Option<super::idb_factory::IdbObjectStoreData> {
    super::idb_factory::object_store_data(scope, &record.database_name, &record.name)
}

fn get_key_path(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match store_data(scope, &record).and_then(|store| store.key_path) {
        Some(path) => {
            if let Some(path) = v8::String::new(scope, &path) {
                result.set(path.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}
fn get_index_names(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let names = store_data(scope, &record)
        .map(|store| store.indexes.keys().cloned().collect())
        .unwrap_or_default();
    match super::dom_string_list::create(scope, names) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn get_transaction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.transaction).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_auto_increment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = store_data(scope, &record).is_some_and(|store| store.auto_increment);
    result.set(v8::Boolean::new(scope, value).into());
}

fn writable(scope: &mut v8::PinScope<'_, '_>, record: &IdbObjectStoreRecord) -> bool {
    let transaction = v8::Local::new(scope, &record.transaction);
    let Some(transaction) = super::idb_transaction::record(scope, transaction) else {
        return false;
    };
    if !transaction.active {
        throw_dom(
            scope,
            "TransactionInactiveError",
            "The transaction is inactive.",
        );
        return false;
    }
    if transaction.mode == "readonly" {
        throw_dom(scope, "ReadOnlyError", "The transaction is read-only.");
        return false;
    }
    true
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    put_value(scope, arguments, &mut result, false);
}
fn put(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    put_value(scope, arguments, &mut result, true);
}
fn put_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: &mut v8::ReturnValue<'_>,
    overwrite: bool,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !writable(scope, &record) {
        return;
    }
    let explicit_key = (!arguments.get(1).is_undefined()).then(|| arguments.get(1));
    match super::idb_factory::put(
        scope,
        &record.database_name,
        &record.name,
        arguments.get(0),
        explicit_key,
        overwrite,
    ) {
        Ok(key) => {
            let key = super::idb_key_range::value_for_key(scope, &key);
            success_request(scope, arguments.this(), &record, key, result);
        }
        Err(error) => error_request(scope, arguments.this(), &record, error, result),
    }
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !writable(scope, &record) {
        return;
    }
    match super::idb_factory::clear_store(scope, &record.database_name, &record.name) {
        Ok(()) => {
            let value = v8::undefined(scope);
            success_request(scope, arguments.this(), &record, value.into(), &mut result)
        }
        Err(error) => error_request(scope, arguments.this(), &record, error, &mut result),
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !writable(scope, &record) {
        return;
    }
    match super::idb_factory::delete_matching(
        scope,
        &record.database_name,
        &record.name,
        arguments.get(0),
    ) {
        Ok(()) => {
            let value = v8::undefined(scope);
            success_request(scope, arguments.this(), &record, value.into(), &mut result)
        }
        Err(error) => error_request(scope, arguments.this(), &record, error, &mut result),
    }
}

fn matching_records(
    scope: &v8::PinScope<'_, '_>,
    record: &IdbObjectStoreRecord,
    query: v8::Local<'_, v8::Value>,
    count: Option<u32>,
) -> Vec<IdbStoredRecord> {
    let mut output = super::idb_factory::records(scope, &record.database_name, &record.name)
        .into_iter()
        .filter(|item| super::idb_key_range::matches_query(scope, query, &item.key))
        .collect::<Vec<_>>();
    if let Some(count) = count {
        output.truncate(count as usize);
    }
    output
}

fn count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let count = matching_records(scope, &record, arguments.get(0), None).len() as u32;
    let value = v8::Integer::new_from_unsigned(scope, count);
    success_request(scope, arguments.this(), &record, value.into(), &mut result);
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let item = matching_records(scope, &record, arguments.get(0), Some(1))
        .into_iter()
        .next();
    let value = item
        .as_ref()
        .map(|item| v8::Local::new(scope, &item.value))
        .unwrap_or_else(|| v8::undefined(scope).into());
    success_request(scope, arguments.this(), &record, value, &mut result);
}

fn get_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let item = matching_records(scope, &record, arguments.get(0), Some(1))
        .into_iter()
        .next();
    let value = item
        .as_ref()
        .map(|item| super::idb_key_range::value_for_key(scope, &item.key))
        .unwrap_or_else(|| v8::undefined(scope).into());
    success_request(scope, arguments.this(), &record, value, &mut result);
}

fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let count = (!arguments.get(1).is_undefined())
        .then(|| arguments.get(1).uint32_value(scope).unwrap_or(0));
    let items = matching_records(scope, &record, arguments.get(0), count);
    let values = v8::Array::new(scope, items.len() as i32);
    for (index, item) in items.iter().enumerate() {
        let _ = values.set_index(scope, index as u32, v8::Local::new(scope, &item.value));
    }
    success_request(scope, arguments.this(), &record, values.into(), &mut result);
}

fn get_all_keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let count = (!arguments.get(1).is_undefined())
        .then(|| arguments.get(1).uint32_value(scope).unwrap_or(0));
    let items = matching_records(scope, &record, arguments.get(0), count);
    let values = v8::Array::new(scope, items.len() as i32);
    for (index, item) in items.iter().enumerate() {
        let key = super::idb_key_range::value_for_key(scope, &item.key);
        let _ = values.set_index(scope, index as u32, key);
    }
    success_request(scope, arguments.this(), &record, values.into(), &mut result);
}

fn get_all_records(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let items = matching_records(scope, &record, arguments.get(0), None);
    let values = v8::Array::new(scope, items.len() as i32);
    for (index, item) in items.iter().enumerate() {
        let key = super::idb_key_range::value_for_key(scope, &item.key);
        let value = v8::Local::new(scope, &item.value);
        if let Ok(idb_record) = super::idb_record::create(scope, key, key, value) {
            let _ = values.set_index(scope, index as u32, idb_record.into());
        }
    }
    success_request(scope, arguments.this(), &record, values.into(), &mut result);
}

fn create_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let transaction = v8::Local::new(scope, &record.transaction);
    if !super::idb_transaction::record(scope, transaction)
        .is_some_and(|transaction| transaction.active && transaction.mode == "versionchange")
    {
        throw_dom(
            scope,
            "InvalidStateError",
            "Indexes can only be created during upgrade.",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let key_path = crate::webidl::value_to_string(scope, arguments.get(1));
    let options = v8::Local::<v8::Object>::try_from(arguments.get(2)).ok();
    let unique =
        options.is_some_and(|options| super::event::boolean_property(scope, options, "unique"));
    let multi_entry =
        options.is_some_and(|options| super::event::boolean_property(scope, options, "multiEntry"));
    if let Err(error) = super::idb_factory::create_index(
        scope,
        &record.database_name,
        &record.name,
        name.clone(),
        key_path,
        multi_entry,
        unique,
    ) {
        super::idb_factory::throw_operation_error(scope, error);
        return;
    }
    match super::idb_index::create(scope, arguments.this(), name) {
        Ok(index) => result.set(index.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn delete_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Err(error) =
        super::idb_factory::delete_index(scope, &record.database_name, &record.name, &name)
    {
        super::idb_factory::throw_operation_error(scope, error);
    }
}

fn index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if !store_data(scope, &record).is_some_and(|store| store.indexes.contains_key(&name)) {
        throw_dom(scope, "NotFoundError", "The index was not found.");
        return;
    }
    match super::idb_index::create(scope, arguments.this(), name) {
        Ok(index) => result.set(index.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn open_cursor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    open_cursor_impl(scope, arguments, &mut result, true);
}
fn open_key_cursor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    open_cursor_impl(scope, arguments, &mut result, false);
}
fn open_cursor_impl(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: &mut v8::ReturnValue<'_>,
    with_value: bool,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let entries = matching_records(scope, &record, arguments.get(0), None);
    let direction = if arguments.get(1).is_undefined() {
        "next".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    let transaction = v8::Local::new(scope, &record.transaction);
    match super::idb_cursor::create_request(
        scope,
        arguments.this().into(),
        transaction,
        entries,
        &direction,
        with_value,
    ) {
        Ok(request) => result.set(request.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn success_request(
    scope: &mut v8::PinScope<'_, '_>,
    source: v8::Local<'_, v8::Object>,
    record: &IdbObjectStoreRecord,
    value: v8::Local<'_, v8::Value>,
    result: &mut v8::ReturnValue<'_>,
) {
    let transaction = v8::Local::new(scope, &record.transaction);
    match super::idb_request::create_success(scope, Some(source.into()), Some(transaction), value) {
        Ok(request) => result.set(request.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn error_request(
    scope: &mut v8::PinScope<'_, '_>,
    source: v8::Local<'_, v8::Object>,
    record: &IdbObjectStoreRecord,
    error: IdbOperationError,
    result: &mut v8::ReturnValue<'_>,
) {
    let Ok(exception) = super::dom_exception::create(scope, error.message, error.name.to_owned())
    else {
        return;
    };
    let transaction = v8::Local::new(scope, &record.transaction);
    match super::idb_request::create_error(scope, Some(source.into()), Some(transaction), exception)
    {
        Ok(request) => result.set(request.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn throw_dom(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    if let Ok(exception) = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbObjectStoreStore>() {
        store.constructor.remove(realm_id);
    }
}
