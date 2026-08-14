use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamTrackProcessorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ProcessorRecord>,
}

#[derive(Clone)]
struct ProcessorRecord {
    readable: v8::Global<v8::Object>,
    total_frames: u64,
    discarded_frames: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamTrackProcessorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamTrackProcessor", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamTrackProcessorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamTrackProcessor",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readable", get_readable)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "totalFrames", get_total_frames)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "discardedFrames",
        get_discarded_frames,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamTrackProcessorStore>()
        .ok_or_else(|| "MediaStreamTrackProcessor state was not prepared".to_owned())?
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
            "Failed to construct 'MediaStreamTrackProcessor': 1 argument required, but only 0 present.",
        );
        return;
    }
    if arguments.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamTrackProcessor': The provided value is not of type 'MediaStreamTrackProcessorInit'.",
        );
        return;
    }
    let track = match track_from_argument(scope, arguments.get(0)) {
        Ok(track) => track,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let track = v8::Local::new(scope, &track);
    if !super::media_stream_track::is_track(scope, track) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStreamTrackProcessor': Overload resolution failed.",
        );
        return;
    }
    let readable = match super::readable_stream::create_empty(scope) {
        Ok(readable) => readable,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let readable = v8::Global::new(scope, readable);
    scope
        .get_slot_mut::<MediaStreamTrackProcessorStore>()
        .expect("MediaStreamTrackProcessor state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ProcessorRecord {
                readable,
                total_frames: 0,
                discarded_frames: 0,
            },
        );
    result.set(arguments.this().into());
}

fn track_from_argument(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Global<v8::Object>, String> {
    let object = v8::Local::<v8::Object>::try_from(value).map_err(|_| {
        "Failed to construct 'MediaStreamTrackProcessor': Overload resolution failed.".to_owned()
    })?;
    if super::media_stream_track::is_track(scope, object) {
        return Ok(v8::Global::new(scope, object));
    }
    let key = v8::String::new(scope, "track").ok_or_else(|| "cannot create key".to_owned())?;
    let track = object.get(scope, key.into()).ok_or_else(|| {
        "Failed to construct 'MediaStreamTrackProcessor': Failed to read the 'track' property from 'MediaStreamTrackProcessorInit': Required member is undefined.".to_owned()
    })?;
    if track.is_undefined() {
        return Err(
            "Failed to construct 'MediaStreamTrackProcessor': Failed to read the 'track' property from 'MediaStreamTrackProcessorInit': Required member is undefined.".to_owned(),
        );
    }
    v8::Local::<v8::Object>::try_from(track)
        .map(|track| v8::Global::new(scope, track))
        .map_err(|_| {
            "Failed to construct 'MediaStreamTrackProcessor': Overload resolution failed."
                .to_owned()
        })
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ProcessorRecord> {
    scope
        .get_slot::<MediaStreamTrackProcessorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_readable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.readable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ProcessorRecord) -> u64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record) as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_total_frames(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_count(scope, arguments, result, |record| record.total_frames);
}

fn get_discarded_frames(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_count(scope, arguments, result, |record| record.discarded_frames);
}
