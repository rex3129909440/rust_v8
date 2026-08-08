use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdbVersionChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, IdbVersionChangeEventRecord>,
}

#[derive(Clone)]
pub(crate) struct IdbVersionChangeEventRecord {
    pub(crate) old_version: u64,
    pub(crate) new_version: Option<u64>,
    pub(crate) data_loss: String,
    pub(crate) data_loss_message: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbVersionChangeEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBVersionChangeEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbVersionChangeEventStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBVersionChangeEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::idb_version_change_event_old_version_property::define(scope, prototype)?;
    super::idb_version_change_event_new_version_property::define(scope, prototype)?;
    super::idb_version_change_event_data_loss_property::define(scope, prototype)?;
    super::idb_version_change_event_data_loss_message_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbVersionChangeEventStore>()
        .ok_or_else(|| "IDBVersionChangeEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'IDBVersionChangeEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let old_version = init
        .map(|value| super::event::number_property(scope, value, "oldVersion", 0.0) as u64)
        .unwrap_or(0);
    let new_version = init.and_then(|value| optional_u64(scope, value, "newVersion"));
    let bubbles = init.is_some_and(|value| super::event::boolean_property(scope, value, "bubbles"));
    let cancelable =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "cancelable"));
    let composed =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "composed"));
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        bubbles,
        cancelable,
        composed,
    );
    attach(
        scope,
        arguments.this(),
        old_version,
        new_version,
        "none",
        "",
    );
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    old_version: u64,
    new_version: Option<u64>,
    data_loss: &str,
    data_loss_message: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBVersionChangeEvent".to_owned());
    }
    super::event::attach(scope, object, event_type.to_owned(), false, false, false);
    attach(
        scope,
        object,
        old_version,
        new_version,
        data_loss,
        data_loss_message,
    );
    Ok(object)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    old_version: u64,
    new_version: Option<u64>,
    data_loss: &str,
    data_loss_message: &str,
) {
    if let Some(store) = scope.get_slot_mut::<IdbVersionChangeEventStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            IdbVersionChangeEventRecord {
                old_version,
                new_version,
                data_loss: data_loss.to_owned(),
                data_loss_message: data_loss_message.to_owned(),
            },
        );
    }
}

pub(crate) fn optional_u64(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u64> {
    let value = object.get(scope, v8::String::new(scope, name)?.into())?;
    if value.is_null_or_undefined() {
        None
    } else {
        value.number_value(scope).map(|value| value as u64)
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbVersionChangeEventRecord> {
    scope
        .get_slot::<IdbVersionChangeEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_old_version(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.old_version as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_new_version(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.new_version {
            Some(value) => result.set(v8::Number::new(scope, value as f64).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbVersionChangeEventRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_data_loss(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.data_loss)
}
pub(crate) fn get_data_loss_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.data_loss_message)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbVersionChangeEventStore>() {
        store.constructor.remove(realm_id);
    }
}
