use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HeadersStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, Vec<(String, String)>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HeadersStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Headers", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<HeadersStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Headers",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "append", 2, append)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getSetCookie", 0, get_set_cookie)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "entries")?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HeadersStore>()
        .ok_or_else(|| "Headers state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: Vec<(String, String)>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Headers".to_owned());
    }
    let mut normalized = Vec::new();
    for (name, value) in initial {
        let name = normalize_name(&name)?;
        normalized.push((name, normalize_value(&value)));
    }
    scope
        .get_slot_mut::<HeadersStore>()
        .ok_or_else(|| "Headers state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), normalized);
    Ok(object)
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, String)>> {
    scope
        .get_slot::<HeadersStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Headers': use new");
        return;
    }
    let mut initial = Vec::new();
    if let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        if let Some(existing) = snapshot(scope, object) {
            initial = existing;
        } else if object.is_array() {
            let array = v8::Local::<v8::Array>::try_from(object).ok();
            if let Some(array) = array {
                for index in 0..array.length() {
                    let Some(pair) = array
                        .get_index(scope, index)
                        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
                    else {
                        continue;
                    };
                    let name = pair
                        .get_index(scope, 0)
                        .map(|value| crate::webidl::value_to_string(scope, value))
                        .unwrap_or_default();
                    let value = pair
                        .get_index(scope, 1)
                        .map(|value| crate::webidl::value_to_string(scope, value))
                        .unwrap_or_default();
                    if let Ok(name) = normalize_name(&name) {
                        initial.push((name, normalize_value(&value)));
                    }
                }
            }
        } else if let Some(names) = object.get_own_property_names(
            scope,
            v8::GetPropertyNamesArgs {
                mode: v8::KeyCollectionMode::OwnOnly,
                property_filter: v8::PropertyFilter::ONLY_ENUMERABLE,
                index_filter: v8::IndexFilter::IncludeIndices,
                key_conversion: v8::KeyConversionMode::ConvertToString,
            },
        ) {
            for index in 0..names.length() {
                let Some(key) = names.get_index(scope, index) else {
                    continue;
                };
                let name = crate::webidl::value_to_string(scope, key);
                let value = object
                    .get(scope, key)
                    .map(|value| crate::webidl::value_to_string(scope, value))
                    .unwrap_or_default();
                if let Ok(name) = normalize_name(&name) {
                    initial.push((name, normalize_value(&value)));
                }
            }
        }
    }
    scope
        .get_slot_mut::<HeadersStore>()
        .expect("Headers state")
        .records
        .insert(arguments.this().get_identity_hash().get(), initial);
    result.set(arguments.this().into());
}

fn normalize_name(name: &str) -> Result<String, String> {
    let name = name.trim().to_ascii_lowercase();
    if name.is_empty()
        || name
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && !b"!#$%&'*+-.^_`|~".contains(&byte))
    {
        return Err("Invalid HTTP header name".to_owned());
    }
    Ok(name)
}

fn normalize_value(value: &str) -> String {
    value
        .trim_matches(|character: char| character == ' ' || character == '\t')
        .to_owned()
}

fn values_for(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, String)>> {
    snapshot(scope, object)
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Vec<(String, String)>),
) -> bool {
    if let Some(values) = scope
        .get_slot_mut::<HeadersStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(values);
        true
    } else {
        false
    }
}

fn header_name(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<String> {
    let value = crate::webidl::value_to_string(scope, value);
    match normalize_name(&value) {
        Ok(value) => Some(value),
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            None
        }
    }
}
fn return_string(scope: &mut v8::PinScope<'_, '_>, value: &str, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into())
    }
}
fn append(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    let value = normalize_value(&crate::webidl::value_to_string(scope, arguments.get(1)));
    if !update(scope, arguments.this(), |values| values.push((name, value))) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    if !update(scope, arguments.this(), |values| {
        values.retain(|(current, _)| current != &name)
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matched: Vec<&str> = values
        .iter()
        .filter(|(current, _)| current == &name)
        .map(|(_, value)| value.as_str())
        .collect();
    if matched.is_empty() {
        result.set(v8::null(scope).into())
    } else {
        return_string(scope, &matched.join(", "), result)
    }
}
fn get_set_cookie(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matched: Vec<&str> = values
        .iter()
        .filter(|(name, _)| name == "set-cookie")
        .map(|(_, value)| value.as_str())
        .collect();
    let array = v8::Array::new(scope, matched.len() as i32);
    for (index, value) in matched.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    result.set(array.into())
}
fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    if let Some(values) = values_for(scope, arguments.this()) {
        result
            .set(v8::Boolean::new(scope, values.iter().any(|(current, _)| current == &name)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(name) = header_name(scope, arguments.get(0)) else {
        return;
    };
    let value = normalize_value(&crate::webidl::value_to_string(scope, arguments.get(1)));
    if !update(scope, arguments.this(), |values| {
        values.retain(|(current, _)| current != &name);
        values.push((name, value))
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn combined(values: &[(String, String)]) -> Vec<(String, String)> {
    let mut output: Vec<(String, String)> = Vec::new();
    for (name, value) in values {
        if name == "set-cookie" {
            output.push((name.clone(), value.clone()));
            continue;
        }
        if let Some((_, existing)) = output.iter_mut().find(|(current, _)| current == name) {
            existing.push_str(", ");
            existing.push_str(value)
        } else {
            output.push((name.clone(), value.clone()))
        }
    }
    output.sort_by(|a, b| a.0.cmp(&b.0));
    output
}
fn iterator(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(function) = array
        .get(scope, key.into())
        .and_then(|v| v8::Local::<v8::Function>::try_from(v).ok())
    else {
        return;
    };
    if let Some(value) = function.call(scope, array.into(), &[]) {
        result.set(value)
    }
}
fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = combined(&values);
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, (name, value)) in values.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        if let Some(name) = v8::String::new(scope, name) {
            let _ = pair.set_index(scope, 0, name.into());
        }
        if let Some(value) = v8::String::new(scope, value) {
            let _ = pair.set_index(scope, 1, value.into());
        }
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    iterator(scope, array, result)
}
fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = combined(&values);
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, (name, _)) in values.iter().enumerate() {
        if let Some(name) = v8::String::new(scope, name) {
            let _ = array.set_index(scope, index as u32, name.into());
        }
    }
    iterator(scope, array, result)
}
fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = combined(&values);
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, (_, value)) in values.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    iterator(scope, array, result)
}
fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let Some(values) = values_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let receiver = arguments.get(1);
    for (name, value) in combined(&values) {
        let Some(name) = v8::String::new(scope, &name) else {
            continue;
        };
        let Some(value) = v8::String::new(scope, &value) else {
            continue;
        };
        let _ = callback.call(
            scope,
            receiver,
            &[value.into(), name.into(), arguments.this().into()],
        );
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<HeadersStore>() {
        store.constructors.remove(&realm_id);
    }
}
