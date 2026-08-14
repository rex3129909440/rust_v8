use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::idb_key_range::{IdbKey, compare, key_from_value, value_for_key};

#[derive(Default)]
pub(crate) struct IdbFactoryStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    factories: HashSet<i32>,
    databases: HashMap<String, IdbDatabaseData>,
}

#[derive(Clone, Default)]
pub(crate) struct IdbDatabaseData {
    pub version: u64,
    pub object_stores: HashMap<String, IdbObjectStoreData>,
}

#[derive(Clone)]
pub(crate) struct IdbObjectStoreData {
    pub key_path: Option<String>,
    pub auto_increment: bool,
    pub next_key: u64,
    pub records: Vec<IdbStoredRecord>,
    pub indexes: HashMap<String, IdbIndexData>,
}

#[derive(Clone)]
pub(crate) struct IdbStoredRecord {
    pub key: IdbKey,
    pub value: v8::Global<v8::Value>,
}

#[derive(Clone)]
pub(crate) struct IdbIndexData {
    pub key_path: String,
    pub multi_entry: bool,
    pub unique: bool,
}

pub(crate) struct IdbOperationError {
    pub name: &'static str,
    pub message: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbFactoryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBFactory", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<IdbFactoryStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBFactory",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "cmp", 2, cmp)?;
    crate::webidl::define_method(scope, prototype, "databases", 0, databases)?;
    crate::webidl::define_method(scope, prototype, "deleteDatabase", 1, delete_database)?;
    crate::webidl::define_method(scope, prototype, "open", 1, open)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbFactoryStore>()
        .ok_or_else(|| "IDBFactory state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let factory = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, factory, prototype.into()) != Some(true) {
        return Err("cannot create IDBFactory".to_owned());
    }
    scope
        .get_slot_mut::<IdbFactoryStore>()
        .ok_or_else(|| "IDBFactory state was not prepared".to_owned())?
        .factories
        .insert(factory.get_identity_hash().get());
    Ok(factory)
}

fn branded(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<IdbFactoryStore>()
        .is_some_and(|store| store.factories.contains(&object.get_identity_hash().get()))
}

fn cmp(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !branded(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(first) = key_from_value(scope, arguments.get(0)) else {
        throw_dom(
            scope,
            "DataError",
            "The first parameter is not a valid key.",
        );
        return;
    };
    let Some(second) = key_from_value(scope, arguments.get(1)) else {
        throw_dom(
            scope,
            "DataError",
            "The second parameter is not a valid key.",
        );
        return;
    };
    let value = match compare(&first, &second) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    };
    result.set(v8::Integer::new(scope, value).into());
}

fn databases(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !branded(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "IDBFactory", "databases", result);
        return;
    }
    let mut entries = scope
        .get_slot::<IdbFactoryStore>()
        .map(|store| {
            store
                .databases
                .iter()
                .map(|(name, database)| (name.clone(), database.version))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let output = v8::Array::new(scope, entries.len() as i32);
    for (index, (name, version)) in entries.iter().enumerate() {
        let entry = v8::Object::new(scope);
        define_string(scope, entry, "name", name);
        define_number(scope, entry, "version", *version as f64);
        let _ = output.set_index(scope, index as u32, entry.into());
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, output.into()) {
        result.set(promise.into());
    }
}

fn delete_database(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !branded(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let old_version = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.remove(&name))
        .map(|database| database.version)
        .unwrap_or(0);
    match super::idb_open_db_request::create_delete(scope, old_version) {
        Ok(request) => result.set(request.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !branded(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let requested = if arguments.get(1).is_undefined() {
        None
    } else {
        let version = arguments.get(1).number_value(scope).unwrap_or(0.0);
        if !version.is_finite() || version < 1.0 {
            crate::webidl::throw_type_error(scope, "The version must be a positive integer");
            return;
        }
        Some(version as u64)
    };
    let old_version = scope
        .get_slot::<IdbFactoryStore>()
        .and_then(|store| store.databases.get(&name))
        .map(|database| database.version)
        .unwrap_or(0);
    let version = requested.unwrap_or_else(|| if old_version == 0 { 1 } else { old_version });
    if version < old_version {
        throw_dom(
            scope,
            "VersionError",
            "The requested version is less than the existing version.",
        );
        return;
    }
    let needs_upgrade = old_version == 0 || version > old_version;
    if needs_upgrade {
        let store = scope
            .get_slot_mut::<IdbFactoryStore>()
            .expect("IDBFactory state");
        let database = store.databases.entry(name.clone()).or_default();
        database.version = version;
    }
    let database = match super::idb_database::create(scope, name.clone(), version) {
        Ok(database) => database,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let transaction = if needs_upgrade {
        let names = object_store_names(scope, &name);
        match super::idb_transaction::create(
            scope,
            database,
            name.clone(),
            names,
            "versionchange",
            "default",
        ) {
            Ok(transaction) => {
                super::idb_database::set_version_change(scope, database, transaction);
                Some(transaction)
            }
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    } else {
        None
    };
    match super::idb_open_db_request::create(
        scope,
        database,
        transaction,
        old_version,
        Some(version),
        needs_upgrade,
    ) {
        Ok(request) => result.set(request.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn object_store_names(scope: &v8::PinScope<'_, '_>, database: &str) -> Vec<String> {
    let mut names = scope
        .get_slot::<IdbFactoryStore>()
        .and_then(|store| store.databases.get(database))
        .map(|database| database.object_stores.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    names.sort();
    names
}

pub(crate) fn create_object_store(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    name: String,
    key_path: Option<String>,
    auto_increment: bool,
) -> Result<(), IdbOperationError> {
    let database = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .ok_or_else(|| operation_error("InvalidStateError", "The database does not exist."))?;
    if database.object_stores.contains_key(&name) {
        return Err(operation_error(
            "ConstraintError",
            "An object store with this name already exists.",
        ));
    }
    database.object_stores.insert(
        name,
        IdbObjectStoreData {
            key_path,
            auto_increment,
            next_key: 1,
            records: Vec::new(),
            indexes: HashMap::new(),
        },
    );
    Ok(())
}

pub(crate) fn delete_object_store(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    name: &str,
) -> Result<(), IdbOperationError> {
    let database = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .ok_or_else(|| operation_error("InvalidStateError", "The database does not exist."))?;
    if database.object_stores.remove(name).is_none() {
        return Err(operation_error(
            "NotFoundError",
            "The specified object store was not found.",
        ));
    }
    Ok(())
}

pub(crate) fn rename_object_store(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    old_name: &str,
    new_name: String,
) -> Result<(), IdbOperationError> {
    let database = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .ok_or_else(|| operation_error("InvalidStateError", "The database does not exist."))?;
    if database.object_stores.contains_key(&new_name) {
        return Err(operation_error(
            "ConstraintError",
            "An object store with the new name already exists.",
        ));
    }
    let data = database
        .object_stores
        .remove(old_name)
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    database.object_stores.insert(new_name, data);
    Ok(())
}

pub(crate) fn object_store_data(
    scope: &v8::PinScope<'_, '_>,
    database: &str,
    name: &str,
) -> Option<IdbObjectStoreData> {
    scope
        .get_slot::<IdbFactoryStore>()?
        .databases
        .get(database)?
        .object_stores
        .get(name)
        .cloned()
}

pub(crate) fn create_index(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    store_name: &str,
    name: String,
    key_path: String,
    multi_entry: bool,
    unique: bool,
) -> Result<(), IdbOperationError> {
    let object_store = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .and_then(|database| database.object_stores.get_mut(store_name))
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    if object_store.indexes.contains_key(&name) {
        return Err(operation_error(
            "ConstraintError",
            "An index with this name already exists.",
        ));
    }
    object_store.indexes.insert(
        name,
        IdbIndexData {
            key_path,
            multi_entry,
            unique,
        },
    );
    Ok(())
}

pub(crate) fn delete_index(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    store_name: &str,
    name: &str,
) -> Result<(), IdbOperationError> {
    let object_store = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .and_then(|database| database.object_stores.get_mut(store_name))
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    if object_store.indexes.remove(name).is_none() {
        return Err(operation_error("NotFoundError", "The index was not found."));
    }
    Ok(())
}

pub(crate) fn rename_index(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    store_name: &str,
    old_name: &str,
    new_name: String,
) -> Result<(), IdbOperationError> {
    let object_store = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .and_then(|database| database.object_stores.get_mut(store_name))
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    if object_store.indexes.contains_key(&new_name) {
        return Err(operation_error(
            "ConstraintError",
            "An index with the new name already exists.",
        ));
    }
    let data = object_store
        .indexes
        .remove(old_name)
        .ok_or_else(|| operation_error("NotFoundError", "The index was not found."))?;
    object_store.indexes.insert(new_name, data);
    Ok(())
}

pub(crate) fn put(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    store_name: &str,
    value: v8::Local<'_, v8::Value>,
    explicit_key: Option<v8::Local<'_, v8::Value>>,
    overwrite: bool,
) -> Result<IdbKey, IdbOperationError> {
    let snapshot = object_store_data(scope, database, store_name)
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    let inline_value = snapshot
        .key_path
        .as_deref()
        .and_then(|path| property_at_path(scope, value, path));
    let mut generated = false;
    let key = if let Some(value) = inline_value.filter(|value| !value.is_undefined()) {
        key_from_value(scope, value)
            .ok_or_else(|| operation_error("DataError", "The inline key is not valid."))?
    } else if let Some(value) = explicit_key.filter(|value| !value.is_undefined()) {
        if snapshot.key_path.is_some() {
            return Err(operation_error(
                "DataError",
                "A key was supplied for an inline-key object store.",
            ));
        }
        key_from_value(scope, value)
            .ok_or_else(|| operation_error("DataError", "The supplied key is not valid."))?
    } else if snapshot.auto_increment {
        generated = true;
        IdbKey::Number(snapshot.next_key as f64)
    } else {
        return Err(operation_error(
            "DataError",
            "A key could not be derived for the value.",
        ));
    };
    if generated {
        if let Some(path) = snapshot.key_path.as_deref() {
            let key_value = value_for_key(scope, &key);
            set_property_at_path(scope, value, path, key_value);
        }
    }
    let stored_value = v8::Global::new(scope, value);
    let object_store = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .and_then(|database| database.object_stores.get_mut(store_name))
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    if let Some(position) = object_store
        .records
        .iter()
        .position(|record| compare(&record.key, &key) == Ordering::Equal)
    {
        if !overwrite {
            return Err(operation_error(
                "ConstraintError",
                "A record with the key already exists.",
            ));
        }
        object_store.records[position].value = stored_value;
    } else {
        object_store.records.push(IdbStoredRecord {
            key: key.clone(),
            value: stored_value,
        });
        object_store
            .records
            .sort_by(|left, right| compare(&left.key, &right.key));
    }
    if let IdbKey::Number(number) = key {
        if number >= object_store.next_key as f64 {
            object_store.next_key = number as u64 + 1;
        }
        Ok(IdbKey::Number(number))
    } else {
        Ok(key)
    }
}

pub(crate) fn records(
    scope: &v8::PinScope<'_, '_>,
    database: &str,
    store_name: &str,
) -> Vec<IdbStoredRecord> {
    object_store_data(scope, database, store_name)
        .map(|store| store.records)
        .unwrap_or_default()
}

pub(crate) fn delete_matching(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    store_name: &str,
    query: v8::Local<'_, v8::Value>,
) -> Result<(), IdbOperationError> {
    let matching = records(scope, database, store_name)
        .into_iter()
        .filter(|record| super::idb_key_range::matches_query(scope, query, &record.key))
        .map(|record| record.key)
        .collect::<Vec<_>>();
    let object_store = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .and_then(|database| database.object_stores.get_mut(store_name))
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    object_store.records.retain(|record| {
        !matching
            .iter()
            .any(|key| compare(key, &record.key) == Ordering::Equal)
    });
    Ok(())
}

pub(crate) fn clear_store(
    scope: &mut v8::PinScope<'_, '_>,
    database: &str,
    store_name: &str,
) -> Result<(), IdbOperationError> {
    let object_store = scope
        .get_slot_mut::<IdbFactoryStore>()
        .and_then(|store| store.databases.get_mut(database))
        .and_then(|database| database.object_stores.get_mut(store_name))
        .ok_or_else(|| operation_error("NotFoundError", "The object store was not found."))?;
    object_store.records.clear();
    Ok(())
}

pub(crate) fn property_at_path<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    path: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let mut current = value;
    for component in path.split('.') {
        let object = v8::Local::<v8::Object>::try_from(current).ok()?;
        current = object.get(scope, v8::String::new(scope, component)?.into())?;
    }
    Some(current)
}

fn set_property_at_path(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    path: &str,
    key: v8::Local<'_, v8::Value>,
) {
    if path.contains('.') {
        return;
    }
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        if let Some(name) = v8::String::new(scope, path) {
            let _ = object.set(scope, name.into(), key);
        }
    }
}

pub(crate) fn operation_error(name: &'static str, message: &str) -> IdbOperationError {
    IdbOperationError {
        name,
        message: message.to_owned(),
    }
}

pub(crate) fn throw_operation_error(scope: &mut v8::PinScope<'_, '_>, error: IdbOperationError) {
    throw_dom(scope, error.name, &error.message);
}

fn throw_dom(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    if let Ok(exception) = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let (Some(name), Some(value)) = (v8::String::new(scope, name), v8::String::new(scope, value))
    {
        let _ = object.set(scope, name.into(), value.into());
    }
}
fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(name) = v8::String::new(scope, name) {
        let _ = object.set(scope, name.into(), v8::Number::new(scope, value).into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbFactoryStore>() {
        store.constructors.remove(&realm_id);
    }
}
