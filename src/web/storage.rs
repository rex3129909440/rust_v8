use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct StorageStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, StorageAreaKey>,
    areas: HashMap<StorageAreaKey, StorageRecord>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum StorageKind {
    Local,
    Session,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct StorageAreaKey {
    kind: StorageKind,
    origin: String,
}
#[derive(Clone, Default)]
struct StorageRecord {
    order: Vec<String>,
    values: HashMap<String, String>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StorageStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Storage", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StorageStore>()
        .and_then(|s| s.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "Storage",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "length", get_length)?;
    crate::webidl::define_method(scope, p, "clear", 0, clear)?;
    crate::webidl::define_method(scope, p, "getItem", 1, get_item)?;
    crate::webidl::define_method(scope, p, "key", 1, key)?;
    crate::webidl::define_method(scope, p, "removeItem", 1, remove_item)?;
    crate::webidl::define_method(scope, p, "setItem", 2, set_item)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StorageStore>()
        .ok_or_else(|| "Storage state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}

pub(crate) fn create_local<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create(scope, StorageKind::Local)
}

pub(crate) fn create_session<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create(scope, StorageKind::Session)
}

fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: StorageKind,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let template = v8::ObjectTemplate::new(scope);
    template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(named_getter)
            .setter(named_setter)
            .query(named_query)
            .deleter(named_deleter)
            .enumerator(named_enumerator),
    );
    let o = template
        .new_instance(scope)
        .ok_or_else(|| "cannot create Storage exotic object".to_owned())?;
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create Storage".to_owned());
    }
    let window = scope.get_current_context().global(scope);
    let area = StorageAreaKey {
        kind,
        origin: super::html_i_frame_element::origin_for_window(scope, window),
    };
    let store = scope
        .get_slot_mut::<StorageStore>()
        .ok_or_else(|| "Storage state was not prepared".to_owned())?;
    store.areas.entry(area.clone()).or_default();
    store.records.insert(o.get_identity_hash().get(), area);
    Ok(o)
}
pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<StorageStore>()
        .is_some_and(|s| s.records.contains_key(&o.get_identity_hash().get()))
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Storage': Illegal constructor");
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<StorageRecord> {
    let store = scope.get_slot::<StorageStore>()?;
    let area = store.records.get(&o.get_identity_hash().get()).cloned()?;
    store.areas.get(&area).cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut StorageRecord),
) {
    let area = scope
        .get_slot::<StorageStore>()
        .and_then(|s| s.records.get(&o.get_identity_hash().get()))
        .cloned();
    if let Some(v) = area.and_then(|area| {
        scope
            .get_slot_mut::<StorageStore>()
            .and_then(|store| store.areas.get_mut(&area))
    }) {
        change(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, v.values.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |v| {
        v.order.clear();
        v.values.clear();
    })
}
fn get_item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(k) = storage_string(scope, a.get(0), "getItem") else {
        return;
    };
    if let Some(value) = v.values.get(&k).and_then(|v| v8::String::new(scope, v)) {
        r.set(value.into())
    } else {
        r.set(v8::null(scope).into())
    }
}
fn key(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let i = a.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(value) = v.order.get(i).and_then(|v| v8::String::new(scope, v)) {
        r.set(value.into())
    } else {
        r.set(v8::null(scope).into())
    }
}
fn remove_item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(k) = storage_string(scope, a.get(0), "removeItem") else {
        return;
    };
    update(scope, a.this(), |v| {
        v.values.remove(&k);
        v.order.retain(|x| x != &k);
    })
}
fn set_item(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.length() < 2 {
        crate::webidl::throw_type_error(scope, "setItem requires 2 arguments");
        return;
    }
    let Some(k) = storage_string(scope, a.get(0), "setItem") else {
        return;
    };
    let Some(value) = storage_string(scope, a.get(1), "setItem") else {
        return;
    };
    update(scope, a.this(), |v| {
        if !v.values.contains_key(&k) {
            v.order.insert(0, k.clone());
        }
        v.values.insert(k, value);
    })
}

fn storage_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    method: &str,
) -> Option<String> {
    if value.is_symbol() {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'Storage': Cannot convert a Symbol value to a string"
            ),
        );
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn property_name(scope: &mut v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    if key.is_symbol() {
        None
    } else {
        key.to_string(scope)
            .map(|key| key.to_rust_string_lossy(scope))
    }
}

fn named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "get", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = record(scope, arguments.holder())
        .and_then(|record| record.values.get(&name).cloned())
        .and_then(|value| v8::String::new(scope, &value))
    else {
        return v8::Intercepted::kNo;
    };
    result.set(value.into());
    v8::Intercepted::kYes
}

fn named_setter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "set", key, Some(value));
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    let value = crate::webidl::value_to_string(scope, value);
    update(scope, arguments.holder(), |record| {
        if !record.values.contains_key(&name) {
            record.order.insert(0, name.clone());
        }
        record.values.insert(name, value);
    });
    result.set_bool(true);
    v8::Intercepted::kYes
}

fn named_query(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "has", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if record(scope, arguments.holder()).is_some_and(|record| record.values.contains_key(&name)) {
        result.set_int32(0);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn named_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "delete", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    update(scope, arguments.holder(), |record| {
        record.values.remove(&name);
        record.order.retain(|candidate| candidate != &name);
    });
    result.set(v8::Boolean::new(scope, true));
    v8::Intercepted::kYes
}

fn named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let values = record(scope, arguments.holder())
        .map(|record| {
            record
                .order
                .iter()
                .filter_map(|name| v8::String::new(scope, name).map(|name| name.into()))
                .collect::<Vec<v8::Local<v8::Value>>>()
        })
        .unwrap_or_default();
    result.set(v8::Array::new_with_elements(scope, &values));
}
