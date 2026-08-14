use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, StreamRecord>,
    next_id: u64,
}

#[derive(Clone)]
struct StreamRecord {
    id: String,
    tracks: Vec<Track>,
    on_add_track: Option<v8::Global<v8::Value>>,
    on_remove_track: Option<v8::Global<v8::Value>>,
    on_active: Option<v8::Global<v8::Value>>,
    on_inactive: Option<v8::Global<v8::Value>>,
}

#[derive(Clone)]
struct Track {
    id: i32,
    value: v8::Global<v8::Object>,
    kind: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamStore::default());
}

#[allow(dead_code)]
pub(crate) fn install_standard_name(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStream", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStream",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "active", get_active)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onaddtrack",
        get_on_add_track,
        set_on_add_track,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onremovetrack",
        get_on_remove_track,
        set_on_remove_track,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onactive", get_on_active, set_on_active)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oninactive",
        get_on_inactive,
        set_on_inactive,
    )?;
    crate::webidl::define_method(scope, prototype, "addTrack", 1, add_track)?;
    crate::webidl::define_method(scope, prototype, "clone", 0, clone_stream)?;
    crate::webidl::define_method(scope, prototype, "getAudioTracks", 0, get_audio_tracks)?;
    crate::webidl::define_method(scope, prototype, "getTrackById", 1, get_track_by_id)?;
    crate::webidl::define_method(scope, prototype, "getTracks", 0, get_tracks)?;
    crate::webidl::define_method(scope, prototype, "getVideoTracks", 0, get_video_tracks)?;
    crate::webidl::define_method(scope, prototype, "removeTrack", 1, remove_track)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamStore>()
        .ok_or_else(|| "MediaStream state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_with_tracks<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    tracks: &[v8::Local<'s, v8::Object>],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaStream".to_owned());
    }
    let mut stored_tracks = Vec::with_capacity(tracks.len());
    for track in tracks {
        stored_tracks.push(track_from_value(scope, (*track).into())?);
    }
    attach(scope, object, stored_tracks);
    Ok(object)
}

pub(crate) fn is_stream(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<MediaStreamStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn has_audio_track(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, object)
        .is_some_and(|record| record.tracks.iter().any(|track| track.kind == "audio"))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaStream': Please use the 'new' operator",
        );
        return;
    }
    let tracks = if arguments.get(0).is_undefined() {
        Vec::new()
    } else {
        match tracks_from_value(scope, arguments.get(0)) {
            Ok(tracks) => tracks,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    };
    let object = arguments.this();
    attach(scope, object, tracks);
    result.set(object.into());
}

fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, tracks: Vec<Track>) {
    super::event_target::attach(scope, object);
    let store = scope
        .get_slot_mut::<MediaStreamStore>()
        .expect("MediaStream state");
    store.next_id += 1;
    let record = StreamRecord {
        id: format!(
            "{:08x}-0000-4000-8000-{:012x}",
            store.next_id, store.next_id
        ),
        tracks,
        on_add_track: None,
        on_remove_track: None,
        on_active: None,
        on_inactive: None,
    };
    store
        .records
        .insert(object.get_identity_hash().get(), record);
}

fn tracks_from_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Vec<Track>, String> {
    if let Ok(stream) = v8::Local::<v8::Object>::try_from(value)
        && let Some(record) = record(scope, stream)
    {
        return Ok(record.tracks);
    }
    let array = v8::Local::<v8::Array>::try_from(value)
        .map_err(|_| "Failed to convert value to a sequence".to_owned())?;
    let mut tracks = Vec::with_capacity(array.length() as usize);
    for index in 0..array.length() {
        let value = array
            .get_index(scope, index)
            .ok_or_else(|| "Cannot read MediaStream track".to_owned())?;
        tracks.push(track_from_value(scope, value)?);
    }
    Ok(tracks)
}

fn track_from_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Track, String> {
    let object = v8::Local::<v8::Object>::try_from(value)
        .map_err(|_| "Failed to convert value to 'MediaStreamTrack'.".to_owned())?;
    if !super::media_stream_track::is_track(scope, object) {
        return Err("Failed to convert value to 'MediaStreamTrack'.".to_owned());
    }
    let kind = super::media_stream_track::kind(scope, object)
        .ok_or_else(|| "Failed to convert value to 'MediaStreamTrack'.".to_owned())?;
    Ok(Track {
        id: object.get_identity_hash().get(),
        value: v8::Global::new(scope, object),
        kind,
    })
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<StreamRecord> {
    scope
        .get_slot::<MediaStreamStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .cloned()
}

fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    handler: Option<v8::Global<v8::Value>>,
) {
    if let Some(handler) = handler {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn normalized_handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    value.is_function().then(|| v8::Global::new(scope, value))
}

fn get_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.id) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_active(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let active = record.tracks.iter().any(|track| {
            let track = v8::Local::new(scope, &track.value);
            let Some(key) = v8::String::new(scope, "readyState") else {
                return false;
            };
            track
                .get(scope, key.into())
                .is_some_and(|value| crate::webidl::value_to_string(scope, value) == "live")
        });
        result.set(v8::Boolean::new(scope, active).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_add_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_handler(scope, &mut result, record.on_add_track);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_on_add_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = normalized_handler(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<MediaStreamStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.on_add_track = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_remove_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_handler(scope, &mut result, record.on_remove_track);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_on_remove_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = normalized_handler(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<MediaStreamStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.on_remove_track = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_active(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_handler(scope, &mut result, record.on_active);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_on_active(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = normalized_handler(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<MediaStreamStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.on_active = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_inactive(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_handler(scope, &mut result, record.on_inactive);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_on_inactive(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = normalized_handler(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<MediaStreamStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.on_inactive = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn add_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "MediaStream.addTrack requires 1 argument");
        return;
    }
    let track = match track_from_value(scope, arguments.get(0)) {
        Ok(track) => track,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if let Some(record) = scope.get_slot_mut::<MediaStreamStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        if !record.tracks.iter().any(|existing| existing.id == track.id) {
            record.tracks.push(track);
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn clone_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut tracks = Vec::with_capacity(record.tracks.len());
    for track in record.tracks {
        let original = v8::Local::new(scope, &track.value);
        let cloned = match super::media_stream_track::clone_object(scope, original) {
            Ok(cloned) => cloned,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        tracks.push(cloned);
    }
    match create_with_tracks(scope, &tracks) {
        Ok(stream) => result.set(stream.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn return_tracks(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    tracks: Vec<Track>,
) {
    let array = v8::Array::new(scope, tracks.len() as i32);
    for (index, track) in tracks.into_iter().enumerate() {
        let value = v8::Local::new(scope, &track.value);
        let _ = array.set_index(scope, index as u32, value.into());
    }
    result.set(array.into());
}

fn get_audio_tracks(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    return_tracks(
        scope,
        &mut result,
        record
            .tracks
            .into_iter()
            .filter(|track| track.kind == "audio")
            .collect(),
    );
}

fn get_track_by_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let id = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    for track in record.tracks {
        let value = v8::Local::new(scope, &track.value);
        let key = v8::String::new(scope, "id").unwrap();
        let track_id = value
            .get(scope, key.into())
            .map(|value| crate::webidl::value_to_string(scope, value))
            .unwrap_or_default();
        if track_id == id {
            result.set(value.into());
            return;
        }
    }
    result.set(v8::null(scope).into());
}

fn get_tracks(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_tracks(scope, &mut result, record.tracks);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_video_tracks(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    return_tracks(
        scope,
        &mut result,
        record
            .tracks
            .into_iter()
            .filter(|track| track.kind == "video")
            .collect(),
    );
}

fn remove_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(track) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "MediaStreamTrack object required");
        return;
    };
    let id = track.get_identity_hash().get();
    if let Some(record) = scope.get_slot_mut::<MediaStreamStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.tracks.retain(|track| track.id != id);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
