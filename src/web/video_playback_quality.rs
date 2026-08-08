use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct VideoPlaybackQualityStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, QualityRecord>,
}

#[derive(Clone)]
struct QualityRecord {
    creation_time: f64,
    total_video_frames: u64,
    dropped_video_frames: u64,
    corrupted_video_frames: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(VideoPlaybackQualityStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "VideoPlaybackQuality", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<VideoPlaybackQualityStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "VideoPlaybackQuality",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "creationTime", get_creation_time)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "totalVideoFrames",
        get_total_video_frames,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "droppedVideoFrames",
        get_dropped_video_frames,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "corruptedVideoFrames",
        get_corrupted_video_frames,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<VideoPlaybackQualityStore>()
        .ok_or_else(|| "VideoPlaybackQuality state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    creation_time: f64,
    total_video_frames: u64,
    dropped_video_frames: u64,
    corrupted_video_frames: u64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create VideoPlaybackQuality".to_owned());
    }
    scope
        .get_slot_mut::<VideoPlaybackQualityStore>()
        .ok_or_else(|| "VideoPlaybackQuality state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            QualityRecord {
                creation_time,
                total_video_frames,
                dropped_video_frames,
                corrupted_video_frames,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<QualityRecord> {
    scope
        .get_slot::<VideoPlaybackQualityStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&QualityRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_creation_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.creation_time)
}
fn get_total_video_frames(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.total_video_frames as f64)
}
fn get_dropped_video_frames(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.dropped_video_frames as f64)
}
fn get_corrupted_video_frames(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.corrupted_video_frames as f64)
}
