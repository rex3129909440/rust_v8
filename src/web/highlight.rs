use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HighlightStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, HighlightRecord>,
}

#[derive(Clone)]
struct HighlightRecord {
    priority: i32,
    highlight_type: String,
    ranges: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HighlightStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Highlight", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<HighlightStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Highlight",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "priority", get_priority, set_priority)?;
    crate::webidl::define_accessor(scope, prototype, "type", get_type, set_type)?;
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
        .get_slot_mut::<HighlightStore>()
        .ok_or_else(|| "Highlight state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Highlight': use new");
        return;
    }
    let mut ranges = Vec::new();
    for index in 0..arguments.length() {
        let Ok(range) = v8::Local::<v8::Object>::try_from(arguments.get(index)) else {
            crate::webidl::throw_type_error(
                scope,
                "Highlight entries must be AbstractRange objects",
            );
            return;
        };
        if super::abstract_range::record(scope, range).is_none() {
            crate::webidl::throw_type_error(
                scope,
                "Highlight entries must be AbstractRange objects",
            );
            return;
        }
        if !ranges.iter().any(|existing: &v8::Global<v8::Object>| {
            v8::Local::new(scope, existing).strict_equals(range.into())
        }) {
            ranges.push(v8::Global::new(scope, range));
        }
    }
    scope
        .get_slot_mut::<HighlightStore>()
        .expect("Highlight state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            HighlightRecord {
                priority: 0,
                highlight_type: "highlight".to_owned(),
                ranges,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<HighlightRecord> {
    scope
        .get_slot::<HighlightStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_highlight(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, object).is_some()
}

pub(crate) fn ranges(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    Some(record(scope, object)?.ranges)
}

fn get_priority(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.priority).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_priority(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).int32_value(scope).unwrap_or(0);
    if let Some(record) = scope.get_slot_mut::<HighlightStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.priority = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.highlight_type) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "highlight" && value != "spelling-error" && value != "grammar-error" {
        crate::webidl::throw_type_error(scope, "Invalid Highlight type");
        return;
    }
    if let Some(record) = scope.get_slot_mut::<HighlightStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.highlight_type = value;
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
        result.set(v8::Integer::new(scope, record.ranges.len() as i32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn required_range(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Object>> {
    let Ok(range) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "Value must be an AbstractRange");
        return None;
    };
    if super::abstract_range::record(scope, range).is_none() {
        crate::webidl::throw_type_error(scope, "Value must be an AbstractRange");
        return None;
    }
    Some(v8::Global::new(scope, range))
}

fn contains(
    scope: &v8::PinScope<'_, '_>,
    ranges: &[v8::Global<v8::Object>],
    candidate: v8::Local<'_, v8::Object>,
) -> bool {
    ranges
        .iter()
        .any(|range| v8::Local::new(scope, range).strict_equals(candidate.into()))
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(range) = required_range(scope, arguments.get(0)) else {
        return;
    };
    let range_local = v8::Local::new(scope, &range);
    let snapshot = record(scope, arguments.this());
    let Some(snapshot) = snapshot else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !contains(scope, &snapshot.ranges, range_local)
        && let Some(record) = scope.get_slot_mut::<HighlightStore>().and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.ranges.push(range);
    }
    result.set(arguments.this().into());
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<HighlightStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.ranges.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(range) = required_range(scope, arguments.get(0)) else {
        return;
    };
    let range_local = v8::Local::new(scope, &range);
    let position = record(scope, arguments.this()).and_then(|record| {
        record
            .ranges
            .iter()
            .position(|existing| v8::Local::new(scope, existing).strict_equals(range_local.into()))
    });
    if let Some(record) = scope.get_slot_mut::<HighlightStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        if let Some(position) = position {
            record.ranges.remove(position);
        }
        result.set(v8::Boolean::new(scope, position.is_some()).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(range) = required_range(scope, arguments.get(0)) else {
        return;
    };
    let range = v8::Local::new(scope, &range);
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, contains(scope, &record.ranges, range)).into());
}

fn range_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    record: &HighlightRecord,
    pairs: bool,
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, record.ranges.len() as i32);
    for (index, range) in record.ranges.iter().enumerate() {
        let range = v8::Local::new(scope, range);
        if pairs {
            let pair = v8::Array::new(scope, 2);
            let _ = pair.set_index(scope, 0, range.into());
            let _ = pair.set_index(scope, 1, range.into());
            let _ = array.set_index(scope, index as u32, pair.into());
        } else {
            let _ = array.set_index(scope, index as u32, range.into());
        }
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
    let array = range_array(scope, &record, true);
    return_iterator(scope, array, result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    values(scope, arguments, result);
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
    let array = range_array(scope, &record, false);
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
    for range in record.ranges {
        let range = v8::Local::new(scope, range);
        let _ = callback.call(
            scope,
            receiver,
            &[range.into(), range.into(), arguments.this().into()],
        );
    }
}
