use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ViewTransitionTypeSetStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<String>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ViewTransitionTypeSetStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ViewTransitionTypeSet", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ViewTransitionTypeSetStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ViewTransitionTypeSet",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "add", 1, add)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "values")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ViewTransitionTypeSetStore>()
        .ok_or_else(|| "ViewTransitionTypeSet state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: Vec<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ViewTransitionTypeSet".to_owned());
    }
    let mut unique = Vec::new();
    for value in values {
        if !unique.iter().any(|existing| existing == &value) {
            unique.push(value);
        }
    }
    scope
        .get_slot_mut::<ViewTransitionTypeSetStore>()
        .ok_or_else(|| "ViewTransitionTypeSet state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), unique);
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Vec<String>> {
    scope
        .get_slot::<ViewTransitionTypeSetStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Vec<String>),
) {
    if let Some(record) = scope
        .get_slot_mut::<ViewTransitionTypeSetStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.len() as i32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        if !record.iter().any(|existing| existing == &value) {
            record.push(value);
        }
    });
    result.set(arguments.this().into());
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), Vec::clear);
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let existed = record(scope, arguments.this())
        .is_some_and(|record| record.iter().any(|existing| existing == &value));
    update(scope, arguments.this(), |record| {
        record.retain(|existing| existing != &value)
    });
    result.set(v8::Boolean::new(scope, existed).into());
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let exists = record.iter().any(|existing| existing == &value);
    result.set(v8::Boolean::new(scope, exists).into());
}

fn string_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    values: &[String],
    pairs: bool,
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            if pairs {
                let pair = v8::Array::new(scope, 2);
                let _ = pair.set_index(scope, 0, value.into());
                let _ = pair.set_index(scope, 1, value.into());
                let _ = array.set_index(scope, index as u32, pair.into());
            } else {
                let _ = array.set_index(scope, index as u32, value.into());
            }
        }
    }
    array
}

fn iterator_from_array(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    result: &mut v8::ReturnValue<'_>,
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

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = string_array(scope, &record, true);
    iterator_from_array(scope, array, &mut result);
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = string_array(scope, &record, false);
    iterator_from_array(scope, array, &mut result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    values(scope, arguments, result);
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "forEach callback must be callable");
        return;
    };
    let this_argument = arguments.get(1);
    for value in record {
        if let Some(value) = v8::String::new(scope, &value) {
            let _ = callback.call(
                scope,
                this_argument,
                &[value.into(), value.into(), arguments.this().into()],
            );
        }
    }
}
