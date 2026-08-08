use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceObserverEntryListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceObserverEntryListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceObserverEntryList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<PerformanceObserverEntryListStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceObserverEntryList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getEntries", 0, get_entries)?;
    crate::webidl::define_method(scope, prototype, "getEntriesByName", 1, get_entries_by_name)?;
    crate::webidl::define_method(scope, prototype, "getEntriesByType", 1, get_entries_by_type)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceObserverEntryListStore>()
        .ok_or_else(|| "PerformanceObserverEntryList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'PerformanceObserverEntryList': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    entries: Vec<v8::Global<v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let list = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, list, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceObserverEntryList".to_owned());
    }
    scope
        .get_slot_mut::<PerformanceObserverEntryListStore>()
        .ok_or_else(|| "PerformanceObserverEntryList state was not prepared".to_owned())?
        .records
        .insert(list.get_identity_hash().get(), entries);
    Ok(list)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<PerformanceObserverEntryListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn array_from_entries<'s>(
    scope: &v8::PinScope<'s, '_>,
    entries: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let output = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let _ = output.set_index(scope, index as u32, v8::Local::new(scope, entry).into());
    }
    output
}

fn get_entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(entries) = record(scope, arguments.this()) {
        let entries = super::performance::chronological_entries(scope, entries);
        result.set(array_from_entries(scope, &entries).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_entries_by_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(entries) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let entry_type = (!arguments.get(1).is_undefined())
        .then(|| crate::webidl::value_to_string(scope, arguments.get(1)));
    let matches = entries
        .into_iter()
        .filter(|entry| {
            let entry = v8::Local::new(scope, entry);
            super::performance_entry::record(scope, entry).is_some_and(|record| {
                record.name == name
                    && entry_type
                        .as_ref()
                        .is_none_or(|entry_type| record.entry_type == *entry_type)
            })
        })
        .collect::<Vec<_>>();
    let matches = super::performance::chronological_entries(scope, matches);
    result.set(array_from_entries(scope, &matches).into());
}

fn get_entries_by_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(entries) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let entry_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let matches = entries
        .into_iter()
        .filter(|entry| {
            let entry = v8::Local::new(scope, entry);
            super::performance_entry::record(scope, entry)
                .is_some_and(|record| record.entry_type == entry_type)
        })
        .collect::<Vec<_>>();
    let matches = super::performance::chronological_entries(scope, matches);
    result.set(array_from_entries(scope, &matches).into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PerformanceObserverEntryListStore>() {
        store.constructor.remove(realm_id);
    }
}
