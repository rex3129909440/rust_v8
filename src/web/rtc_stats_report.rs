use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcStatsReportStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<(String, v8::Global<v8::Value>)>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcStatsReportStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCStatsReport", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcStatsReportStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCStatsReport",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
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
        .get_slot_mut::<RtcStatsReportStore>()
        .ok_or_else(|| "RTCStatsReport state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    entries: Vec<(String, v8::Local<'_, v8::Value>)>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let report = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, report, prototype.into()) != Some(true) {
        return Err("cannot create RTCStatsReport".to_owned());
    }
    let entries = entries
        .into_iter()
        .map(|(key, value)| (key, v8::Global::new(scope, value)))
        .collect();
    scope
        .get_slot_mut::<RtcStatsReportStore>()
        .ok_or_else(|| "RTCStatsReport state was not prepared".to_owned())?
        .records
        .insert(report.get_identity_hash().get(), entries);
    Ok(report)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCStatsReport': Illegal constructor",
    );
}

fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    report: v8::Local<'_, v8::Object>,
) -> Option<Vec<(String, v8::Global<v8::Value>)>> {
    scope
        .get_slot::<RtcStatsReportStore>()?
        .records
        .get(&report.get_identity_hash().get())
        .cloned()
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(entries) = snapshot(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, entries.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some((_, value)) = entries.iter().find(|(current, _)| current == &key) {
        result.set(v8::Local::new(scope, value));
    } else {
        result.set(v8::undefined(scope).into());
    }
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    result.set(v8::Boolean::new(scope, entries.iter().any(|(current, _)| current == &key)).into());
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, (key, value)) in entries.iter().enumerate() {
        let pair = v8::Array::new(scope, 2);
        if let Some(key) = v8::String::new(scope, key) {
            let _ = pair.set_index(scope, 0, key.into());
        }
        let _ = pair.set_index(scope, 1, v8::Local::new(scope, value));
        let _ = array.set_index(scope, index as u32, pair.into());
    }
    return_iterator(scope, array, result);
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, (key, _)) in entries.iter().enumerate() {
        if let Some(key) = v8::String::new(scope, key) {
            let _ = array.set_index(scope, index as u32, key.into());
        }
    }
    return_iterator(scope, array, result);
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, (_, value)) in entries.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, value));
    }
    return_iterator(scope, array, result);
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
    let Some(entries) = snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let receiver = arguments.get(1);
    for (key, value) in entries {
        let Some(key) = v8::String::new(scope, &key) else {
            continue;
        };
        let value = v8::Local::new(scope, &value);
        let _ = callback.call(
            scope,
            receiver,
            &[value, key.into(), arguments.this().into()],
        );
    }
}

fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(function) = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    if let Some(iterator) = function.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}
