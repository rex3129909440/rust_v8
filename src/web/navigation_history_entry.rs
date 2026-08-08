use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ENTRY_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
pub(crate) struct NavigationHistoryEntryStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationHistoryEntryRecord>,
}

#[derive(Clone)]
struct NavigationHistoryEntryRecord {
    key: String,
    id: String,
    url: String,
    index: i32,
    same_document: bool,
    ondispose: Option<v8::Global<v8::Value>>,
    state: v8::Global<v8::Value>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationHistoryEntryStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigationHistoryEntry", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigationHistoryEntryStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let c = crate::webidl::create_function(
        scope,
        "NavigationHistoryEntry",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "key", get_key)?;
    crate::webidl::define_readonly_accessor(scope, p, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, p, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, p, "index", get_index)?;
    crate::webidl::define_readonly_accessor(scope, p, "sameDocument", get_same_document)?;
    crate::webidl::define_accessor(scope, p, "ondispose", get_ondispose, set_ondispose)?;
    crate::webidl::define_method(scope, p, "getState", 0, get_state)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NavigationHistoryEntryStore>()
        .ok_or_else(|| "NavigationHistoryEntry state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    url: String,
    index: i32,
    same_document: bool,
    state: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create NavigationHistoryEntry".to_owned());
    }
    super::event_target::attach(scope, object);
    let sequence = NEXT_ENTRY_ID.fetch_add(1, Ordering::Relaxed);
    let state = super::performance_mark::clone_value(scope, state);
    let state = v8::Global::new(scope, state);
    scope
        .get_slot_mut::<NavigationHistoryEntryStore>()
        .ok_or_else(|| "NavigationHistoryEntry state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            NavigationHistoryEntryRecord {
                key: format!("entry-key-{sequence}"),
                id: format!("entry-id-{sequence}"),
                url,
                index,
                same_document,
                ondispose: None,
                state,
            },
        );
    Ok(object)
}
pub(crate) fn is_entry(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<NavigationHistoryEntryStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}
pub(crate) fn key(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.key)
}
pub(crate) fn id(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.id)
}
pub(crate) fn url(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.url)
}
pub(crate) fn index(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    record(scope, object).map(|record| record.index)
}
pub(crate) fn state<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Value>> {
    let record = record(scope, object)?;
    let value = v8::Local::new(scope, &record.state);
    Some(super::performance_mark::clone_value(scope, value))
}
pub(crate) fn set_index(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    index: i32,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<NavigationHistoryEntryStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.index = index;
    true
}
pub(crate) fn replace_state(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    state: v8::Local<'_, v8::Value>,
) -> bool {
    let state = super::performance_mark::clone_value(scope, state);
    let state = v8::Global::new(scope, state);
    let Some(record) = scope
        .get_slot_mut::<NavigationHistoryEntryStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.state = state;
    true
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NavigationHistoryEntry': Illegal constructor",
    )
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationHistoryEntryRecord> {
    scope
        .get_slot::<NavigationHistoryEntryStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigationHistoryEntryRecord) -> &str,
) {
    if let Some(v) = record(scope, a.this()) {
        if let Some(s) = v8::String::new(scope, select(&v)) {
            r.set(s.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.key)
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.id)
}
fn get_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.url)
}
fn get_index(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new(scope, v.index).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_same_document(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.same_document).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_ondispose(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(scope, a.this()) {
        Some(v) => match v.ondispose {
            Some(h) => r.set(v8::Local::new(scope, &h)),
            None => r.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
fn set_ondispose(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if a.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, a.get(0)))
    };
    if let Some(v) = scope
        .get_slot_mut::<NavigationHistoryEntryStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.ondispose = value
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        let value = v8::Local::new(scope, &v.state);
        r.set(super::performance_mark::clone_value(scope, value))
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
