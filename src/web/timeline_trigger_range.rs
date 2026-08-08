use std::collections::HashMap;

#[derive(Clone)]
struct TimelineTriggerRangeRecord {
    timeline: v8::Global<v8::Value>,
    activation_range_start: v8::Global<v8::Value>,
    activation_range_end: v8::Global<v8::Value>,
    active_range_start: v8::Global<v8::Value>,
    active_range_end: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct TimelineTriggerRangeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TimelineTriggerRangeRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TimelineTriggerRangeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TimelineTriggerRange", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<TimelineTriggerRangeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TimelineTriggerRange",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timeline", get_timeline)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "activationRangeStart",
        get_activation_range_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "activationRangeEnd",
        get_activation_range_end,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "activeRangeStart",
        get_active_range_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "activeRangeEnd",
        get_active_range_end,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TimelineTriggerRangeStore>()
        .ok_or_else(|| "TimelineTriggerRange state was not prepared".to_owned())?
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
        "Failed to construct 'TimelineTriggerRange': Illegal constructor",
    )
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    init: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create TimelineTriggerRange".to_owned());
    }
    let timeline = property(scope, init, "timeline")
        .filter(|value| !value.is_undefined())
        .unwrap_or_else(|| document_timeline(scope));
    let activation_range_start = property_or_text(scope, init, "activationRangeStart", "normal");
    let activation_range_end = property_or_text(scope, init, "activationRangeEnd", "normal");
    let active_range_start = property_or_text(scope, init, "activeRangeStart", "auto");
    let active_range_end = property_or_text(scope, init, "activeRangeEnd", "auto");
    let record = TimelineTriggerRangeRecord {
        timeline: v8::Global::new(scope, timeline),
        activation_range_start: v8::Global::new(scope, activation_range_start),
        activation_range_end: v8::Global::new(scope, activation_range_end),
        active_range_start: v8::Global::new(scope, active_range_start),
        active_range_end: v8::Global::new(scope, active_range_end),
    };
    scope
        .get_slot_mut::<TimelineTriggerRangeStore>()
        .ok_or_else(|| "TimelineTriggerRange state is unavailable".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn property_or_text<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    default: &str,
) -> v8::Local<'s, v8::Value> {
    property(scope, object, name)
        .filter(|value| !value.is_undefined())
        .or_else(|| v8::String::new(scope, default).map(Into::into))
        .unwrap_or_else(|| v8::undefined(scope).into())
}

fn document_timeline<'s>(scope: &mut v8::PinScope<'s, '_>) -> v8::Local<'s, v8::Value> {
    let global = scope.get_current_context().global(scope);
    v8::String::new(scope, "document")
        .and_then(|key| global.get(scope, key.into()))
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .and_then(|document| {
            v8::String::new(scope, "timeline").and_then(|key| document.get(scope, key.into()))
        })
        .unwrap_or_else(|| v8::null(scope).into())
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TimelineTriggerRangeRecord> {
    scope
        .get_slot::<TimelineTriggerRangeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TimelineTriggerRangeRecord) -> &v8::Global<v8::Value>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, select(&record)));
}
fn get_timeline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| &record.timeline)
}
fn get_activation_range_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| &record.activation_range_start)
}
fn get_activation_range_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| &record.activation_range_end)
}
fn get_active_range_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| &record.active_range_start)
}
fn get_active_range_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_value(s, a, r, |record| &record.active_range_end)
}
