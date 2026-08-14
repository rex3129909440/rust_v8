use std::collections::HashMap;

#[derive(Clone)]
struct CacheEntry {
    request: String,
    response: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CacheStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, Vec<CacheEntry>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CacheStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Cache", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CacheStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Cache",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "add", 1, add)?;
    crate::webidl::define_method(scope, prototype, "addAll", 1, add_all)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "match", 1, match_one)?;
    crate::webidl::define_method(scope, prototype, "matchAll", 0, match_all)?;
    crate::webidl::define_method(scope, prototype, "put", 2, put)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CacheStore>()
        .ok_or_else(|| "Cache state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Cache': Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Cache".to_owned());
    }
    scope
        .get_slot_mut::<CacheStore>()
        .ok_or_else(|| "Cache state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<CacheStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

fn request_text(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> String {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value)
        && let Some(key) = v8::String::new(scope, "url")
        && let Some(url) = object.get(scope, key.into())
        && !url.is_undefined()
    {
        return crate::webidl::value_to_string(scope, url);
    }
    crate::webidl::value_to_string(scope, value)
}

pub(crate) fn request_text_for_storage(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> String {
    request_text(scope, value)
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

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Cache", "add", result);
        return;
    }
    let request = request_text(scope, arguments.get(0));
    let Ok(response) = super::response::create_fetch_response(
        scope,
        request.clone(),
        200,
        "OK".to_owned(),
        Vec::new(),
        Vec::new(),
    ) else {
        return;
    };
    insert(scope, arguments.this(), request, response);
    resolve(scope, v8::undefined(scope).into(), result);
}

fn add_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Cache", "addAll", result);
        return;
    }
    let Ok(values) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Cache.addAll requires a sequence");
        return;
    };
    for index in 0..values.length() {
        let Some(value) = values.get_index(scope, index) else {
            continue;
        };
        let request = request_text(scope, value);
        if let Ok(response) = super::response::create_fetch_response(
            scope,
            request.clone(),
            200,
            "OK".to_owned(),
            Vec::new(),
            Vec::new(),
        ) {
            insert(scope, arguments.this(), request, response);
        }
    }
    resolve(scope, v8::undefined(scope).into(), result);
}

fn insert(
    scope: &mut v8::PinScope<'_, '_>,
    cache: v8::Local<'_, v8::Object>,
    request: String,
    response: v8::Local<'_, v8::Object>,
) {
    let response = v8::Global::new(scope, response);
    if let Some(entries) = scope
        .get_slot_mut::<CacheStore>()
        .and_then(|store| store.records.get_mut(&cache.get_identity_hash().get()))
    {
        entries.retain(|entry| entry.request != request);
        entries.push(CacheEntry { request, response });
    }
}

fn put(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Cache", "put", result);
        return;
    }
    let Ok(response) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "Cache.put requires a Response");
        return;
    };
    let request = request_text(scope, arguments.get(0));
    insert(scope, arguments.this(), request, response);
    resolve(scope, v8::undefined(scope).into(), result);
}

fn match_one(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Cache", "match", result);
        return;
    }
    let request = request_text(scope, arguments.get(0));
    let response = scope
        .get_slot::<CacheStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .and_then(|entries| entries.iter().find(|entry| entry.request == request))
        .map(|entry| entry.response.clone());
    match response {
        Some(response) => {
            let response = v8::Local::new(scope, &response);
            resolve(scope, response.into(), result);
        }
        None => resolve(scope, v8::undefined(scope).into(), result),
    }
}

fn match_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Cache", "matchAll", result);
        return;
    }
    let requested =
        (!arguments.get(0).is_undefined()).then(|| request_text(scope, arguments.get(0)));
    let responses = scope
        .get_slot::<CacheStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|entries| {
            entries
                .iter()
                .filter(|entry| {
                    requested
                        .as_ref()
                        .is_none_or(|request| entry.request == *request)
                })
                .map(|entry| entry.response.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let array = v8::Array::new(scope, responses.len() as i32);
    for (index, response) in responses.iter().enumerate() {
        let response = v8::Local::new(scope, response);
        let _ = array.set_index(scope, index as u32, response.into());
    }
    resolve(scope, array.into(), result);
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Cache", "delete", result);
        return;
    }
    let request = request_text(scope, arguments.get(0));
    let removed = scope
        .get_slot_mut::<CacheStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
        .is_some_and(|entries| {
            let before = entries.len();
            entries.retain(|entry| entry.request != request);
            entries.len() != before
        });
    resolve(scope, v8::Boolean::new(scope, removed).into(), result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "Cache", "keys", result);
        return;
    }
    let requests = scope
        .get_slot::<CacheStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|entries| {
            entries
                .iter()
                .map(|entry| entry.request.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let array = v8::Array::new(scope, requests.len() as i32);
    for (index, request) in requests.iter().enumerate() {
        if let Ok(object) = super::request::create_from_input(
            scope,
            v8::String::new(scope, request).unwrap().into(),
        ) {
            let _ = array.set_index(scope, index as u32, object.into());
        }
    }
    resolve(scope, array.into(), result);
}

pub(crate) fn find<'s>(
    scope: &v8::PinScope<'s, '_>,
    cache: v8::Local<'_, v8::Object>,
    request: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let response = scope
        .get_slot::<CacheStore>()?
        .records
        .get(&cache.get_identity_hash().get())?
        .iter()
        .find(|entry| entry.request == request)?
        .response
        .clone();
    Some(v8::Local::new(scope, &response))
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CacheStore>() {
        store.constructors.remove(&realm_id);
    }
}
