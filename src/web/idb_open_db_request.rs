use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdbOpenDbRequestStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbOpenDbRequestRecord>,
}

#[derive(Clone)]
struct IdbOpenDbRequestRecord {
    old_version: u64,
    new_version: Option<u64>,
    needs_upgrade: bool,
    processed_upgrade: bool,
    database: Option<v8::Global<v8::Object>>,
    transaction: Option<v8::Global<v8::Object>>,
    onblocked: Option<v8::Global<v8::Function>>,
    onupgradeneeded: Option<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbOpenDbRequestStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBOpenDBRequest", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbOpenDbRequestStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBOpenDBRequest",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "onblocked", get_onblocked, set_onblocked)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onupgradeneeded",
        get_onupgradeneeded,
        set_onupgradeneeded,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::idb_request::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbOpenDbRequestStore>()
        .ok_or_else(|| "IDBOpenDBRequest state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    database: v8::Local<'_, v8::Object>,
    transaction: Option<v8::Local<'_, v8::Object>>,
    old_version: u64,
    new_version: Option<u64>,
    needs_upgrade: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let request = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, request, prototype.into()) != Some(true) {
        return Err("cannot create IDBOpenDBRequest".to_owned());
    }
    super::idb_request::attach(
        scope,
        request,
        None,
        None,
        database.into(),
        None,
        false,
        false,
    )?;
    let database = Some(v8::Global::new(scope, database));
    let transaction = transaction.map(|value| v8::Global::new(scope, value));
    scope
        .get_slot_mut::<IdbOpenDbRequestStore>()
        .ok_or_else(|| "IDBOpenDBRequest state was not prepared".to_owned())?
        .records
        .insert(
            request.get_identity_hash().get(),
            IdbOpenDbRequestRecord {
                old_version,
                new_version,
                needs_upgrade,
                processed_upgrade: false,
                database,
                transaction,
                onblocked: None,
                onupgradeneeded: None,
            },
        );
    Ok(request)
}

pub(crate) fn create_delete<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    old_version: u64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let request = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, request, prototype.into()) != Some(true) {
        return Err("cannot create IDBOpenDBRequest".to_owned());
    }
    super::idb_request::attach(
        scope,
        request,
        None,
        None,
        v8::undefined(scope).into(),
        None,
        false,
        false,
    )?;
    scope
        .get_slot_mut::<IdbOpenDbRequestStore>()
        .ok_or_else(|| "IDBOpenDBRequest state was not prepared".to_owned())?
        .records
        .insert(
            request.get_identity_hash().get(),
            IdbOpenDbRequestRecord {
                old_version,
                new_version: None,
                needs_upgrade: false,
                processed_upgrade: false,
                database: None,
                transaction: None,
                onblocked: None,
                onupgradeneeded: None,
            },
        );
    Ok(request)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbOpenDbRequestRecord> {
    scope
        .get_slot::<IdbOpenDbRequestStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn before_success_handler(
    scope: &mut v8::PinScope<'_, '_>,
    request: v8::Local<'_, v8::Object>,
) {
    let Some(record) = record(scope, request) else {
        return;
    };
    if !record.processed_upgrade {
        process_upgrade(scope, request);
    }
    super::idb_request::mark_done(scope, request);
}

fn process_upgrade(scope: &mut v8::PinScope<'_, '_>, request: v8::Local<'_, v8::Object>) {
    let id = request.get_identity_hash().get();
    let Some(record) = record(scope, request) else {
        return;
    };
    super::idb_request::mark_done(scope, request);
    if let Some(transaction) = record.transaction.as_ref() {
        let transaction = v8::Local::new(scope, transaction);
        super::idb_request::set_transaction(scope, request, transaction);
    }
    if record.needs_upgrade {
        if let Ok(event) = super::idb_version_change_event::create(
            scope,
            "upgradeneeded",
            record.old_version,
            record.new_version,
            "none",
            "",
        ) {
            if let Some(handler) = record.onupgradeneeded {
                let handler = v8::Local::new(scope, &handler);
                let _ = handler.call(scope, request.into(), &[event.into()]);
            }
            let _ = super::event_target::dispatch(scope, request, event);
        }
    }
    if let Some(transaction) = record.transaction {
        let transaction = v8::Local::new(scope, &transaction);
        super::idb_transaction::finish_version_change(scope, transaction);
    }
    if let Some(database) = record.database {
        let database = v8::Local::new(scope, &database);
        super::idb_database::finish_version_change(scope, database);
    }
    super::idb_request::clear_transaction(scope, request);
    if let Some(record) = scope
        .get_slot_mut::<IdbOpenDbRequestStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.processed_upgrade = true;
        record.transaction = None;
    }
}

fn handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Function>> {
    v8::Local::<v8::Function>::try_from(value)
        .ok()
        .map(|value| v8::Global::new(scope, value))
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbOpenDbRequestRecord) -> Option<v8::Global<v8::Function>>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match select(&record) {
            Some(handler) => result.set(v8::Local::new(scope, &handler).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_onblocked(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |record| record.onblocked.clone())
}
fn get_onupgradeneeded(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |record| record.onupgradeneeded.clone())
}
fn set_onblocked(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<IdbOpenDbRequestStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.onblocked = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_onupgradeneeded(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(scope, arguments.get(0));
    let mut should_process = false;
    if let Some(record) = scope
        .get_slot_mut::<IdbOpenDbRequestStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.onupgradeneeded = value;
        should_process = !record.processed_upgrade;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if should_process {
        process_upgrade(scope, arguments.this());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbOpenDbRequestStore>() {
        store.constructor.remove(realm_id);
    }
}
