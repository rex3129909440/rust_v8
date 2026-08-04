use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct KeyboardLayoutMapStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<crate::KeyboardLayoutEntryFingerprint>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(KeyboardLayoutMapStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "KeyboardLayoutMap", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<KeyboardLayoutMapStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "KeyboardLayoutMap",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "entries")?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<KeyboardLayoutMapStore>()
        .ok_or_else(|| "KeyboardLayoutMap state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create KeyboardLayoutMap".to_owned());
    }
    let values = crate::fingerprint::edge(scope)
        .hardware_devices
        .keyboard_layout
        .clone();
    scope
        .get_slot_mut::<KeyboardLayoutMapStore>()
        .ok_or_else(|| "KeyboardLayoutMap state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), values);
    Ok(object)
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<crate::KeyboardLayoutEntryFingerprint>> {
    scope
        .get_slot::<KeyboardLayoutMapStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn native_map<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: &[crate::KeyboardLayoutEntryFingerprint],
) -> Option<v8::Local<'s, v8::Map>> {
    let mut map = v8::Map::new(scope);
    for entry in values {
        let key = v8::String::new(scope, &entry.code)?;
        let value = v8::String::new(scope, &entry.value)?;
        map = map.set(scope, key.into(), value.into())?;
    }
    Some(map)
}
fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let key = crate::webidl::value_to_string(s, a.get(0));
        if let Some(value) = v
            .iter()
            .find(|entry| entry.code == key)
            .and_then(|entry| v8::String::new(s, &entry.value))
        {
            r.set(value.into())
        } else {
            r.set(v8::undefined(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn has(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let key = crate::webidl::value_to_string(s, a.get(0));
        r.set(v8::Boolean::new(s, v.iter().any(|entry| entry.code == key)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn iterator(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    name: &str,
) {
    let Some(values) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let Some(map) = native_map(s, &values) else {
        return;
    };
    let key = v8::String::new(s, name).unwrap();
    let map_object: v8::Local<'_, v8::Object> = map.into();
    let Some(method) = map_object.get(s, key.into()) else {
        return;
    };
    let Ok(method) = v8::Local::<v8::Function>::try_from(method) else {
        return;
    };
    if let Some(value) = method.call(s, map.into(), &[]) {
        r.set(value)
    }
}
fn entries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a, r, "entries")
}
fn keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a, r, "keys")
}
fn values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a, r, "values")
}
fn for_each(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(values) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "callback is not callable");
        return;
    };
    for entry in values {
        if let (Some(key), Some(value)) = (
            v8::String::new(s, &entry.code),
            v8::String::new(s, &entry.value),
        ) {
            let _ = callback.call(s, a.get(1), &[value.into(), key.into(), a.this().into()]);
        }
    }
}
