use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct CookieStoreManagerStore {
    constructor: crate::webidl::RealmConstructor,
    subscriptions: HashMap<i32, HashSet<String>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CookieStoreManagerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CookieStoreManager", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CookieStoreManagerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CookieStoreManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getSubscriptions", 0, get_subscriptions)?;
    crate::webidl::define_method(scope, prototype, "subscribe", 1, subscribe)?;
    crate::webidl::define_method(scope, prototype, "unsubscribe", 1, unsubscribe)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<CookieStoreManagerStore>()
        .ok_or_else(|| "CookieStoreManager state was not prepared".to_owned())?
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
        return Err("cannot create CookieStoreManager".to_owned());
    }
    scope
        .get_slot_mut::<CookieStoreManagerStore>()
        .ok_or_else(|| "CookieStoreManager state was not prepared".to_owned())?
        .subscriptions
        .insert(object.get_identity_hash().get(), HashSet::new());
    Ok(object)
}

fn registration_key(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> String {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return String::new();
    };
    let name = crate::webidl::string(scope, "name")
        .ok()
        .and_then(|key| object.get(scope, key.into()))
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let url = crate::webidl::string(scope, "url")
        .ok()
        .and_then(|key| object.get(scope, key.into()))
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    format!("{name}\u{0}{url}")
}

fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

fn subscribe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Ok(sequence) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "CookieStoreManager.subscribe requires a sequence");
        return;
    };
    let mut values = Vec::new();
    for index in 0..sequence.length() {
        if let Some(value) = sequence.get_index(scope, index) {
            values.push(registration_key(scope, value));
        }
    }
    let Some(subscriptions) = scope
        .get_slot_mut::<CookieStoreManagerStore>()
        .and_then(|store| {
            store
                .subscriptions
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    subscriptions.extend(values);
    resolve(scope, v8::undefined(scope).into(), result);
}

fn unsubscribe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Ok(sequence) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "CookieStoreManager.unsubscribe requires a sequence",
        );
        return;
    };
    let mut values = Vec::new();
    for index in 0..sequence.length() {
        if let Some(value) = sequence.get_index(scope, index) {
            values.push(registration_key(scope, value));
        }
    }
    let Some(subscriptions) = scope
        .get_slot_mut::<CookieStoreManagerStore>()
        .and_then(|store| {
            store
                .subscriptions
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    for value in values {
        subscriptions.remove(&value);
    }
    resolve(scope, v8::undefined(scope).into(), result);
}

fn get_subscriptions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(subscriptions) = scope
        .get_slot::<CookieStoreManagerStore>()
        .and_then(|store| {
            store
                .subscriptions
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, subscriptions.len() as i32);
    for (index, encoded) in subscriptions.iter().enumerate() {
        let object = v8::Object::new(scope);
        let (name, url) = encoded.split_once('\0').unwrap_or((encoded, ""));
        if let (Some(name_key), Some(name_value)) =
            (v8::String::new(scope, "name"), v8::String::new(scope, name))
        {
            let _ = object.set(scope, name_key.into(), name_value.into());
        }
        if let (Some(url_key), Some(url_value)) =
            (v8::String::new(scope, "url"), v8::String::new(scope, url))
        {
            let _ = object.set(scope, url_key.into(), url_value.into());
        }
        let _ = array.set_index(scope, index as u32, object.into());
    }
    resolve(scope, array.into(), result);
}
