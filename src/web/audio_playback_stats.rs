use std::collections::HashMap;

#[derive(Clone, Default)]
struct PlaybackStatsRecord {
    underrun_duration: f64,
    underrun_events: u64,
    total_duration: f64,
    average_latency: f64,
    minimum_latency: f64,
    maximum_latency: f64,
}

#[derive(Default)]
pub(crate) struct AudioPlaybackStatsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PlaybackStatsRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioPlaybackStatsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioPlaybackStats", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioPlaybackStatsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioPlaybackStats",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "underrunDuration",
        get_underrun_duration,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "underrunEvents",
        get_underrun_events,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "totalDuration", get_total_duration)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "averageLatency",
        get_average_latency,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "minimumLatency",
        get_minimum_latency,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maximumLatency",
        get_maximum_latency,
    )?;
    crate::webidl::define_method(scope, prototype, "resetLatency", 0, reset_latency)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioPlaybackStatsStore>()
        .ok_or_else(|| "AudioPlaybackStats state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create AudioPlaybackStats".to_owned());
    }
    scope
        .get_slot_mut::<AudioPlaybackStatsStore>()
        .ok_or_else(|| "AudioPlaybackStats state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            PlaybackStatsRecord::default(),
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PlaybackStatsRecord> {
    scope
        .get_slot::<AudioPlaybackStatsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PlaybackStatsRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_underrun_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.underrun_duration)
}
fn get_underrun_events(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Number::new(s, v.underrun_events as f64).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_total_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.total_duration)
}
fn get_average_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.average_latency)
}
fn get_minimum_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.minimum_latency)
}
fn get_maximum_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.maximum_latency)
}

fn reset_latency(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<AudioPlaybackStatsStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.average_latency = 0.0;
        record.minimum_latency = 0.0;
        record.maximum_latency = 0.0;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
    }
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
    let object = v8::Object::new(scope);
    define_number(scope, object, "underrunDuration", record.underrun_duration);
    define_number(
        scope,
        object,
        "underrunEvents",
        record.underrun_events as f64,
    );
    define_number(scope, object, "totalDuration", record.total_duration);
    define_number(scope, object, "averageLatency", record.average_latency);
    define_number(scope, object, "minimumLatency", record.minimum_latency);
    define_number(scope, object, "maximumLatency", record.maximum_latency);
    result.set(object.into());
}
