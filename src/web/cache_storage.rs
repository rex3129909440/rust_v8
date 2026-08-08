use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CacheStorageStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashMap<i32, HashMap<String, v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CacheStorageStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CacheStorage", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CacheStorageStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CacheStorage",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "match", 1, match_one)?;
    crate::webidl::define_method(scope, prototype, "open", 1, open)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CacheStorageStore>()
        .ok_or_else(|| "CacheStorage state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CacheStorage': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CacheStorage".to_owned());
    }
    scope
        .get_slot_mut::<CacheStorageStore>()
        .ok_or_else(|| "CacheStorage state was not prepared".to_owned())?
        .instances
        .insert(object.get_identity_hash().get(), HashMap::new());
    Ok(object)
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

fn name(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> String {
    crate::webidl::value_to_string(scope, value)
}

fn open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let name = name(scope, arguments.get(0));
    let existing = scope
        .get_slot::<CacheStorageStore>()
        .and_then(|store| store.instances.get(&identity))
        .and_then(|caches| caches.get(&name))
        .cloned();
    let cache = match existing {
        Some(cache) => v8::Local::new(scope, &cache),
        None => {
            let Ok(cache) = super::cache::create(scope) else {
                return;
            };
            let persistent = v8::Global::new(scope, cache);
            if let Some(caches) = scope
                .get_slot_mut::<CacheStorageStore>()
                .and_then(|store| store.instances.get_mut(&identity))
            {
                caches.insert(name, persistent);
            } else {
                crate::webidl::throw_type_error(scope, "Illegal invocation");
                return;
            }
            cache
        }
    };
    resolve(scope, cache.into(), result);
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let name = name(scope, arguments.get(0));
    let present = scope
        .get_slot::<CacheStorageStore>()
        .and_then(|store| {
            store
                .instances
                .get(&arguments.this().get_identity_hash().get())
        })
        .is_some_and(|caches| caches.contains_key(&name));
    resolve(scope, v8::Boolean::new(scope, present).into(), result);
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let name = name(scope, arguments.get(0));
    let removed = scope
        .get_slot_mut::<CacheStorageStore>()
        .and_then(|store| {
            store
                .instances
                .get_mut(&arguments.this().get_identity_hash().get())
        })
        .and_then(|caches| caches.remove(&name))
        .is_some();
    resolve(scope, v8::Boolean::new(scope, removed).into(), result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let names = scope
        .get_slot::<CacheStorageStore>()
        .and_then(|store| {
            store
                .instances
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|caches| caches.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let array = v8::Array::new(scope, names.len() as i32);
    for (index, name) in names.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, name) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    resolve(scope, array.into(), result);
}

fn match_one(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let request = super::cache::request_text_for_storage(scope, arguments.get(0));
    let caches = scope
        .get_slot::<CacheStorageStore>()
        .and_then(|store| {
            store
                .instances
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|caches| caches.values().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    for cache in caches {
        let cache = v8::Local::new(scope, &cache);
        if let Some(response) = super::cache::find(scope, cache, &request) {
            resolve(scope, response.into(), result);
            return;
        }
    }
    resolve(scope, v8::undefined(scope).into(), result);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CacheStorageStore>() {
        store.constructors.remove(&realm_id);
    }
}
