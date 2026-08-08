use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamTrackAudioStatsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioStatsRecord>,
}

#[derive(Clone, Copy, Default)]
struct AudioStatsRecord {
    delivered_frames: u64,
    delivered_frames_duration: f64,
    total_frames: u64,
    total_frames_duration: f64,
    latency: f64,
    average_latency: f64,
    minimum_latency: f64,
    maximum_latency: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamTrackAudioStatsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamTrackAudioStats", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamTrackAudioStatsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamTrackAudioStats",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "deliveredFrames",
        get_delivered_frames,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "deliveredFramesDuration",
        get_delivered_frames_duration,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "totalFrames", get_total_frames)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "totalFramesDuration",
        get_total_frames_duration,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "latency", get_latency)?;
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
        .get_slot_mut::<MediaStreamTrackAudioStatsStore>()
        .ok_or_else(|| "MediaStreamTrackAudioStats state was not prepared".to_owned())?
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
        return Err("cannot create MediaStreamTrackAudioStats".to_owned());
    }
    scope
        .get_slot_mut::<MediaStreamTrackAudioStatsStore>()
        .ok_or_else(|| "MediaStreamTrackAudioStats state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AudioStatsRecord::default(),
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioStatsRecord> {
    scope
        .get_slot::<MediaStreamTrackAudioStatsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(AudioStatsRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_delivered_frames(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.delivered_frames as f64);
}
fn get_delivered_frames_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.delivered_frames_duration);
}
fn get_total_frames(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.total_frames as f64);
}
fn get_total_frames_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.total_frames_duration);
}
fn get_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.latency);
}
fn get_average_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.average_latency);
}
fn get_minimum_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.minimum_latency);
}
fn get_maximum_latency(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.maximum_latency);
}

fn reset_latency(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot_mut::<MediaStreamTrackAudioStatsStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.latency = 0.0;
    record.average_latency = 0.0;
    record.minimum_latency = 0.0;
    record.maximum_latency = 0.0;
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
    define_number(
        scope,
        object,
        "deliveredFrames",
        record.delivered_frames as f64,
    );
    define_number(
        scope,
        object,
        "deliveredFramesDuration",
        record.delivered_frames_duration,
    );
    define_number(scope, object, "totalFrames", record.total_frames as f64);
    define_number(
        scope,
        object,
        "totalFramesDuration",
        record.total_frames_duration,
    );
    define_number(scope, object, "latency", record.latency);
    define_number(scope, object, "averageLatency", record.average_latency);
    define_number(scope, object, "minimumLatency", record.minimum_latency);
    define_number(scope, object, "maximumLatency", record.maximum_latency);
    result.set(object.into());
}

fn define_number(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let number = v8::Number::new(scope, value);
        let _ = object.set(scope, key.into(), number.into());
    }
}
