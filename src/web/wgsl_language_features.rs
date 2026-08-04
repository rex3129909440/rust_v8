use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct WgslLanguageFeaturesStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, HashSet<String>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WgslLanguageFeaturesStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WGSLLanguageFeatures", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<WgslLanguageFeaturesStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WGSLLanguageFeatures",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "values")?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WgslLanguageFeaturesStore>()
        .ok_or_else(|| "WGSLLanguageFeatures state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create WGSLLanguageFeatures".to_owned());
    }
    scope
        .get_slot_mut::<WgslLanguageFeaturesStore>()
        .ok_or_else(|| "WGSLLanguageFeatures state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            [
                "readonly_and_readwrite_storage_textures",
                "packed_4x8_integer_dot_product",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<HashSet<String>> {
    scope
        .get_slot::<WgslLanguageFeaturesStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn native_set<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: &HashSet<String>,
) -> Option<v8::Local<'s, v8::Set>> {
    let mut set = v8::Set::new(scope);
    for value in values {
        let text = v8::String::new(scope, value)?;
        set = set.add(scope, text.into())?;
    }
    Some(set)
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
fn has(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let k = crate::webidl::value_to_string(s, a.get(0));
        r.set(v8::Boolean::new(s, v.contains(&k)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn iter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    entry: bool,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let Some(set) = native_set(s, &v) else { return };
    let key = v8::String::new(s, if entry { "entries" } else { "values" }).unwrap();
    let Some(m) = set.get(s, key.into()) else {
        return;
    };
    let Ok(m) = v8::Local::<v8::Function>::try_from(m) else {
        return;
    };
    if let Some(v) = m.call(s, set.into(), &[]) {
        r.set(v)
    }
}
fn entries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iter(s, a, r, true)
}
fn keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iter(s, a, r, false)
}
fn values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iter(s, a, r, false)
}
fn for_each(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let Ok(f) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "callback is not callable");
        return;
    };
    for x in v {
        if let Some(t) = v8::String::new(s, &x) {
            let _ = f.call(s, a.get(1), &[t.into(), t.into(), a.this().into()]);
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WgslLanguageFeaturesStore>() {
        store.constructor.remove(realm_id);
    }
}
