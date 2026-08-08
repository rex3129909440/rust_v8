use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamTrackVideoStatsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, VideoStatsRecord>,
}

#[derive(Clone, Copy)]
struct VideoStatsRecord {
    delivered_frames: u64,
    discarded_frames: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamTrackVideoStatsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamTrackVideoStats", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamTrackVideoStatsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamTrackVideoStats",
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
        "discardedFrames",
        get_discarded_frames,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "totalFrames", get_total_frames)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamTrackVideoStatsStore>()
        .ok_or_else(|| "MediaStreamTrackVideoStats state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    delivered_frames: u64,
    discarded_frames: u64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaStreamTrackVideoStats".to_owned());
    }
    scope
        .get_slot_mut::<MediaStreamTrackVideoStatsStore>()
        .ok_or_else(|| "MediaStreamTrackVideoStats state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            VideoStatsRecord {
                delivered_frames,
                discarded_frames,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<VideoStatsRecord> {
    scope
        .get_slot::<MediaStreamTrackVideoStatsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(VideoStatsRecord) -> u64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(record) as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_delivered_frames(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_number(scope, arguments, result, |record| record.delivered_frames);
}

fn get_discarded_frames(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_number(scope, arguments, result, |record| record.discarded_frames);
}

fn get_total_frames(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_number(scope, arguments, result, |record| {
        record.delivered_frames + record.discarded_frames
    });
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
    define_number(scope, object, "deliveredFrames", record.delivered_frames);
    define_number(scope, object, "discardedFrames", record.discarded_frames);
    define_number(
        scope,
        object,
        "totalFrames",
        record.delivered_frames + record.discarded_frames,
    );
    result.set(object.into());
}

fn define_number(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: u64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let number = v8::Number::new(scope, value as f64);
        let _ = object.set(scope, key.into(), number.into());
    }
}
