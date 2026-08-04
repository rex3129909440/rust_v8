use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct StorageEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) key: Option<String>,
    pub(crate) old_value: Option<String>,
    pub(crate) new_value: Option<String>,
    pub(crate) url: String,
    pub(crate) storage_area: Option<v8::Global<v8::Object>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StorageEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "StorageEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StorageEventStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "StorageEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::storage_event_key_property::define(scope, p)?;
    super::storage_event_old_value_property::define(scope, p)?;
    super::storage_event_new_value_property::define(scope, p)?;
    super::storage_event_url_property::define(scope, p)?;
    super::storage_event_storage_area_property::define(scope, p)?;
    super::storage_event_init_storage_event::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StorageEventStore>()
        .ok_or_else(|| "StorageEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let event_type = crate::webidl::string(scope, "")?;
    constructor
        .new_instance(scope, &[event_type.into()])
        .ok_or_else(|| "cannot create StorageEvent".to_owned())
}

pub(crate) fn optional_string(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> Option<String> {
    let k = v8::String::new(scope, n)?;
    let v = o.get(scope, k.into())?;
    if v.is_null() || v.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, v))
    }
}
pub(crate) fn object<'s>(
    scope: &v8::PinScope<'s, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let k = v8::String::new(scope, n)?;
    let v = o.get(scope, k.into())?;
    v8::Local::<v8::Object>::try_from(v).ok()
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(scope, "StorageEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, a.get(0));
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let key = init.and_then(|o| optional_string(scope, o, "key"));
    let old_value = init.and_then(|o| optional_string(scope, o, "oldValue"));
    let new_value = init.and_then(|o| optional_string(scope, o, "newValue"));
    let url = init
        .and_then(|o| optional_string(scope, o, "url"))
        .unwrap_or_default();
    let storage = init.and_then(|o| object(scope, o, "storageArea"));
    if storage.is_some_and(|o| !super::storage::is_instance(scope, o)) {
        crate::webidl::throw_type_error(scope, "storageArea must be a Storage object");
        return;
    }
    let storage_area = storage.map(|o| v8::Global::new(scope, o));
    let bubbles = init.is_some_and(|o| super::event::boolean_property(scope, o, "bubbles"));
    let cancelable = init.is_some_and(|o| super::event::boolean_property(scope, o, "cancelable"));
    super::event::attach(scope, a.this(), event_type, bubbles, cancelable, false);
    scope
        .get_slot_mut::<StorageEventStore>()
        .expect("StorageEvent state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            Record {
                key,
                old_value,
                new_value,
                url,
                storage_area,
            },
        );
    r.set(a.this().into())
}
pub(crate) fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<StorageEventStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn nullable(scope: &v8::PinScope<'_, '_>, v: Option<&str>, r: &mut v8::ReturnValue<'_>) {
    if let Some(v) = v.and_then(|v| v8::String::new(scope, v)) {
        r.set(v.into())
    } else {
        r.set(v8::null(scope).into())
    }
}
pub(crate) fn get_key(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    nullable(scope, v.key.as_deref(), &mut r)
}
pub(crate) fn get_old_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    nullable(scope, v.old_value.as_deref(), &mut r)
}
pub(crate) fn get_new_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    nullable(scope, v.new_value.as_deref(), &mut r)
}
pub(crate) fn get_url(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, &v.url) {
        r.set(s.into())
    }
}
pub(crate) fn get_storage_area(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(o) = v.storage_area {
        r.set(v8::Local::new(scope, &o).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
pub(crate) fn init_storage_event(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let key = if a.get(4).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(4)))
    };
    let old_value = if a.get(5).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(5)))
    };
    let new_value = if a.get(6).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(6)))
    };
    let url = crate::webidl::value_to_string(scope, a.get(7));
    let Some(v) = scope
        .get_slot_mut::<StorageEventStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    v.key = key;
    v.old_value = old_value;
    v.new_value = new_value;
    v.url = url;
}
