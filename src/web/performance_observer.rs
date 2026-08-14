use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct PerformanceObserverStore {
    constructor: crate::webidl::RealmConstructor,
    supported_entry_types: HashMap<i32, v8::Global<v8::Array>>,
    records: HashMap<i32, ObserverRecord>,
    buffered_entries: HashMap<(i32, String), Vec<v8::Global<v8::Object>>>,
}

#[derive(Clone)]
struct ObserverRecord {
    callback: v8::Global<v8::Function>,
    observer: v8::Global<v8::Object>,
    realm_id: i32,
    observed_types: HashSet<String>,
    queued_entries: Vec<v8::Global<v8::Object>>,
    microtask_scheduled: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceObserverStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceObserver", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<PerformanceObserverStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceObserver",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "disconnect", 0, disconnect)?;
    crate::webidl::define_method(scope, prototype, "observe", 0, observe)?;
    crate::webidl::define_method(scope, prototype, "takeRecords", 0, take_records)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(
        scope,
        constructor.into(),
        "supportedEntryTypes",
        get_supported_entry_types,
    )?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceObserverStore>()
        .ok_or_else(|| "PerformanceObserver state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PerformanceObserver': 1 argument required",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PerformanceObserver': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let record = ObserverRecord {
        callback: v8::Global::new(scope, callback),
        observer: v8::Global::new(scope, arguments.this()),
        realm_id: crate::webidl::realm_id(scope),
        observed_types: HashSet::new(),
        queued_entries: Vec::new(),
        microtask_scheduled: false,
    };
    scope
        .get_slot_mut::<PerformanceObserverStore>()
        .expect("PerformanceObserver state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn get_supported_entry_types(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let existing = scope
        .get_slot::<PerformanceObserverStore>()
        .and_then(|store| {
            store
                .supported_entry_types
                .get(&crate::webidl::realm_id(scope))
        })
        .cloned();
    if let Some(existing) = existing {
        result.set(v8::Local::new(scope, &existing).into());
        return;
    }
    let chromium_151 = crate::browser_surface::current_version(scope).major() >= 151;
    let values = v8::Array::new(scope, if chromium_151 { 15 } else { 13 });
    set_string_index(scope, values, 0, "element");
    set_string_index(scope, values, 1, "event");
    set_string_index(scope, values, 2, "first-input");
    if chromium_151 {
        set_string_index(scope, values, 3, "interaction-contentful-paint");
        set_string_index(scope, values, 4, "largest-contentful-paint");
        set_string_index(scope, values, 5, "layout-shift");
        set_string_index(scope, values, 6, "long-animation-frame");
        set_string_index(scope, values, 7, "longtask");
        set_string_index(scope, values, 8, "mark");
        set_string_index(scope, values, 9, "measure");
        set_string_index(scope, values, 10, "navigation");
        set_string_index(scope, values, 11, "paint");
        set_string_index(scope, values, 12, "resource");
        set_string_index(scope, values, 13, "soft-navigation");
        set_string_index(scope, values, 14, "visibility-state");
    } else {
        set_string_index(scope, values, 3, "largest-contentful-paint");
        set_string_index(scope, values, 4, "layout-shift");
        set_string_index(scope, values, 5, "long-animation-frame");
        set_string_index(scope, values, 6, "longtask");
        set_string_index(scope, values, 7, "mark");
        set_string_index(scope, values, 8, "measure");
        set_string_index(scope, values, 9, "navigation");
        set_string_index(scope, values, 10, "paint");
        set_string_index(scope, values, 11, "resource");
        set_string_index(scope, values, 12, "visibility-state");
    }
    let global = v8::Global::new(scope, values);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<PerformanceObserverStore>()
        .expect("PerformanceObserver state")
        .supported_entry_types
        .insert(realm_id, global);
    result.set(values.into());
}

fn set_string_index(
    scope: &v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    index: u32,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        let _ = array.set_index(scope, index, value.into());
    }
}

fn disconnect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<PerformanceObserverStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.observed_types.clear();
        record.queued_entries.clear();
        record.microtask_scheduled = false;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn observe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    if !scope
        .get_slot::<PerformanceObserverStore>()
        .is_some_and(|store| store.records.contains_key(&identity))
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "An observe() call must include either entryTypes or type arguments",
        );
        return;
    };
    let single_type = string_property(scope, options, "type");
    let entry_types = sequence_property(scope, options, "entryTypes");
    let buffered = boolean_property(scope, options, "buffered");
    if single_type.is_none() && entry_types.is_empty() {
        crate::webidl::throw_type_error(
            scope,
            "An observe() call must include either entryTypes or type arguments",
        );
        return;
    }
    let buffered_type = buffered.then(|| single_type.clone()).flatten();
    if let Some(record) = scope
        .get_slot_mut::<PerformanceObserverStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        if let Some(single_type) = single_type {
            record.observed_types.insert(single_type);
        } else {
            record.observed_types.clear();
            for entry_type in entry_types {
                record.observed_types.insert(entry_type);
            }
        }
    }
    if let Some(entry_type) = buffered_type {
        let entries = observer_buffered_entries(scope, &entry_type);
        queue_entries_for_observer(scope, identity, entries);
    }
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn boolean_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    let Some(key) = v8::String::new(scope, name) else {
        return false;
    };
    object
        .get(scope, key.into())
        .is_some_and(|value| value.boolean_value(scope))
}

fn sequence_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Vec<String> {
    let Some(key) = v8::String::new(scope, name) else {
        return Vec::new();
    };
    let Some(value) = object.get(scope, key.into()) else {
        return Vec::new();
    };
    let Ok(sequence) = v8::Local::<v8::Object>::try_from(value) else {
        return Vec::new();
    };
    let Some(length_key) = v8::String::new(scope, "length") else {
        return Vec::new();
    };
    let length = sequence
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let mut output = Vec::with_capacity(length as usize);
    for index in 0..length {
        if let Some(value) = sequence.get_index(scope, index) {
            output.push(crate::webidl::value_to_string(scope, value));
        }
    }
    output
}

fn take_records(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let entries = {
        let Some(record) = scope
            .get_slot_mut::<PerformanceObserverStore>()
            .and_then(|store| {
                store
                    .records
                    .get_mut(&arguments.this().get_identity_hash().get())
            })
        else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        };
        std::mem::take(&mut record.queued_entries)
    };
    let output = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let _ = output.set_index(scope, index as u32, v8::Local::new(scope, entry).into());
    }
    result.set(output.into());
}

pub(crate) fn queue_entry(
    scope: &mut v8::PinScope<'_, '_>,
    entry: v8::Local<'_, v8::Object>,
    entry_type: &str,
) {
    let realm_id = crate::webidl::realm_id(scope);
    let entry_global = v8::Global::new(scope, entry);
    if let Some(store) = scope.get_slot_mut::<PerformanceObserverStore>() {
        let buffer = store
            .buffered_entries
            .entry((realm_id, entry_type.to_owned()))
            .or_default();
        let maximum = maximum_buffer_size(entry_type);
        if buffer.len() < maximum {
            buffer.push(entry_global.clone());
        }
    }
    let observer_ids = scope
        .get_slot::<PerformanceObserverStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| {
                    record.realm_id == realm_id && record.observed_types.contains(entry_type)
                })
                .map(|(observer_id, _)| *observer_id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for observer_id in observer_ids {
        queue_entries_for_observer(scope, observer_id, vec![entry_global.clone()]);
    }
}

fn maximum_buffer_size(entry_type: &str) -> usize {
    match entry_type {
        "first-input" => 1,
        "paint" => 2,
        "resource" => 250,
        "longtask" | "long-animation-frame" => 200,
        "element" | "event" | "largest-contentful-paint" | "layout-shift" => 150,
        _ => usize::MAX,
    }
}

fn observer_buffered_entries(
    scope: &v8::PinScope<'_, '_>,
    entry_type: &str,
) -> Vec<v8::Global<v8::Object>> {
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot::<PerformanceObserverStore>()
        .and_then(|store| {
            store
                .buffered_entries
                .get(&(realm_id, entry_type.to_owned()))
        })
        .cloned()
        .unwrap_or_default()
}

pub(crate) fn task_start(scope: &v8::PinScope<'_, '_>) -> f64 {
    super::performance::now_for_current_realm(scope).unwrap_or(0.0)
}

pub(crate) fn record_completed_task(
    scope: &mut v8::PinScope<'_, '_>,
    start_time: f64,
    include_animation_frame: bool,
) -> bool {
    let end_time = super::performance::now_for_current_realm(scope).unwrap_or(start_time);
    let duration = (end_time - start_time).max(0.0);
    if duration < 50.0 {
        return false;
    }

    let attribution = super::task_attribution_timing::create(
        scope,
        "window".to_owned(),
        String::new(),
        String::new(),
        String::new(),
    )
    .ok()
    .map(|entry| vec![v8::Global::new(scope, entry)])
    .unwrap_or_default();
    if let Ok(entry) = super::performance_long_task_timing::create(
        scope,
        "self".to_owned(),
        start_time,
        duration,
        attribution,
    ) {
        queue_entry(scope, entry, "longtask");
    }

    if include_animation_frame {
        let scripts = super::performance_script_timing::create(
            scope,
            "script".to_owned(),
            start_time,
            duration,
            String::new(),
            String::new(),
        )
        .ok()
        .map(|entry| vec![v8::Global::new(scope, entry)])
        .unwrap_or_default();
        if let Ok(entry) = super::performance_long_animation_frame_timing::create(
            scope, start_time, duration, scripts,
        ) {
            super::performance::add_entry_for_current_realm(scope, entry, "long-animation-frame");
        }
    }
    true
}

fn queue_entries_for_observer(
    scope: &mut v8::PinScope<'_, '_>,
    observer_id: i32,
    entries: Vec<v8::Global<v8::Object>>,
) {
    if entries.is_empty() {
        return;
    }
    let should_schedule = scope
        .get_slot_mut::<PerformanceObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
        .is_some_and(|record| {
            record.queued_entries.extend(entries);
            if record.microtask_scheduled {
                false
            } else {
                record.microtask_scheduled = true;
                true
            }
        });
    if should_schedule {
        let data = v8::Integer::new(scope, observer_id);
        if let Some(callback) = v8::Function::builder(deliver_entries)
            .data(data.into())
            .length(0)
            .constructor_behavior(v8::ConstructorBehavior::Throw)
            .build(scope)
        {
            scope.enqueue_microtask(callback);
        }
    }
}

fn deliver_entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(observer_id) = arguments.data().int32_value(scope) else {
        return;
    };
    let Some((callback, observer, entries)) = scope
        .get_slot_mut::<PerformanceObserverStore>()
        .and_then(|store| store.records.get_mut(&observer_id))
        .and_then(|record| {
            record.microtask_scheduled = false;
            if record.queued_entries.is_empty() {
                return None;
            }
            Some((
                record.callback.clone(),
                record.observer.clone(),
                std::mem::take(&mut record.queued_entries),
            ))
        })
    else {
        return;
    };
    let Ok(list) = super::performance_observer_entry_list::create(scope, entries) else {
        return;
    };
    let options = v8::Object::new(scope);
    if let Some(key) = v8::String::new(scope, "droppedEntriesCount") {
        let _ = options.create_data_property(scope, key.into(), v8::Integer::new(scope, 0).into());
    }
    let callback = v8::Local::new(scope, &callback);
    let observer = v8::Local::new(scope, &observer);
    let list = crate::trace::visible_callback_value(
        scope,
        list.into(),
        "PerformanceObserver callback entries",
    );
    let observer = crate::trace::visible_callback_value(
        scope,
        observer.into(),
        "PerformanceObserver callback observer",
    );
    let _ = callback.call(
        scope,
        v8::undefined(scope).into(),
        &[list, observer, options.into()],
    );
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PerformanceObserverStore>() {
        store.constructor.remove(realm_id);
        store.supported_entry_types.remove(&realm_id);
        store
            .records
            .retain(|_, record| record.realm_id != realm_id);
        store
            .buffered_entries
            .retain(|(entry_realm_id, _), _| *entry_realm_id != realm_id);
    }
}
