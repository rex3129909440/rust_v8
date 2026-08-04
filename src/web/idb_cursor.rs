use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct IdbCursorEntry {
    key: super::idb_key_range::IdbKey,
    primary_key: super::idb_key_range::IdbKey,
    value: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct IdbCursorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbCursorRecord>,
}

#[derive(Clone)]
struct IdbCursorRecord {
    source: v8::Global<v8::Value>,
    transaction: v8::Global<v8::Object>,
    direction: String,
    request: v8::Global<v8::Object>,
    entries: Vec<IdbCursorEntry>,
    position: usize,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbCursorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBCursor", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbCursorStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBCursor",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "source", get_source)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "direction", get_direction)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "key", get_key)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "primaryKey", get_primary_key)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "request", get_request)?;
    crate::webidl::define_method(scope, prototype, "advance", 1, advance)?;
    crate::webidl::define_method(scope, prototype, "continue", 0, continue_cursor)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "continuePrimaryKey",
        2,
        continue_primary_key,
    )?;
    crate::webidl::define_method(scope, prototype, "delete", 0, delete)?;
    crate::webidl::define_method(scope, prototype, "update", 1, update)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbCursorStore>()
        .ok_or_else(|| "IDBCursor state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_request<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Value>,
    transaction: v8::Local<'_, v8::Object>,
    records: Vec<super::idb_factory::IdbStoredRecord>,
    direction: &str,
    with_value: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let entries = records
        .into_iter()
        .map(|record| IdbCursorEntry {
            key: record.key.clone(),
            primary_key: record.key,
            value: Some(record.value),
        })
        .collect();
    create(scope, source, transaction, entries, direction, with_value)
}

pub(crate) fn create_index_request<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Value>,
    transaction: v8::Local<'_, v8::Object>,
    records: Vec<super::idb_index::IdbIndexEntry>,
    direction: &str,
    with_value: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let entries = records
        .into_iter()
        .map(|record| IdbCursorEntry {
            key: record.key,
            primary_key: record.primary_key,
            value: Some(record.value),
        })
        .collect();
    create(scope, source, transaction, entries, direction, with_value)
}

fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Value>,
    transaction: v8::Local<'_, v8::Object>,
    mut entries: Vec<IdbCursorEntry>,
    direction: &str,
    with_value: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if matches!(direction, "prev" | "prevunique") {
        entries.reverse();
    }
    let request = super::idb_request::create_success(
        scope,
        Some(source),
        Some(transaction),
        v8::undefined(scope).into(),
    )?;
    let constructor = if with_value {
        super::idb_cursor_with_value::ensure_constructor(scope)?
    } else {
        ensure_constructor(scope)?
    };
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let cursor = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, cursor, prototype.into()) != Some(true) {
        return Err("cannot create IDBCursor".to_owned());
    }
    let source = v8::Global::new(scope, source);
    let transaction = v8::Global::new(scope, transaction);
    let request_global = v8::Global::new(scope, request);
    scope
        .get_slot_mut::<IdbCursorStore>()
        .ok_or_else(|| "IDBCursor state was not prepared".to_owned())?
        .records
        .insert(
            cursor.get_identity_hash().get(),
            IdbCursorRecord {
                source,
                transaction,
                direction: direction.to_owned(),
                request: request_global,
                entries,
                position: 0,
            },
        );
    let first = if is_exhausted(scope, cursor) {
        v8::undefined(scope).into()
    } else {
        cursor.into()
    };
    super::idb_request::set_result(scope, request, first, false);
    Ok(request)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbCursorRecord> {
    scope
        .get_slot::<IdbCursorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_cursor(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

fn is_exhausted(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_none_or(|record| record.position >= record.entries.len())
}

fn current(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbCursorEntry> {
    let record = record(scope, object)?;
    record.entries.get(record.position).cloned()
}

pub(crate) fn current_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Value>> {
    current(scope, object)?.value
}

fn get_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.source));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_direction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.direction) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match current(scope, arguments.this()) {
        Some(entry) => result.set(super::idb_key_range::value_for_key(scope, &entry.key)),
        None if is_cursor(scope, arguments.this()) => result.set(v8::undefined(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
fn get_primary_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match current(scope, arguments.this()) {
        Some(entry) => result.set(super::idb_key_range::value_for_key(
            scope,
            &entry.primary_key,
        )),
        None if is_cursor(scope, arguments.this()) => result.set(v8::undefined(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
fn get_request(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.request).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn advance(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let count = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if count == 0 {
        crate::webidl::throw_type_error(scope, "The count must be greater than zero");
        return;
    }
    move_by(scope, arguments.this(), count as usize);
}
fn continue_cursor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    move_by(scope, arguments.this(), 1);
}
fn continue_primary_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "2 arguments required");
        return;
    }
    let Some(key) = super::idb_key_range::key_from_value(scope, arguments.get(0)) else {
        throw_dom(scope, "DataError", "The key is invalid.");
        return;
    };
    let Some(primary) = super::idb_key_range::key_from_value(scope, arguments.get(1)) else {
        throw_dom(scope, "DataError", "The primary key is invalid.");
        return;
    };
    let id = arguments.this().get_identity_hash().get();
    if let Some(record) = scope
        .get_slot_mut::<IdbCursorStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        let next = record
            .entries
            .iter()
            .enumerate()
            .skip(record.position + 1)
            .find(|(_, entry)| {
                let order = super::idb_key_range::compare(&entry.key, &key);
                order == std::cmp::Ordering::Greater
                    || (order == std::cmp::Ordering::Equal
                        && super::idb_key_range::compare(&entry.primary_key, &primary)
                            != std::cmp::Ordering::Less)
            })
            .map(|(index, _)| index)
            .unwrap_or(record.entries.len());
        record.position = next;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    publish_position(scope, arguments.this());
}

fn move_by(scope: &mut v8::PinScope<'_, '_>, cursor: v8::Local<'_, v8::Object>, count: usize) {
    if let Some(record) = scope
        .get_slot_mut::<IdbCursorStore>()
        .and_then(|store| store.records.get_mut(&cursor.get_identity_hash().get()))
    {
        record.position = record
            .position
            .saturating_add(count)
            .min(record.entries.len());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    publish_position(scope, cursor);
}

fn publish_position(scope: &mut v8::PinScope<'_, '_>, cursor: v8::Local<'_, v8::Object>) {
    let Some(record) = record(scope, cursor) else {
        return;
    };
    let request = v8::Local::new(scope, &record.request);
    let value = if record.position < record.entries.len() {
        cursor.into()
    } else {
        v8::undefined(scope).into()
    };
    super::idb_request::set_result(scope, request, value, true);
    super::idb_request::fire_success(scope, request);
}

fn source_store(
    scope: &v8::PinScope<'_, '_>,
    record: &IdbCursorRecord,
) -> Option<super::idb_object_store::IdbObjectStoreRecord> {
    let source = v8::Local::new(scope, &record.source);
    let object = v8::Local::<v8::Object>::try_from(source).ok()?;
    if let Some(store) = super::idb_object_store::record(scope, object) {
        return Some(store);
    }
    let index = super::idb_index::record(scope, object)?;
    let object_store = v8::Local::new(scope, &index.object_store);
    super::idb_object_store::record(scope, object_store)
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(cursor) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(entry) = current(scope, arguments.this()) else {
        throw_dom(scope, "InvalidStateError", "The cursor is exhausted.");
        return;
    };
    let Some(store) = source_store(scope, &cursor) else {
        throw_dom(scope, "InvalidStateError", "The cursor source is invalid.");
        return;
    };
    let key = super::idb_key_range::value_for_key(scope, &entry.primary_key);
    if let Err(error) =
        super::idb_factory::delete_matching(scope, &store.database_name, &store.name, key)
    {
        super::idb_factory::throw_operation_error(scope, error);
        return;
    }
    let transaction = v8::Local::new(scope, &cursor.transaction);
    match super::idb_request::create_success(
        scope,
        Some(arguments.this().into()),
        Some(transaction),
        v8::undefined(scope).into(),
    ) {
        Ok(request) => result.set(request.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(cursor) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(entry) = current(scope, arguments.this()) else {
        throw_dom(scope, "InvalidStateError", "The cursor is exhausted.");
        return;
    };
    let Some(store) = source_store(scope, &cursor) else {
        throw_dom(scope, "InvalidStateError", "The cursor source is invalid.");
        return;
    };
    let key = super::idb_key_range::value_for_key(scope, &entry.primary_key);
    let store_data =
        super::idb_factory::object_store_data(scope, &store.database_name, &store.name);
    let explicit = store_data
        .as_ref()
        .is_some_and(|store| store.key_path.is_none())
        .then_some(key);
    match super::idb_factory::put(
        scope,
        &store.database_name,
        &store.name,
        arguments.get(0),
        explicit,
        true,
    ) {
        Ok(key) => {
            let key = super::idb_key_range::value_for_key(scope, &key);
            let transaction = v8::Local::new(scope, &cursor.transaction);
            match super::idb_request::create_success(
                scope,
                Some(arguments.this().into()),
                Some(transaction),
                key,
            ) {
                Ok(request) => result.set(request.into()),
                Err(message) => crate::webidl::throw_type_error(scope, &message),
            }
        }
        Err(error) => super::idb_factory::throw_operation_error(scope, error),
    }
}

fn throw_dom(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    if let Ok(exception) = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbCursorStore>() {
        store.constructor.remove(realm_id);
    }
}
