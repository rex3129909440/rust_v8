use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct StylePropertyMapReadOnlyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MapRecord>,
}
#[derive(Clone, Default)]
pub(crate) struct MapRecord {
    pub order: Vec<String>,
    pub values: HashMap<String, Vec<v8::Global<v8::Value>>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StylePropertyMapReadOnlyStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "StylePropertyMapReadOnly", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StylePropertyMapReadOnlyStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "StylePropertyMapReadOnly",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "size", get_size)?;
    crate::webidl::define_method(scope, p, "get", 1, get)?;
    crate::webidl::define_method(scope, p, "getAll", 1, get_all)?;
    crate::webidl::define_method(scope, p, "has", 1, has)?;
    crate::webidl::define_method(scope, p, "entries", 0, entries)?;
    crate::webidl::define_method(scope, p, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, p, "keys", 0, keys)?;
    crate::webidl::define_method(scope, p, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_iterator_alias(scope, p, "entries")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StylePropertyMapReadOnlyStore>()
        .ok_or_else(|| "StylePropertyMapReadOnly state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn attach(scope: &mut v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) {
    scope
        .get_slot_mut::<StylePropertyMapReadOnlyStore>()
        .expect("StylePropertyMapReadOnly state")
        .records
        .insert(o.get_identity_hash().get(), MapRecord::default());
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<MapRecord> {
    scope
        .get_slot::<StylePropertyMapReadOnlyStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut MapRecord),
) -> bool {
    if let Some(v) = scope
        .get_slot_mut::<StylePropertyMapReadOnlyStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v);
        true
    } else {
        false
    }
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'StylePropertyMapReadOnly': Illegal constructor",
    );
}
fn property(scope: &v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>) -> String {
    crate::webidl::value_to_string(scope, v)
        .trim()
        .to_ascii_lowercase()
}
fn get_size(
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
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = property(scope, a.get(0));
    if let Some(value) = v.values.get(&name).and_then(|v| v.first()) {
        r.set(v8::Local::new(scope, value))
    } else {
        r.set(v8::undefined(scope).into())
    }
}
fn values_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    values: &[v8::Global<v8::Value>],
) -> v8::Local<'s, v8::Array> {
    let a = v8::Array::new(scope, values.len() as i32);
    for (i, v) in values.iter().enumerate() {
        let v = v8::Local::new(scope, v);
        let _ = a.set_index(scope, i as u32, v);
    }
    a
}
fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = property(scope, a.get(0));
    let array = v
        .values
        .get(&name)
        .map(|v| values_array(scope, v))
        .unwrap_or_else(|| v8::Array::new(scope, 0));
    r.set(array.into())
}
fn has(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = property(scope, a.get(0));
    r.set(v8::Boolean::new(scope, v.values.contains_key(&name)).into())
}
fn iterator<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    array: v8::Local<'s, v8::Array>,
    method: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, method)?;
    let value = array.get(scope, key.into())?;
    let function = v8::Local::<v8::Function>::try_from(value).ok()?;
    function.call(scope, array.into(), &[])
}
fn keys_array<'s>(scope: &v8::PinScope<'s, '_>, v: &MapRecord) -> v8::Local<'s, v8::Array> {
    let a = v8::Array::new(scope, v.order.len() as i32);
    for (i, key) in v.order.iter().enumerate() {
        if let Some(key) = v8::String::new(scope, key) {
            let _ = a.set_index(scope, i as u32, key.into());
        }
    }
    a
}
fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let out = v8::Array::new(scope, v.order.len() as i32);
    for (i, key) in v.order.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        if let Some(k) = v8::String::new(scope, key) {
            let _ = pair.set_index(scope, 0, k.into());
        }
        let vals = v
            .values
            .get(key)
            .map(|x| values_array(scope, x))
            .unwrap_or_else(|| v8::Array::new(scope, 0));
        let _ = pair.set_index(scope, 1, vals.into());
        let _ = out.set_index(scope, i as u32, pair.into());
    }
    if let Some(v) = iterator(scope, out, "values") {
        r.set(v)
    }
}
fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let out = keys_array(scope, &v);
    if let Some(v) = iterator(scope, out, "values") {
        r.set(v)
    }
}
fn values(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let out = v8::Array::new(scope, v.order.len() as i32);
    for (i, key) in v.order.iter().enumerate() {
        let vals = v
            .values
            .get(key)
            .map(|x| values_array(scope, x))
            .unwrap_or_else(|| v8::Array::new(scope, 0));
        let _ = out.set_index(scope, i as u32, vals.into());
    }
    if let Some(v) = iterator(scope, out, "values") {
        r.set(v)
    }
}
fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "forEach requires a function");
        return;
    };
    let receiver = if a.length() > 1 {
        a.get(1)
    } else {
        v8::undefined(scope).into()
    };
    for key in &v.order {
        let vals = v
            .values
            .get(key)
            .map(|x| values_array(scope, x))
            .unwrap_or_else(|| v8::Array::new(scope, 0));
        let Some(name) = v8::String::new(scope, key) else {
            continue;
        };
        let _ = callback.call(
            scope,
            receiver,
            &[vals.into(), name.into(), a.this().into()],
        );
    }
}
