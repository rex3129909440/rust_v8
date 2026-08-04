use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IdbRecordStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbRecordData>,
}

#[derive(Clone)]
struct IdbRecordData {
    key: v8::Global<v8::Value>,
    primary_key: v8::Global<v8::Value>,
    value: v8::Global<v8::Value>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbRecordStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBRecord", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbRecordStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBRecord",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "key", get_key)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "primaryKey", get_primary_key)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "value", get_value)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbRecordStore>()
        .ok_or_else(|| "IDBRecord state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    key: v8::Local<'_, v8::Value>,
    primary_key: v8::Local<'_, v8::Value>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBRecord".to_owned());
    }
    let key = v8::Global::new(scope, key);
    let primary_key = v8::Global::new(scope, primary_key);
    let value = v8::Global::new(scope, value);
    scope
        .get_slot_mut::<IdbRecordStore>()
        .ok_or_else(|| "IDBRecord state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdbRecordData {
                key,
                primary_key,
                value,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbRecordData> {
    scope
        .get_slot::<IdbRecordStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbRecordData) -> v8::Global<v8::Value>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &select(&record)));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| record.key.clone())
}
fn get_primary_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| record.primary_key.clone())
}
fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| record.value.clone())
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbRecordStore>() {
        store.constructor.remove(realm_id);
    }
}
