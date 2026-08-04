use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdbTransactionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbTransactionRecord>,
}

#[derive(Clone)]
pub(crate) struct IdbTransactionRecord {
    pub database: v8::Global<v8::Object>,
    pub database_name: String,
    pub object_store_names: Vec<String>,
    pub mode: String,
    pub durability: String,
    pub active: bool,
    error: Option<v8::Global<v8::Object>>,
    onabort: Option<v8::Global<v8::Function>>,
    oncomplete: Option<v8::Global<v8::Function>>,
    onerror: Option<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbTransactionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBTransaction", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbTransactionStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBTransaction",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "objectStoreNames",
        get_object_store_names,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mode", get_mode)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "durability", get_durability)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "db", get_db)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "error", get_error)?;
    crate::webidl::define_accessor(scope, prototype, "onabort", get_onabort, set_onabort)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncomplete",
        get_oncomplete,
        set_oncomplete,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::define_method(scope, prototype, "commit", 0, commit)?;
    crate::webidl::define_method(scope, prototype, "objectStore", 1, object_store)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbTransactionStore>()
        .ok_or_else(|| "IDBTransaction state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    database: v8::Local<'_, v8::Object>,
    database_name: String,
    object_store_names: Vec<String>,
    mode: &str,
    durability: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBTransaction".to_owned());
    }
    super::event_target::attach(scope, object);
    let database = v8::Global::new(scope, database);
    scope
        .get_slot_mut::<IdbTransactionStore>()
        .ok_or_else(|| "IDBTransaction state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdbTransactionRecord {
                database,
                database_name,
                object_store_names,
                mode: mode.to_owned(),
                durability: durability.to_owned(),
                active: true,
                error: None,
                onabort: None,
                oncomplete: None,
                onerror: None,
            },
        );
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbTransactionRecord> {
    scope
        .get_slot::<IdbTransactionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn finish_version_change(
    scope: &mut v8::PinScope<'_, '_>,
    transaction: v8::Local<'_, v8::Object>,
) {
    complete(scope, transaction, false);
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
    match super::dom_string_list::create(scope, record.object_store_names) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbTransactionRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.mode)
}
fn get_durability(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.durability)
}
fn get_db(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.database).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.error {
            Some(error) => result.set(v8::Local::new(scope, &error).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn object_store(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if !record.object_store_names.contains(&name) {
        throw_dom(
            scope,
            "NotFoundError",
            "The specified object store was not found.",
        );
        return;
    }
    match super::idb_object_store::create(scope, record.database_name, name, arguments.this()) {
        Ok(store) => result.set(store.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    complete(scope, arguments.this(), true);
}
fn commit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    complete(scope, arguments.this(), false);
}

fn complete(
    scope: &mut v8::PinScope<'_, '_>,
    transaction: v8::Local<'_, v8::Object>,
    aborted: bool,
) {
    let id = transaction.get_identity_hash().get();
    let snapshot = record(scope, transaction);
    let Some(snapshot) = snapshot else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !snapshot.active {
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<IdbTransactionStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.active = false;
    }
    let handler = if aborted {
        snapshot.onabort
    } else {
        snapshot.oncomplete
    };
    let event_type = if aborted { "abort" } else { "complete" };
    let event = super::event_target::create_event(scope, event_type);
    if let Some(handler) = handler {
        let handler = v8::Local::new(scope, &handler);
        let _ = handler.call(scope, transaction.into(), &[event.into()]);
    }
    let _ = super::event_target::dispatch(scope, transaction, event);
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbTransactionRecord) -> Option<v8::Global<v8::Function>>,
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
    select: impl FnOnce(&mut IdbTransactionRecord) -> &mut Option<v8::Global<v8::Function>>,
) {
    let handler = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|value| v8::Global::new(scope, value));
    if let Some(record) = scope
        .get_slot_mut::<IdbTransactionStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *select(record) = handler;
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
fn get_oncomplete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |x| x.oncomplete.clone())
}
fn set_oncomplete(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x| &mut x.oncomplete)
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

fn throw_dom(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    if let Ok(exception) = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbTransactionStore>() {
        store.constructor.remove(realm_id);
    }
}
