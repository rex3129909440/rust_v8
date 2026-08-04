use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HighlightRegistryStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<(String, v8::Global<v8::Object>)>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HighlightRegistryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HighlightRegistry", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<HighlightRegistryStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "HighlightRegistry",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "highlightsFromPoint",
        2,
        highlights_from_point,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "entries")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HighlightRegistryStore>()
        .ok_or_else(|| "HighlightRegistry state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HighlightRegistry".to_owned());
    }
    scope
        .get_slot_mut::<HighlightRegistryStore>()
        .ok_or_else(|| "HighlightRegistry state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, v8::Global<v8::Object>)>> {
    scope
        .get_slot::<HighlightRegistryStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
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

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<HighlightRegistryStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let existed = record(scope, arguments.this())
        .is_some_and(|record| record.iter().any(|(key, _)| key == &name));
    if let Some(record) = scope
        .get_slot_mut::<HighlightRegistryStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.retain(|(key, _)| key != &name);
        result.set(v8::Boolean::new(scope, existed).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some((_, highlight)) = record.iter().find(|(key, _)| key == &name) {
        result.set(v8::Local::new(scope, highlight).into());
    }
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, record.iter().any(|(key, _)| key == &name)).into());
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(highlight) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "HighlightRegistry values must be Highlight objects",
        );
        return;
    };
    if !super::highlight::is_highlight(scope, highlight) {
        crate::webidl::throw_type_error(
            scope,
            "HighlightRegistry values must be Highlight objects",
        );
        return;
    }
    let highlight = v8::Global::new(scope, highlight);
    if let Some(record) = scope
        .get_slot_mut::<HighlightRegistryStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        if let Some((_, value)) = record.iter_mut().find(|(key, _)| key == &name) {
            *value = highlight;
        } else {
            record.push((name, highlight));
        }
        result.set(arguments.this().into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn entry_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    record: &[(String, v8::Global<v8::Object>)],
    mode: u8,
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, record.len() as i32);
    for (index, (name, highlight)) in record.iter().enumerate() {
        let name = v8::String::new(scope, name).expect("registry name");
        let highlight = v8::Local::new(scope, highlight);
        let value: v8::Local<v8::Value> = match mode {
            1 => name.into(),
            2 => highlight.into(),
            _ => {
                let pair = v8::Array::new(scope, 2);
                let _ = pair.set_index(scope, 0, name.into());
                let _ = pair.set_index(scope, 1, highlight.into());
                pair.into()
            }
        };
        let _ = array.set_index(scope, index as u32, value);
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

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = entry_array(scope, &record, 0);
    return_iterator(scope, array, result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = entry_array(scope, &record, 1);
    return_iterator(scope, array, result);
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = entry_array(scope, &record, 2);
    return_iterator(scope, array, result);
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "forEach callback must be callable");
        return;
    };
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let receiver = arguments.get(1);
    for (name, highlight) in record {
        let Some(name) = v8::String::new(scope, &name) else {
            continue;
        };
        let highlight = v8::Local::new(scope, highlight);
        let _ = callback.call(
            scope,
            receiver,
            &[highlight.into(), name.into(), arguments.this().into()],
        );
    }
}

fn highlights_from_point(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.get(0).number_value(scope).is_none()
        || arguments.get(1).number_value(scope).is_none()
    {
        crate::webidl::throw_type_error(scope, "Coordinates must be finite numbers");
        return;
    }
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let hits = v8::Array::new(scope, record.len() as i32);
    for (index, (_, highlight)) in record.iter().enumerate() {
        let hit = v8::Object::new(scope);
        let highlight_object = v8::Local::new(scope, highlight);
        if let Some(key) = v8::String::new(scope, "highlight") {
            let _ = hit.set(scope, key.into(), highlight_object.into());
        }
        let ranges = super::highlight::ranges(scope, highlight_object).unwrap_or_default();
        let range_array = v8::Array::new(scope, ranges.len() as i32);
        for (range_index, range) in ranges.iter().enumerate() {
            let _ = range_array.set_index(
                scope,
                range_index as u32,
                v8::Local::new(scope, range).into(),
            );
        }
        if let Some(key) = v8::String::new(scope, "ranges") {
            let _ = hit.set(scope, key.into(), range_array.into());
        }
        let _ = hits.set_index(scope, index as u32, hit.into());
    }
    result.set(hits.into());
}
