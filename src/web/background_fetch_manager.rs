use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct BackgroundFetchManagerStore {
    constructor: crate::webidl::RealmConstructor,
    native_objects: HashSet<i32>,
    registrations: HashMap<String, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BackgroundFetchManagerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BackgroundFetchManager", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<BackgroundFetchManagerStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BackgroundFetchManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "fetch", 2, fetch)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getIds", 0, get_ids)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BackgroundFetchManagerStore>()
        .ok_or_else(|| "BackgroundFetchManager state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let manager = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, manager, prototype.into()) != Some(true) {
        return Err("cannot create BackgroundFetchManager".to_owned());
    }
    scope
        .get_slot_mut::<BackgroundFetchManagerStore>()
        .ok_or_else(|| "BackgroundFetchManager state was not prepared".to_owned())?
        .native_objects
        .insert(manager.get_identity_hash().get());
    Ok(manager)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'BackgroundFetchManager': Illegal constructor",
    );
}

fn valid_this(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<BackgroundFetchManagerStore>()
        .is_some_and(|store| {
            store
                .native_objects
                .contains(&object.get_identity_hash().get())
        })
}

fn fetch(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_this(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "BackgroundFetchManager",
            "fetch",
            result,
        );
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'fetch' on 'BackgroundFetchManager': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let id = crate::webidl::value_to_string(scope, arguments.get(0));
    if id.is_empty() {
        crate::webidl::throw_type_error(scope, "Background fetch id must not be empty");
        return;
    }
    let inputs = match request_inputs(scope, arguments.get(1)) {
        Ok(inputs) => inputs,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let mut records = Vec::new();
    for input in inputs {
        let Ok(request) = super::request::create_from_input(scope, input) else {
            return;
        };
        let url = super::request::url(scope, request).unwrap_or_default();
        let Ok(response) = super::response::create_fetch_response(
            scope,
            url,
            200,
            "OK".to_owned(),
            Vec::new(),
            Vec::new(),
        ) else {
            return;
        };
        let Ok(record) = super::background_fetch_record::create(scope, request, response) else {
            return;
        };
        records.push(record);
    }
    let download_total = option_number(scope, arguments.get(2), "downloadTotal").unwrap_or(0.0);
    let Ok(registration) =
        super::background_fetch_registration::create(scope, id.clone(), records, download_total)
    else {
        return;
    };
    let registration_global = v8::Global::new(scope, registration);
    if let Some(store) = scope.get_slot_mut::<BackgroundFetchManagerStore>() {
        store.registrations.insert(id, registration_global);
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, registration.into()) {
        result.set(promise.into());
    }
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_this(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "BackgroundFetchManager",
            "get",
            result,
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'get' on 'BackgroundFetchManager': 1 argument required, but only 0 present.",
        );
        return;
    }
    let id = crate::webidl::value_to_string(scope, arguments.get(0));
    let registration = scope
        .get_slot::<BackgroundFetchManagerStore>()
        .and_then(|store| store.registrations.get(&id))
        .cloned();
    let value = registration
        .map(|value| v8::Local::new(scope, &value).into())
        .unwrap_or_else(|| v8::undefined(scope).into());
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

fn get_ids(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_this(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "BackgroundFetchManager",
            "getIds",
            result,
        );
        return;
    }
    let ids = scope
        .get_slot::<BackgroundFetchManagerStore>()
        .map(|store| store.registrations.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let values = v8::Array::new(scope, ids.len() as i32);
    for (index, id) in ids.iter().enumerate() {
        if let Some(id) = v8::String::new(scope, id) {
            let _ = values.set_index(scope, index as u32, id.into());
        }
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, values.into()) {
        result.set(promise.into());
    }
}

fn request_inputs<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> Result<Vec<v8::Local<'s, v8::Value>>, String> {
    if let Ok(array) = v8::Local::<v8::Array>::try_from(value) {
        let mut inputs = Vec::new();
        for index in 0..array.length() {
            if let Some(value) = array.get_index(scope, index) {
                inputs.push(value);
            }
        }
        if inputs.is_empty() {
            return Err("Background fetch requires at least one request".to_owned());
        }
        Ok(inputs)
    } else {
        Ok(vec![value])
    }
}

fn option_number(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<f64> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())?.number_value(scope)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<BackgroundFetchManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
