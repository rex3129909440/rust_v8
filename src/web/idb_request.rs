use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdbRequestStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbRequestRecord>,
}

#[derive(Clone)]
struct IdbRequestRecord {
    result: v8::Global<v8::Value>,
    error: Option<v8::Global<v8::Object>>,
    source: Option<v8::Global<v8::Value>>,
    transaction: Option<v8::Global<v8::Object>>,
    done: bool,
    settle_on_handler: bool,
    onsuccess: Option<v8::Global<v8::Function>>,
    onerror: Option<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbRequestStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBRequest", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbRequestStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBRequest",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "result", get_result)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "error", get_error)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "source", get_source)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transaction", get_transaction)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readyState", get_ready_state)?;
    crate::webidl::define_accessor(scope, prototype, "onsuccess", get_onsuccess, set_onsuccess)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbRequestStore>()
        .ok_or_else(|| "IDBRequest state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_success<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: Option<v8::Local<'_, v8::Value>>,
    transaction: Option<v8::Local<'_, v8::Object>>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBRequest".to_owned());
    }
    attach(scope, object, source, transaction, value, None, false, true)?;
    Ok(object)
}

pub(crate) fn create_error<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: Option<v8::Local<'_, v8::Value>>,
    transaction: Option<v8::Local<'_, v8::Object>>,
    error: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBRequest".to_owned());
    }
    attach(
        scope,
        object,
        source,
        transaction,
        v8::undefined(scope).into(),
        Some(error),
        false,
        true,
    )?;
    Ok(object)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    source: Option<v8::Local<'_, v8::Value>>,
    transaction: Option<v8::Local<'_, v8::Object>>,
    result: v8::Local<'_, v8::Value>,
    error: Option<v8::Local<'_, v8::Object>>,
    done: bool,
    settle_on_handler: bool,
) -> Result<(), String> {
    super::event_target::attach(scope, object);
    let result = v8::Global::new(scope, result);
    let error = error.map(|value| v8::Global::new(scope, value));
    let source = source.map(|value| v8::Global::new(scope, value));
    let transaction = transaction.map(|value| v8::Global::new(scope, value));
    scope
        .get_slot_mut::<IdbRequestStore>()
        .ok_or_else(|| "IDBRequest state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdbRequestRecord {
                result,
                error,
                source,
                transaction,
                done,
                settle_on_handler,
                onsuccess: None,
                onerror: None,
            },
        );
    Ok(())
}

pub(crate) fn set_result(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    done: bool,
) {
    let value = v8::Global::new(scope, value);
    if let Some(record) = scope
        .get_slot_mut::<IdbRequestStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.result = value;
        record.error = None;
        record.done = done;
    }
}

pub(crate) fn mark_done(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    if let Some(record) = scope
        .get_slot_mut::<IdbRequestStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.done = true;
        record.settle_on_handler = false;
    }
}

pub(crate) fn clear_transaction(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = scope
        .get_slot_mut::<IdbRequestStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.transaction = None;
    }
}

pub(crate) fn set_transaction(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    transaction: v8::Local<'_, v8::Object>,
) {
    let transaction = v8::Global::new(scope, transaction);
    if let Some(record) = scope
        .get_slot_mut::<IdbRequestStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.transaction = Some(transaction);
    }
}

pub(crate) fn fire_success(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    mark_done(scope, object);
    let handler = scope
        .get_slot::<IdbRequestStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .and_then(|record| record.onsuccess.clone());
    fire(scope, object, "success", handler);
}

pub(crate) fn fire_error(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    mark_done(scope, object);
    let handler = scope
        .get_slot::<IdbRequestStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .and_then(|record| record.onerror.clone());
    fire(scope, object, "error", handler);
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbRequestRecord> {
    scope
        .get_slot::<IdbRequestStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn invalid_state(scope: &mut v8::PinScope<'_, '_>) {
    if let Ok(exception) = super::dom_exception::create(
        scope,
        "The request has not finished.".to_owned(),
        "InvalidStateError".to_owned(),
    ) {
        scope.throw_exception(exception.into());
    }
}

fn get_result(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.done {
        invalid_state(scope);
        return;
    }
    result.set(v8::Local::new(scope, &record.result));
}

fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.done {
        invalid_state(scope);
        return;
    }
    match record.error {
        Some(error) => result.set(v8::Local::new(scope, &error).into()),
        None => result.set(v8::null(scope).into()),
    }
}

fn get_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.source {
            Some(source) => result.set(v8::Local::new(scope, &source)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_transaction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.transaction {
            Some(transaction) => result.set(v8::Local::new(scope, &transaction).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_ready_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let state = if record.done { "done" } else { "pending" };
        if let Some(state) = v8::String::new(scope, state) {
            result.set(state.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbRequestRecord) -> Option<v8::Global<v8::Function>>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match select(&record) {
            Some(handler) => result.set(v8::Local::new(scope, &handler).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_onsuccess(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |record| record.onsuccess.clone())
}
fn get_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |record| record.onerror.clone())
}

fn handler_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Function>> {
    v8::Local::<v8::Function>::try_from(value)
        .ok()
        .map(|function| v8::Global::new(scope, function))
}

fn set_onsuccess(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let handler = handler_value(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<IdbRequestStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.onsuccess = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::idb_open_db_request::before_success_handler(scope, arguments.this());
    let snapshot = record(scope, arguments.this());
    if let Some(snapshot) = snapshot {
        if snapshot.settle_on_handler {
            mark_done(scope, arguments.this());
        }
        if snapshot.error.is_none() {
            let handler = record(scope, arguments.this()).and_then(|value| value.onsuccess);
            fire(scope, arguments.this(), "success", handler);
        }
    }
}

fn set_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let handler = handler_value(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<IdbRequestStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.onerror = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let snapshot = record(scope, arguments.this());
    if let Some(snapshot) = snapshot {
        if snapshot.settle_on_handler {
            mark_done(scope, arguments.this());
        }
        if snapshot.error.is_some() {
            let handler = record(scope, arguments.this()).and_then(|value| value.onerror);
            fire(scope, arguments.this(), "error", handler);
        }
    }
}

fn fire(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event_type: &str,
    handler: Option<v8::Global<v8::Function>>,
) {
    let event = super::event_target::create_event(scope, event_type);
    if let Some(handler) = handler {
        let handler = v8::Local::new(scope, &handler);
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
    let _ = super::event_target::dispatch(scope, target, event);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbRequestStore>() {
        store.constructor.remove(realm_id);
    }
}
