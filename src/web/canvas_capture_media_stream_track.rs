use std::collections::HashMap;

#[derive(Clone)]
struct CanvasCaptureRecord {
    canvas: v8::Global<v8::Object>,
    requested_frames: u64,
}

#[derive(Default)]
pub(crate) struct CanvasCaptureMediaStreamTrackStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CanvasCaptureRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CanvasCaptureMediaStreamTrackStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CanvasCaptureMediaStreamTrack", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CanvasCaptureMediaStreamTrackStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::media_stream_track::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "CanvasCaptureMediaStreamTrack",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "canvas", get_canvas)?;
    crate::webidl::define_method(scope, prototype, "requestFrame", 0, request_frame)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CanvasCaptureMediaStreamTrackStore>()
        .ok_or_else(|| "CanvasCaptureMediaStreamTrack state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    canvas: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let track = super::media_stream_track::create(
        scope,
        "video",
        Some("CanvasCaptureMediaStreamTrack".to_owned()),
    )?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    if crate::webidl::set_platform_prototype(scope, track, prototype.into()) != Some(true) {
        return Err("cannot create CanvasCaptureMediaStreamTrack".to_owned());
    }
    let canvas = v8::Global::new(scope, canvas);
    scope
        .get_slot_mut::<CanvasCaptureMediaStreamTrackStore>()
        .ok_or_else(|| "CanvasCaptureMediaStreamTrack state was not prepared".to_owned())?
        .records
        .insert(
            track.get_identity_hash().get(),
            CanvasCaptureRecord {
                canvas,
                requested_frames: 0,
            },
        );
    Ok(track)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CanvasCaptureMediaStreamTrack': Illegal constructor",
    );
}

fn get_canvas(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot::<CanvasCaptureMediaStreamTrackStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, &record.canvas).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn request_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<CanvasCaptureMediaStreamTrackStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.requested_frames += 1;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
