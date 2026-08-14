use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceEventTimingStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, EventTimingRecord>,
}

#[derive(Clone)]
pub(crate) struct EventTimingRecord {
    pub(crate) processing_start: f64,
    pub(crate) processing_end: f64,
    pub(crate) cancelable: bool,
    pub(crate) target: Option<v8::Global<v8::Value>>,
    pub(crate) interaction_id: i32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceEventTimingStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceEventTiming", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceEventTimingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceEventTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::performance_event_timing_processing_start_property::define(scope, prototype)?;
    super::performance_event_timing_processing_end_property::define(scope, prototype)?;
    super::performance_event_timing_cancelable_property::define(scope, prototype)?;
    super::performance_event_timing_target_property::define(scope, prototype)?;
    super::performance_event_timing_interaction_id_property::define(scope, prototype)?;
    super::performance_event_timing_to_json::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceEventTimingStore>()
        .ok_or_else(|| "PerformanceEventTiming state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'PerformanceEventTiming': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    start_time: f64,
    duration: f64,
    cancelable: bool,
    target: Option<v8::Local<'_, v8::Value>>,
    interaction_id: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_with_entry_type(
        scope,
        name,
        "event",
        start_time,
        duration,
        cancelable,
        target,
        interaction_id,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_with_entry_type<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    entry_type: &str,
    start_time: f64,
    duration: f64,
    cancelable: bool,
    target: Option<v8::Local<'_, v8::Value>>,
    interaction_id: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceEventTiming".to_owned());
    }
    super::performance_entry::attach(
        scope,
        timing,
        name,
        entry_type.to_owned(),
        start_time,
        duration,
    );
    let target = target.map(|value| v8::Global::new(scope, value));
    scope
        .get_slot_mut::<PerformanceEventTimingStore>()
        .ok_or_else(|| "PerformanceEventTiming state was not prepared".to_owned())?
        .records
        .insert(
            timing.get_identity_hash().get(),
            EventTimingRecord {
                processing_start: start_time,
                processing_end: start_time + duration,
                cancelable,
                target,
                interaction_id,
            },
        );
    Ok(timing)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<EventTimingRecord> {
    scope
        .get_slot::<PerformanceEventTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn set_processing_times(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    processing_start: f64,
    processing_end: f64,
) {
    if let Some(record) = scope
        .get_slot_mut::<PerformanceEventTimingStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.processing_start = processing_start;
        record.processing_end = processing_end.max(processing_start);
    }
}
pub(crate) fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&EventTimingRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_processing_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.processing_start);
}
pub(crate) fn get_processing_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.processing_end);
}

pub(crate) fn get_cancelable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.cancelable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_interaction_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.interaction_id).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(target) = record.target {
        result.set(v8::Local::new(scope, &target));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}
pub(crate) fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(base) = super::performance_entry::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = super::performance_entry::to_object(scope, &base);
    define_value(
        scope,
        output,
        "processingStart",
        v8::Number::new(scope, record.processing_start).into(),
    );
    define_value(
        scope,
        output,
        "processingEnd",
        v8::Number::new(scope, record.processing_end).into(),
    );
    define_value(
        scope,
        output,
        "cancelable",
        v8::Boolean::new(scope, record.cancelable).into(),
    );
    if let Some(target) = record.target {
        define_value(scope, output, "target", v8::Local::new(scope, &target));
    } else {
        define_value(scope, output, "target", v8::null(scope).into());
    }
    define_value(
        scope,
        output,
        "interactionId",
        v8::Integer::new(scope, record.interaction_id).into(),
    );
    result.set(output.into());
}
