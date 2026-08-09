use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, PerformanceRecord>,
}

#[derive(Clone)]
struct PerformanceRecord {
    realm_id: i32,
    is_window: bool,
    target: v8::Global<v8::Object>,
    time_origin: f64,
    monotonic_origin: f64,
    on_resource_timing_buffer_full: Option<v8::Global<v8::Value>>,
    entries: Vec<v8::Global<v8::Object>>,
    resource_timing_buffer_size: usize,
    resource_timing_buffer_full_notified: bool,
    timing: v8::Global<v8::Object>,
    navigation: v8::Global<v8::Object>,
    memory: v8::Global<v8::Object>,
    event_counts: v8::Global<v8::Object>,
    interaction_count: i32,
    profile_entries_override: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Performance", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Performance",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timeOrigin", get_time_origin)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onresourcetimingbufferfull",
        get_on_resource_timing_buffer_full,
        set_on_resource_timing_buffer_full,
    )?;
    crate::webidl::define_method(scope, prototype, "clearMarks", 0, clear_marks)?;
    crate::webidl::define_method(scope, prototype, "clearMeasures", 0, clear_measures)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "clearResourceTimings",
        0,
        clear_resource_timings,
    )?;
    crate::webidl::define_method(scope, prototype, "getEntries", 0, get_entries)?;
    crate::webidl::define_method(scope, prototype, "getEntriesByName", 1, get_entries_by_name)?;
    crate::webidl::define_method(scope, prototype, "getEntriesByType", 1, get_entries_by_type)?;
    crate::webidl::define_method(scope, prototype, "mark", 1, mark)?;
    crate::webidl::define_method(scope, prototype, "measure", 1, measure)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setResourceTimingBufferSize",
        1,
        set_resource_timing_buffer_size,
    )?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_method(scope, prototype, "now", 0, now)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timing", get_timing)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "navigation", get_navigation)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "memory", get_memory)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "eventCounts", get_event_counts)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "interactionCount",
        get_interaction_count,
    )?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceStore>()
        .ok_or_else(|| "Performance state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Performance': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    is_window: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let performance = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, performance, prototype.into()) != Some(true) {
        return Err("cannot create Performance".to_owned());
    }
    super::event_target::attach(scope, performance);
    let elapsed = crate::determinism::elapsed_milliseconds(scope);
    let is_root_realm = scope
        .get_slot::<PerformanceStore>()
        .is_some_and(|store| store.records.is_empty());
    let configured_entries = is_root_realm
        .then(|| crate::fingerprint::edge(scope).performance.entries.clone())
        .flatten();
    let monotonic_origin = if is_root_realm { 0.0 } else { elapsed };
    let time_origin = crate::determinism::high_resolution_milliseconds(
        scope,
        crate::determinism::epoch_milliseconds(scope) - (elapsed - monotonic_origin),
    );
    let timing = super::performance_timing::create(scope, time_origin.floor())?;
    let navigation = super::performance_navigation::create(scope, 0, 0)?;
    let memory_profile = crate::fingerprint::edge(scope).memory.clone();
    let memory = v8::Object::new(scope);
    define_number(
        scope,
        memory,
        "jsHeapSizeLimit",
        memory_profile.performance_js_heap_size_limit as f64,
    );
    define_number(
        scope,
        memory,
        "totalJSHeapSize",
        memory_profile.performance_total_js_heap_size as f64,
    );
    define_number(
        scope,
        memory,
        "usedJSHeapSize",
        memory_profile.performance_used_js_heap_size as f64,
    );
    let event_counts =
        super::event_counts::create(scope, super::event_counts::edge_150_initial_values())?;
    let record = PerformanceRecord {
        realm_id: crate::webidl::realm_id(scope),
        is_window,
        target: v8::Global::new(scope, performance),
        time_origin,
        monotonic_origin,
        on_resource_timing_buffer_full: None,
        entries: Vec::new(),
        resource_timing_buffer_size: 250,
        resource_timing_buffer_full_notified: false,
        timing: v8::Global::new(scope, timing),
        navigation: v8::Global::new(scope, navigation),
        memory: v8::Global::new(scope, memory),
        event_counts: v8::Global::new(scope, event_counts),
        interaction_count: 0,
        profile_entries_override: configured_entries.is_some(),
    };
    scope
        .get_slot_mut::<PerformanceStore>()
        .ok_or_else(|| "Performance state was not prepared".to_owned())?
        .records
        .insert(performance.get_identity_hash().get(), record);
    if let Some(entries) = configured_entries {
        for configured in entries {
            let entry = match configured.entry_type.as_str() {
                "navigation" => {
                    super::performance_navigation_timing::create_from_profile(scope, &configured)
                }
                "resource" => {
                    super::performance_resource_timing::create_from_profile(scope, &configured)
                }
                "visibility-state" => super::visibility_state_entry::create(
                    scope,
                    &configured.name,
                    configured.start_time,
                ),
                "paint" => super::performance_paint_timing::create(
                    scope,
                    configured.name.clone(),
                    configured.start_time,
                    configured.paint_time,
                    configured.presentation_time,
                ),
                _ => continue,
            }?;
            push_entry(scope, performance, entry, &configured.entry_type);
        }
    } else {
        if let Some(url) = current_location_href(scope) {
            ensure_navigation_entry(
                scope,
                url,
                200,
                crate::page_init::html(scope).len(),
                crate::page_init::content_type(scope),
            );
        }
        ensure_initial_visibility_entry(scope);
    }
    Ok(performance)
}

fn ensure_initial_visibility_entry(scope: &mut v8::PinScope<'_, '_>) {
    if !buffered_entries(scope, "visibility-state").is_empty() {
        return;
    }
    let state = crate::fingerprint::edge(scope)
        .document
        .visibility_state
        .clone()
        .unwrap_or_else(|| "visible".to_owned());
    if let Ok(entry) = super::visibility_state_entry::create(scope, &state, 0.0) {
        add_entry_for_current_realm(scope, entry, "visibility-state");
    }
}

pub(crate) fn finalize_page_load(scope: &mut v8::PinScope<'_, '_>) {
    if current_realm_uses_profile_entries(scope) {
        return;
    }
    let completed_at = now_for_current_realm(scope).unwrap_or(0.0).max(0.0);
    if let Some(navigation) = buffered_entries(scope, "navigation").into_iter().next() {
        let navigation = v8::Local::new(scope, &navigation);
        super::performance_navigation_timing::finalize_page_load(scope, navigation, completed_at);
    }

    if !buffered_entries(scope, "paint").is_empty() || !document_has_contentful_paint(scope) {
        return;
    }
    for name in ["first-paint", "first-contentful-paint"] {
        if let Ok(entry) = super::performance_paint_timing::create(
            scope,
            name.to_owned(),
            completed_at,
            completed_at,
            completed_at,
        ) {
            add_entry_for_current_realm(scope, entry, "paint");
        }
    }
}

fn document_has_contentful_paint(scope: &v8::PinScope<'_, '_>) -> bool {
    if super::window_view_state::inner_width(scope) <= 0.0
        || super::window_view_state::inner_height(scope) <= 0.0
    {
        return false;
    }
    let Some(document) = super::document_global::value(scope) else {
        return false;
    };
    subtree_has_contentful_paint(scope, document, true)
}

fn subtree_has_contentful_paint(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    rendered: bool,
) -> bool {
    let Some(record) = super::node::record(scope, node) else {
        return false;
    };
    let rendered = rendered
        && !matches!(
            record.node_name.as_str(),
            "HEAD" | "NOSCRIPT" | "SCRIPT" | "STYLE" | "TEMPLATE" | "TITLE"
        );
    if rendered
        && ((record.node_type == super::node::TEXT_NODE
            && record
                .node_value
                .as_deref()
                .is_some_and(|text| !text.trim().is_empty()))
            || (record.node_type == super::node::ELEMENT_NODE
                && matches!(
                    record.node_name.as_str(),
                    "CANVAS" | "IMG" | "PICTURE" | "SVG" | "VIDEO"
                )))
    {
        return true;
    }
    super::node::children(scope, node)
        .into_iter()
        .any(|child| subtree_has_contentful_paint(scope, child, rendered))
}

fn current_location_href(scope: &mut v8::PinScope<'_, '_>) -> Option<String> {
    let global = scope.get_current_context().global(scope);
    let location_key = v8::String::new(scope, "location")?;
    let location = global.get(scope, location_key.into())?;
    let location = v8::Local::<v8::Object>::try_from(location).ok()?;
    let href_key = v8::String::new(scope, "href")?;
    let href = location.get(scope, href_key.into())?;
    (!href.is_undefined()).then(|| crate::webidl::value_to_string(scope, href))
}

pub(crate) fn ensure_navigation_entry(
    scope: &mut v8::PinScope<'_, '_>,
    name: String,
    response_status: u16,
    body_size: usize,
    content_type: String,
) {
    let already_present = buffered_entries(scope, "navigation")
        .into_iter()
        .next()
        .is_some();
    if already_present {
        return;
    }
    let duration = now_for_current_realm(scope).unwrap_or(0.0).max(0.0);
    if let Ok(entry) = super::performance_navigation_timing::create_for_navigation(
        scope,
        name,
        duration,
        response_status,
        body_size,
        content_type,
    ) {
        add_entry_for_current_realm(scope, entry, "navigation");
    }
}

pub(crate) fn replace_navigation_entry(
    scope: &mut v8::PinScope<'_, '_>,
    name: String,
    response_status: u16,
    body_size: usize,
    content_type: String,
) {
    if current_realm_uses_profile_entries(scope) {
        return;
    }
    let realm_id = crate::webidl::realm_id(scope);
    let current = scope.get_slot::<PerformanceStore>().and_then(|store| {
        store.records.iter().find_map(|(id, record)| {
            (record.realm_id == realm_id).then(|| (*id, record.entries.clone()))
        })
    });
    if let Some((id, entries)) = current {
        let retained = entries
            .into_iter()
            .filter(|entry| {
                let entry = v8::Local::new(scope, entry);
                !super::performance_entry::record(scope, entry)
                    .is_some_and(|record| record.entry_type == "navigation")
            })
            .collect();
        if let Some(record) = scope
            .get_slot_mut::<PerformanceStore>()
            .and_then(|store| store.records.get_mut(&id))
        {
            record.entries = retained;
        }
    }
    ensure_navigation_entry(scope, name, response_status, body_size, content_type);
}

pub(crate) fn buffered_entries(
    scope: &v8::PinScope<'_, '_>,
    entry_type: &str,
) -> Vec<v8::Global<v8::Object>> {
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot::<PerformanceStore>()
        .and_then(|store| {
            store
                .records
                .values()
                .find(|record| record.realm_id == realm_id)
        })
        .map(|record| {
            record
                .entries
                .iter()
                .filter(|entry| {
                    let entry = v8::Local::new(scope, *entry);
                    super::performance_entry::record(scope, entry)
                        .is_some_and(|record| record.entry_type == entry_type)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PerformanceRecord> {
    scope
        .get_slot::<PerformanceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn now_for_record(scope: &v8::PinScope<'_, '_>, record: &PerformanceRecord) -> f64 {
    crate::determinism::relative_high_resolution_milliseconds(
        scope,
        crate::determinism::elapsed_milliseconds(scope),
        record.monotonic_origin,
    )
}

pub(crate) fn now_for_realm(scope: &v8::PinScope<'_, '_>, realm_id: i32) -> Option<f64> {
    scope
        .get_slot::<PerformanceStore>()?
        .records
        .values()
        .find(|record| record.realm_id == realm_id)
        .map(|record| now_for_record(scope, record))
}

pub(crate) fn now_for_current_realm(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    now_for_realm(scope, crate::webidl::realm_id(scope))
}

pub(crate) fn current_realm_is_window(scope: &v8::PinScope<'_, '_>) -> bool {
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot::<PerformanceStore>()
        .and_then(|store| {
            store
                .records
                .values()
                .find(|record| record.realm_id == realm_id)
        })
        .is_some_and(|record| record.is_window)
}

fn get_time_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.time_origin).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_resource_timing_buffer_full(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.on_resource_timing_buffer_full {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_on_resource_timing_buffer_full(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, arguments.get(0)))
    };
    super::event_target::set_attribute_handler(
        scope,
        arguments.this(),
        "resourcetimingbufferfull",
        value.is_some(),
    );
    if let Some(record) = scope.get_slot_mut::<PerformanceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.on_resource_timing_buffer_full = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn clear_entry_type(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    entry_type: &str,
    name: Option<String>,
) {
    let Some(snapshot) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let retained = snapshot
        .entries
        .into_iter()
        .filter(|entry| {
            let entry = v8::Local::new(scope, entry);
            super::performance_entry::record(scope, entry).is_none_or(|record| {
                record.entry_type != entry_type
                    || name
                        .as_ref()
                        .is_some_and(|name| record.name.as_str() != name)
            })
        })
        .collect::<Vec<_>>();
    if let Some(record) = scope
        .get_slot_mut::<PerformanceStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.entries = retained;
    }
}

fn optional_name(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Option<String> {
    (!value.is_undefined()).then(|| crate::webidl::value_to_string(scope, value))
}

fn clear_marks(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    clear_entry_type(
        scope,
        arguments.this(),
        "mark",
        optional_name(scope, arguments.get(0)),
    );
}

fn clear_measures(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    clear_entry_type(
        scope,
        arguments.this(),
        "measure",
        optional_name(scope, arguments.get(0)),
    );
}

fn clear_resource_timings(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    clear_entry_type(scope, arguments.this(), "resource", None);
    if let Some(record) = scope.get_slot_mut::<PerformanceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.resource_timing_buffer_full_notified = false;
    }
}

fn entries_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    entries: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let output = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let _ = output.set_index(scope, index as u32, v8::Local::new(scope, entry).into());
    }
    output
}

pub(crate) fn chronological_entries(
    scope: &v8::PinScope<'_, '_>,
    mut entries: Vec<v8::Global<v8::Object>>,
) -> Vec<v8::Global<v8::Object>> {
    entries.sort_by(|left, right| {
        let left = v8::Local::new(scope, left);
        let right = v8::Local::new(scope, right);
        let left = super::performance_entry::record(scope, left)
            .map(|record| record.start_time)
            .unwrap_or(0.0);
        let right = super::performance_entry::record(scope, right)
            .map(|record| record.start_time)
            .unwrap_or(0.0);
        left.partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries
}

fn get_entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let entries = chronological_entries(scope, record.entries);
        result.set(entries_array(scope, &entries).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_entries_by_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let entry_type = optional_name(scope, arguments.get(1));
    let entries = record
        .entries
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
    let entries = chronological_entries(scope, entries);
    result.set(entries_array(scope, &entries).into());
}

fn get_entries_by_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let entry_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let entries = record
        .entries
        .into_iter()
        .filter(|entry| {
            let entry = v8::Local::new(scope, entry);
            super::performance_entry::record(scope, entry)
                .is_some_and(|record| record.entry_type == entry_type)
        })
        .collect::<Vec<_>>();
    let entries = chronological_entries(scope, entries);
    result.set(entries_array(scope, &entries).into());
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

const LEGACY_TIMING_NAMES: [&str; 21] = [
    "navigationStart",
    "unloadEventStart",
    "unloadEventEnd",
    "redirectStart",
    "redirectEnd",
    "fetchStart",
    "domainLookupStart",
    "domainLookupEnd",
    "connectStart",
    "connectEnd",
    "secureConnectionStart",
    "requestStart",
    "responseStart",
    "responseEnd",
    "domLoading",
    "domInteractive",
    "domContentLoadedEventStart",
    "domContentLoadedEventEnd",
    "domComplete",
    "loadEventStart",
    "loadEventEnd",
];

pub(crate) fn is_legacy_timing_name(name: &str) -> bool {
    LEGACY_TIMING_NAMES.contains(&name)
}

fn non_finite_option_error(scope: &mut v8::PinScope<'_, '_>, member: &str, dictionary: &str) {
    crate::webidl::throw_type_error(
        scope,
        &format!(
            "Failed to read the '{member}' property from '{dictionary}': The provided double value is non-finite."
        ),
    );
}

fn optional_finite_number(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    member: &str,
    dictionary: &str,
) -> Option<Result<Option<f64>, ()>> {
    if value.is_undefined() {
        return Some(Ok(None));
    }
    let value = value.number_value(scope)?;
    if !value.is_finite() {
        non_finite_option_error(scope, member, dictionary);
        return Some(Err(()));
    }
    Some(Ok(Some(value)))
}

fn mark(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'mark' on 'Performance': 1 argument required, but only 0 present.",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if snapshot.is_window && is_legacy_timing_name(&name) {
        super::node::throw_dom_exception(
            scope,
            "SyntaxError",
            &format!(
                "Failed to execute 'mark' on 'Performance': '{name}' is part of the PerformanceTiming interface, and cannot be used as a mark name."
            ),
        );
        return;
    }
    let options_value = arguments.get(1);
    let options = if options_value.is_undefined() || options_value.is_null() {
        None
    } else {
        let Ok(options) = v8::Local::<v8::Object>::try_from(options_value) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'mark' on 'Performance': parameter 2 is not of type 'Object'.",
            );
            return;
        };
        Some(options)
    };
    // PerformanceMarkOptions members are converted in Web IDL name order.
    let detail = match options {
        Some(options) => {
            let Some(value) = property(scope, options, "detail") else {
                return;
            };
            if value.is_undefined() {
                v8::null(scope).into()
            } else {
                value
            }
        }
        None => v8::null(scope).into(),
    };
    let default_start = now_for_record(scope, &snapshot);
    let start_time = match options {
        Some(options) => {
            let Some(value) = property(scope, options, "startTime") else {
                return;
            };
            let Some(converted) =
                optional_finite_number(scope, value, "startTime", "PerformanceMarkOptions")
            else {
                return;
            };
            let Ok(converted) = converted else {
                return;
            };
            converted.unwrap_or(default_start)
        }
        None => default_start,
    };
    if start_time < 0.0 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'mark' on 'Performance': '{name}' cannot have a negative time stamp."
            ),
        );
        return;
    }
    let detail = v8::Global::new(scope, detail);
    let Some(detail) = super::performance_mark::structured_clone_detail(scope, detail) else {
        return;
    };
    let detail = v8::Local::new(scope, &detail);
    let entry = match super::performance_mark::create(scope, name, start_time, detail) {
        Ok(entry) => entry,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    push_entry(scope, arguments.this(), entry, "mark");
    result.set(entry.into());
}

fn find_mark_time(
    scope: &v8::PinScope<'_, '_>,
    entries: &[v8::Global<v8::Object>],
    name: &str,
) -> Option<f64> {
    entries.iter().rev().find_map(|entry| {
        let entry = v8::Local::new(scope, entry);
        super::performance_entry::record(scope, entry).and_then(|record| {
            (record.entry_type == "mark" && record.name == name).then_some(record.start_time)
        })
    })
}

enum MeasurePoint {
    Mark(String),
    Timestamp(f64),
}

fn parse_measure_point(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    member: &str,
) -> Option<Result<Option<MeasurePoint>, ()>> {
    if value.is_undefined() {
        return Some(Ok(None));
    }
    if value.is_string() {
        return Some(Ok(Some(MeasurePoint::Mark(
            crate::webidl::value_to_string(scope, value),
        ))));
    }
    let converted = optional_finite_number(scope, value, member, "PerformanceMeasureOptions")?;
    Some(converted.map(|value| value.map(MeasurePoint::Timestamp)))
}

fn resolve_measure_point(
    scope: &mut v8::PinScope<'_, '_>,
    snapshot: &PerformanceRecord,
    measure_name: &str,
    point: &MeasurePoint,
) -> Option<f64> {
    match point {
        MeasurePoint::Timestamp(value) => {
            if *value < 0.0 {
                crate::webidl::throw_type_error(
                    scope,
                    &format!(
                        "Failed to execute 'measure' on 'Performance': '{measure_name}' cannot have a negative time stamp."
                    ),
                );
                None
            } else {
                Some(*value)
            }
        }
        MeasurePoint::Mark(mark_name) => {
            if let Some(value) = find_mark_time(scope, &snapshot.entries, mark_name) {
                return Some(value);
            }
            if is_legacy_timing_name(mark_name) {
                if !snapshot.is_window {
                    crate::webidl::throw_type_error(
                        scope,
                        "A PerformanceTiming attribute was used in a worker.",
                    );
                    return None;
                }
                if mark_name == "navigationStart" {
                    return Some(0.0);
                }
                let timing = v8::Local::new(scope, &snapshot.timing);
                let navigation_start =
                    super::performance_timing::named_value(scope, timing, "navigationStart")?;
                let end = super::performance_timing::named_value(scope, timing, mark_name)?;
                if end == 0.0 {
                    super::node::throw_dom_exception(
                        scope,
                        "InvalidAccessError",
                        &format!(
                            "The PerformanceTiming attribute '{mark_name}' has a value of zero."
                        ),
                    );
                    return None;
                }
                return Some(end - navigation_start);
            }
            super::node::throw_dom_exception(
                scope,
                "SyntaxError",
                &format!(
                    "Failed to execute 'measure' on 'Performance': The mark '{mark_name}' does not exist."
                ),
            );
            None
        }
    }
}

fn measure(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'measure' on 'Performance': 1 argument required, but only 0 present.",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let mut start = 0.0;
    let mut end = now_for_record(scope, &snapshot);
    let mut detail: v8::Local<v8::Value> = v8::null(scope).into();
    let second = arguments.get(1);
    let options = if second.is_undefined() || second.is_null() {
        second.is_null().then(|| None)
    } else {
        v8::Local::<v8::Object>::try_from(second).ok().map(Some)
    };
    if let Some(options) = options {
        let (detail_present, duration, end_point, start_point) = match options {
            Some(options) => {
                // PerformanceMeasureOptions members are converted in Web IDL name order.
                let Some(detail_value) = property(scope, options, "detail") else {
                    return;
                };
                let detail_present = !detail_value.is_undefined();
                if detail_present {
                    detail = detail_value;
                }
                let Some(duration_value) = property(scope, options, "duration") else {
                    return;
                };
                let Some(duration) = optional_finite_number(
                    scope,
                    duration_value,
                    "duration",
                    "PerformanceMeasureOptions",
                ) else {
                    return;
                };
                let Ok(duration) = duration else {
                    return;
                };
                let Some(end_value) = property(scope, options, "end") else {
                    return;
                };
                let Some(end_point) = parse_measure_point(scope, end_value, "end") else {
                    return;
                };
                let Ok(end_point) = end_point else {
                    return;
                };
                let Some(start_value) = property(scope, options, "start") else {
                    return;
                };
                let Some(start_point) = parse_measure_point(scope, start_value, "start") else {
                    return;
                };
                let Ok(start_point) = start_point else {
                    return;
                };
                (detail_present, duration, end_point, start_point)
            }
            None => (false, None, None, None),
        };
        let non_empty =
            detail_present || duration.is_some() || end_point.is_some() || start_point.is_some();
        if non_empty && start_point.is_none() && end_point.is_none() {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'measure' on 'Performance': If a non-empty PerformanceMeasureOptions object was passed, at least one of its 'start' or 'end' properties must be present.",
            );
            return;
        }
        if start_point.is_some() && end_point.is_some() && duration.is_some() {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'measure' on 'Performance': If a non-empty PerformanceMeasureOptions object was passed, it must not have all of its 'start', 'duration', and 'end' properties defined",
            );
            return;
        }
        if !arguments.get(2).is_undefined() && non_empty {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'measure' on 'Performance': Cannot specify an end mark when using a PerformanceMeasureOptions object.",
            );
            return;
        }
        let end_mark = if arguments.get(2).is_undefined() {
            None
        } else {
            Some(MeasurePoint::Mark(crate::webidl::value_to_string(
                scope,
                arguments.get(2),
            )))
        };
        if let Some(duration) = duration
            && duration < 0.0
        {
            crate::webidl::throw_type_error(
                scope,
                &format!(
                    "Failed to execute 'measure' on 'Performance': '{name}' cannot have a negative time stamp."
                ),
            );
            return;
        }
        let resolved_start = match start_point.as_ref() {
            Some(point) => match resolve_measure_point(scope, &snapshot, &name, point) {
                Some(value) => Some(value),
                None => return,
            },
            None => None,
        };
        let resolved_end = match end_mark.as_ref().or(end_point.as_ref()) {
            Some(point) => match resolve_measure_point(scope, &snapshot, &name, point) {
                Some(value) => Some(value),
                None => return,
            },
            None => None,
        };
        match (resolved_start, resolved_end, duration) {
            (Some(value), Some(end_value), _) => {
                start = value;
                end = end_value;
            }
            (Some(value), None, Some(duration)) => {
                start = value;
                end = value + duration;
            }
            (None, Some(value), Some(duration)) => {
                end = value;
                start = value - duration;
            }
            (Some(value), None, None) => start = value,
            (None, Some(value), None) => end = value,
            (None, None, None) => {}
            (None, None, Some(_)) => unreachable!(),
        }
    } else if !second.is_undefined() {
        let start_name = crate::webidl::value_to_string(scope, arguments.get(1));
        let start_point = MeasurePoint::Mark(start_name);
        start = match resolve_measure_point(scope, &snapshot, &name, &start_point) {
            Some(value) => value,
            None => return,
        };
        if !arguments.get(2).is_undefined() {
            let end_name = crate::webidl::value_to_string(scope, arguments.get(2));
            let end_point = MeasurePoint::Mark(end_name);
            end = match resolve_measure_point(scope, &snapshot, &name, &end_point) {
                Some(value) => value,
                None => return,
            };
        }
    }
    let detail = v8::Global::new(scope, detail);
    let Some(detail) = super::performance_mark::structured_clone_detail(scope, detail) else {
        return;
    };
    let detail = v8::Local::new(scope, &detail);
    let entry = match super::performance_measure::create(scope, name, start, end - start, detail) {
        Ok(entry) => entry,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    push_entry(scope, arguments.this(), entry, "measure");
    result.set(entry.into());
}

fn push_entry(
    scope: &mut v8::PinScope<'_, '_>,
    performance: v8::Local<'_, v8::Object>,
    entry: v8::Local<'_, v8::Object>,
    entry_type: &str,
) {
    let entry_global = v8::Global::new(scope, entry);
    if let Some(record) = scope.get_slot_mut::<PerformanceStore>().and_then(|store| {
        store
            .records
            .get_mut(&performance.get_identity_hash().get())
    }) {
        record.entries.push(entry_global);
    }
    super::performance_observer::queue_entry(scope, entry, entry_type);
}

pub(crate) fn add_entry_for_current_realm(
    scope: &mut v8::PinScope<'_, '_>,
    entry: v8::Local<'_, v8::Object>,
    entry_type: &str,
) {
    let realm_id = crate::webidl::realm_id(scope);
    let snapshot = scope.get_slot::<PerformanceStore>().and_then(|store| {
        store
            .records
            .iter()
            .find(|(_, record)| record.realm_id == realm_id)
            .map(|(identity, record)| (*identity, record.clone()))
    });
    let Some((identity, record)) = snapshot else {
        return;
    };
    if record.profile_entries_override {
        return;
    }
    if entry_type == "resource" {
        let resource_count = record
            .entries
            .iter()
            .filter(|candidate| {
                let candidate = v8::Local::new(scope, *candidate);
                super::performance_entry::record(scope, candidate)
                    .is_some_and(|record| record.entry_type == "resource")
            })
            .count();
        if resource_count >= record.resource_timing_buffer_size {
            let target = if let Some(record) = scope
                .get_slot_mut::<PerformanceStore>()
                .and_then(|store| store.records.get_mut(&identity))
            {
                if record.resource_timing_buffer_full_notified {
                    None
                } else {
                    record.resource_timing_buffer_full_notified = true;
                    Some(record.target.clone())
                }
            } else {
                None
            };
            if let Some(target) = target {
                let target = v8::Local::new(scope, &target);
                if let Ok(event) = super::event::create(scope, "resourcetimingbufferfull") {
                    super::event_target::dispatch(scope, target, event);
                }
            }
            super::performance_observer::queue_entry(scope, entry, entry_type);
            return;
        }
    }
    let entry_global = v8::Global::new(scope, entry);
    let added = if let Some(record) = scope
        .get_slot_mut::<PerformanceStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.entries.push(entry_global);
        true
    } else {
        false
    };
    if added {
        super::performance_observer::queue_entry(scope, entry, entry_type);
    }
}

fn current_realm_uses_profile_entries(scope: &v8::PinScope<'_, '_>) -> bool {
    let realm_id = crate::webidl::realm_id(scope);
    scope.get_slot::<PerformanceStore>().is_some_and(|store| {
        store
            .records
            .values()
            .any(|record| record.realm_id == realm_id && record.profile_entries_override)
    })
}

fn set_resource_timing_buffer_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let size = arguments.get(0).uint32_value(scope).unwrap_or(0) as usize;
    if let Some(record) = scope.get_slot_mut::<PerformanceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.resource_timing_buffer_size = size;
        record.resource_timing_buffer_full_notified = false;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "resourcetimingbufferfull" {
        return;
    }
    let handler = record(scope, target).and_then(|record| record.on_resource_timing_buffer_full);
    super::window_event_handler_support::invoke(scope, target, event, handler);
}

fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}
fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    define_value(scope, object, name, v8::Number::new(scope, value).into());
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    define_number(scope, output, "timeOrigin", record.time_origin);
    define_value(
        scope,
        output,
        "timing",
        v8::Local::new(scope, &record.timing).into(),
    );
    define_value(
        scope,
        output,
        "navigation",
        v8::Local::new(scope, &record.navigation).into(),
    );
    result.set(output.into());
}

fn now(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, now_for_record(scope, &record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PerformanceRecord) -> v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_timing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.timing.clone());
}
fn get_navigation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.navigation.clone());
}
fn get_memory(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.memory.clone());
}
fn get_event_counts(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.event_counts.clone());
}
fn get_interaction_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.interaction_count).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PerformanceStore>() {
        store.constructors.remove(&realm_id);
        store
            .records
            .retain(|_, record| record.realm_id != realm_id);
    }
}
