use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub(crate) struct CssFontFeatureValuesMapStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BTreeMap<String, Vec<u32>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssFontFeatureValuesMapStore::default());
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssFontFeatureValuesMapStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSFontFeatureValuesMap",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "entries")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssFontFeatureValuesMapStore>()
        .ok_or_else(|| "CSSFontFeatureValuesMap state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    entries: BTreeMap<String, Vec<u32>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let map = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, map, prototype.into()) != Some(true) {
        return Err("cannot create CSSFontFeatureValuesMap".to_owned());
    }
    scope
        .get_slot_mut::<CssFontFeatureValuesMapStore>()
        .ok_or_else(|| "CSSFontFeatureValuesMap state was not prepared".to_owned())?
        .records
        .insert(map.get_identity_hash().get(), entries);
    Ok(map)
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BTreeMap<String, Vec<u32>>> {
    scope
        .get_slot::<CssFontFeatureValuesMapStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CSSFontFeatureValuesMap': Illegal constructor",
    );
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = snapshot(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, values.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'set' on 'CSSFontFeatureValuesMap': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(sequence) = v8::Local::<v8::Array>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'set' on 'CSSFontFeatureValuesMap': The provided value cannot be converted to a sequence.",
        );
        return;
    };
    let mut values = Vec::new();
    for index in 0..sequence.length() {
        let value = sequence
            .get_index(scope, index)
            .and_then(|value| value.uint32_value(scope))
            .unwrap_or(0);
        values.push(value);
    }
    let Some(record) = scope
        .get_slot_mut::<CssFontFeatureValuesMapStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.insert(name, values);
    result.set(arguments.this().into());
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot_mut::<CssFontFeatureValuesMapStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.clear();
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<CssFontFeatureValuesMapStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let removed = record.remove(&name).is_some();
    result.set(v8::Boolean::new(scope, removed).into());
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = snapshot(scope, arguments.this()).and_then(|values| values.get(&name).cloned());
    if let Some(value) = value {
        result.set(number_array(scope, &value).into());
    } else if snapshot(scope, arguments.this()).is_some() {
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(values) = snapshot(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, values.contains_key(&name)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, key) in values.keys().enumerate() {
        if let Some(key) = v8::String::new(scope, key) {
            let _ = array.set_index(scope, index as u32, key.into());
        }
    }
    return_iterator(scope, array, result);
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.values().enumerate() {
        let value = number_array(scope, value);
        let _ = array.set_index(scope, index as u32, value.into());
    }
    return_iterator(scope, array, result);
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, (key, value)) in values.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        if let Some(key) = v8::String::new(scope, key) {
            let _ = pair.set_index(scope, 0, key.into());
        }
        let value = number_array(scope, value);
        let _ = pair.set_index(scope, 1, value.into());
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    return_iterator(scope, array, result);
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'forEach' on 'CSSFontFeatureValuesMap': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let Some(values) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let this_arg = if arguments.length() > 1 {
        arguments.get(1)
    } else {
        v8::undefined(scope).into()
    };
    for (key, value) in values {
        let value = number_array(scope, &value);
        let Some(key) = v8::String::new(scope, &key) else {
            continue;
        };
        let _ = callback.call(
            scope,
            this_arg,
            &[value.into(), key.into(), arguments.this().into()],
        );
    }
}

fn number_array<'s>(scope: &v8::PinScope<'s, '_>, values: &[u32]) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let value = v8::Integer::new_from_unsigned(scope, *value);
        let _ = array.set_index(scope, index as u32, value.into());
    }
    array
}

fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(method) = array.get(scope, key.into()) else {
        return;
    };
    let Ok(method) = v8::Local::<v8::Function>::try_from(method) else {
        return;
    };
    if let Some(iterator) = method.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}
