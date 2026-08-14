use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdbIndexStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbIndexRecord>,
}

#[derive(Clone)]
pub(crate) struct IdbIndexRecord {
    pub object_store: v8::Global<v8::Object>,
    pub name: String,
}

#[derive(Clone)]
pub(crate) struct IdbIndexEntry {
    pub key: super::idb_key_range::IdbKey,
    pub primary_key: super::idb_key_range::IdbKey,
    pub value: v8::Global<v8::Value>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbIndexStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBIndex", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbIndexStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBIndex",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "name", get_name, set_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "objectStore", get_object_store)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "keyPath", get_key_path)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "multiEntry", get_multi_entry)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "unique", get_unique)?;
    crate::webidl::define_method(scope, prototype, "count", 0, count)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getAll", 0, get_all)?;
    crate::webidl::define_method(scope, prototype, "getAllKeys", 0, get_all_keys)?;
    crate::webidl::define_method(scope, prototype, "getAllRecords", 0, get_all_records)?;
    crate::webidl::define_method(scope, prototype, "getKey", 1, get_key)?;
    crate::webidl::define_method(scope, prototype, "openCursor", 0, open_cursor)?;
    crate::webidl::define_method(scope, prototype, "openKeyCursor", 0, open_key_cursor)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbIndexStore>()
        .ok_or_else(|| "IDBIndex state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object_store: v8::Local<'_, v8::Object>,
    name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBIndex".to_owned());
    }
    let object_store = v8::Global::new(scope, object_store);
    scope
        .get_slot_mut::<IdbIndexStore>()
        .ok_or_else(|| "IDBIndex state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdbIndexRecord { object_store, name },
        );
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbIndexRecord> {
    scope
        .get_slot::<IdbIndexStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn metadata(
    scope: &v8::PinScope<'_, '_>,
    record: &IdbIndexRecord,
) -> Option<(
    super::idb_object_store::IdbObjectStoreRecord,
    super::idb_factory::IdbIndexData,
)> {
    let object_store = v8::Local::new(scope, &record.object_store);
    let store_record = super::idb_object_store::record(scope, object_store)?;
    let store_data = super::idb_factory::object_store_data(
        scope,
        &store_record.database_name,
        &store_record.name,
    )?;
    let index = store_data.indexes.get(&record.name)?.clone();
    Some((store_record, index))
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(name) = v8::String::new(scope, &record.name) {
            result.set(name.into());
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
    let Some((store, _)) = metadata(scope, &snapshot) else {
        throw_dom(scope, "InvalidStateError", "The index was deleted.");
        return;
    };
    let transaction = v8::Local::new(scope, &store.transaction);
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
    if let Err(error) = super::idb_factory::rename_index(
        scope,
        &store.database_name,
        &store.name,
        &snapshot.name,
        name.clone(),
    ) {
        super::idb_factory::throw_operation_error(scope, error);
        return;
    }
    if let Some(record) = scope.get_slot_mut::<IdbIndexStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.name = name;
    }
}

fn get_object_store(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.object_store).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
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
    if let Some((_, index)) = metadata(scope, &record) {
        if let Some(value) = v8::String::new(scope, &index.key_path) {
            result.set(value.into());
        }
    } else {
        throw_dom(scope, "InvalidStateError", "The index was deleted.");
    }
}
fn get_multi_entry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = metadata(scope, &record).is_some_and(|(_, index)| index.multi_entry);
    result.set(v8::Boolean::new(scope, value).into());
}
fn get_unique(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = metadata(scope, &record).is_some_and(|(_, index)| index.unique);
    result.set(v8::Boolean::new(scope, value).into());
}

fn entries(
    scope: &v8::PinScope<'_, '_>,
    record: &IdbIndexRecord,
    query: v8::Local<'_, v8::Value>,
    count: Option<u32>,
) -> Vec<IdbIndexEntry> {
    let Some((store, index)) = metadata(scope, record) else {
        return Vec::new();
    };
    let mut output = Vec::new();
    for stored in super::idb_factory::records(scope, &store.database_name, &store.name) {
        let value = v8::Local::new(scope, &stored.value);
        let Some(index_value) = super::idb_factory::property_at_path(scope, value, &index.key_path)
        else {
            continue;
        };
        let Some(index_key) = super::idb_key_range::key_from_value(scope, index_value) else {
            continue;
        };
        if super::idb_key_range::matches_query(scope, query, &index_key) {
            output.push(IdbIndexEntry {
                key: index_key,
                primary_key: stored.key,
                value: stored.value,
            });
        }
    }
    output.sort_by(|left, right| {
        let order = super::idb_key_range::compare(&left.key, &right.key);
        if order == Ordering::Equal {
            super::idb_key_range::compare(&left.primary_key, &right.primary_key)
        } else {
            order
        }
    });
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
    let value = v8::Integer::new_from_unsigned(
        scope,
        entries(scope, &record, arguments.get(0), None).len() as u32,
    );
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
    let entry = entries(scope, &record, arguments.get(0), Some(1))
        .into_iter()
        .next();
    let value = entry
        .map(|entry| v8::Local::new(scope, &entry.value))
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
    let entry = entries(scope, &record, arguments.get(0), Some(1))
        .into_iter()
        .next();
    let value = entry
        .map(|entry| super::idb_key_range::value_for_key(scope, &entry.primary_key))
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
    let entries = entries(scope, &record, arguments.get(0), count);
    let output = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let _ = output.set_index(scope, index as u32, v8::Local::new(scope, &entry.value));
    }
    success_request(scope, arguments.this(), &record, output.into(), &mut result);
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
    let entries = entries(scope, &record, arguments.get(0), count);
    let output = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let value = super::idb_key_range::value_for_key(scope, &entry.primary_key);
        let _ = output.set_index(scope, index as u32, value);
    }
    success_request(scope, arguments.this(), &record, output.into(), &mut result);
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
    let entries = entries(scope, &record, arguments.get(0), None);
    let output = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let key = super::idb_key_range::value_for_key(scope, &entry.key);
        let primary_key = super::idb_key_range::value_for_key(scope, &entry.primary_key);
        let value = v8::Local::new(scope, &entry.value);
        if let Ok(record) = super::idb_record::create(scope, key, primary_key, value) {
            let _ = output.set_index(scope, index as u32, record.into());
        }
    }
    success_request(scope, arguments.this(), &record, output.into(), &mut result);
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
    let entries = entries(scope, &record, arguments.get(0), None);
    let direction = if arguments.get(1).is_undefined() {
        "next".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    let Some((store, _)) = metadata(scope, &record) else {
        throw_dom(scope, "InvalidStateError", "The index was deleted.");
        return;
    };
    let transaction = v8::Local::new(scope, &store.transaction);
    match super::idb_cursor::create_index_request(
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
    record: &IdbIndexRecord,
    value: v8::Local<'_, v8::Value>,
    result: &mut v8::ReturnValue<'_>,
) {
    let Some((store, _)) = metadata(scope, record) else {
        return;
    };
    let transaction = v8::Local::new(scope, &store.transaction);
    match super::idb_request::create_success(scope, Some(source.into()), Some(transaction), value) {
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
    if let Some(store) = scope.get_slot_mut::<IdbIndexStore>() {
        store.constructor.remove(realm_id);
    }
}
