use std::collections::{HashMap, HashSet};
#[derive(Clone, Default)]
struct DevicesRecord {
    ondevicechange: Option<v8::Global<v8::Value>>,
    capture_handle: Option<String>,
}
#[derive(Default)]
pub(crate) struct MediaDevicesStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DevicesRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaDevicesStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaDevices", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MediaDevicesStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaDevices",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "ondevicechange", get_handler, set_handler)?;
    crate::webidl::define_method(scope, prototype, "enumerateDevices", 0, enumerate_devices)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getSupportedConstraints",
        0,
        get_supported_constraints,
    )?;
    crate::webidl::define_method(scope, prototype, "getUserMedia", 0, get_user_media)?;
    crate::webidl::define_method(scope, prototype, "getDisplayMedia", 0, get_display_media)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setCaptureHandleConfig",
        0,
        set_capture_handle_config,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MediaDevicesStore>()
        .ok_or_else(|| "MediaDevices state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaDevices".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<MediaDevicesStore>()
        .ok_or_else(|| "MediaDevices state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), DevicesRecord::default());
    Ok(object)
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DevicesRecord> {
    scope
        .get_slot::<MediaDevicesStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.ondevicechange),
        r,
    )
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(record) = s
        .get_slot_mut::<MediaDevicesStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.ondevicechange = handler
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn resolve(
    s: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(s, value) {
        r.set(promise.into())
    }
}
fn enumerate_devices(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let fingerprint = crate::fingerprint::edge(s);
    let devices = fingerprint.media.devices.clone();
    let permissions = fingerprint.permissions.clone();
    let mut visible = Vec::new();
    let mut masked_kinds = HashSet::new();
    for mut device in devices {
        let permission = match device.kind.as_str() {
            "audioinput" => permissions.microphone.as_str(),
            "videoinput" => permissions.camera.as_str(),
            "audiooutput" => permissions.speaker_selection.as_str(),
            _ => "denied",
        };
        if permission != "granted" {
            if !masked_kinds.insert(device.kind.clone()) {
                continue;
            }
            device.device_id.clear();
            device.label.clear();
            device.group_id.clear();
        }
        visible.push(device);
    }
    let array = v8::Array::new(s, visible.len() as i32);
    for (index, device) in visible.into_iter().enumerate() {
        if let Ok(info) = super::media_device_info::create(
            s,
            device.device_id,
            device.kind,
            device.label,
            device.group_id,
        ) {
            let _ = array.set_index(s, index as u32, info.into());
        }
    }
    resolve(s, array.into(), r)
}
fn get_supported_constraints(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let constraints = crate::fingerprint::edge(s)
        .media
        .supported_constraints
        .clone();
    let object = v8::Object::new(s);
    for constraint in constraints {
        if let Some(key) = v8::String::new(s, &constraint) {
            let truth = v8::Boolean::new(s, true);
            let _ = object.set(s, key.into(), truth.into());
        }
    }
    r.set(object.into())
}
fn reject(
    s: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::rejected_promise(s, value) {
        r.set(promise.into());
    }
}

fn reject_not_allowed(s: &mut v8::PinScope<'_, '_>, r: v8::ReturnValue<'_>) {
    if let Ok(exception) = super::dom_exception::create(
        s,
        "Permission denied".to_owned(),
        "NotAllowedError".to_owned(),
    ) {
        reject(s, exception.into(), r);
    }
}

fn constraint_requested(
    s: &mut v8::PinScope<'_, '_>,
    constraints: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    let Some(key) = v8::String::new(s, name) else {
        return false;
    };
    let Some(value) = constraints.get(s, key.into()) else {
        return false;
    };
    if value.is_undefined() || value.is_null() {
        false
    } else if value.is_boolean() {
        value.boolean_value(s)
    } else {
        true
    }
}

fn capture_stream(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let Ok(constraints) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        let Some(message) = v8::String::new(s, "At least one of audio and video must be requested")
        else {
            return;
        };
        reject(s, v8::Exception::type_error(s, message), r);
        return;
    };
    let audio = constraint_requested(s, constraints, "audio");
    let video = constraint_requested(s, constraints, "video");
    if !audio && !video {
        let Some(message) = v8::String::new(s, "At least one of audio and video must be requested")
        else {
            return;
        };
        reject(s, v8::Exception::type_error(s, message), r);
        return;
    }
    let fingerprint = crate::fingerprint::edge(s);
    if (audio && fingerprint.permissions.microphone != "granted")
        || (video && fingerprint.permissions.camera != "granted")
    {
        reject_not_allowed(s, r);
        return;
    }
    let devices = fingerprint.media.devices.clone();
    let mut tracks = Vec::new();
    if audio {
        let label = devices
            .iter()
            .find(|device| device.kind == "audioinput")
            .map(|device| device.label.clone())
            .filter(|label| !label.is_empty());
        if let Ok(track) = super::media_stream_track::create(s, "audio", label) {
            tracks.push(track);
        }
    }
    if video {
        let label = devices
            .iter()
            .find(|device| device.kind == "videoinput")
            .map(|device| device.label.clone())
            .filter(|label| !label.is_empty());
        if let Ok(track) = super::media_stream_track::create(s, "video", label) {
            tracks.push(track);
        }
    }
    if let Ok(stream) = super::media_stream::create_with_tracks(s, &tracks) {
        resolve(s, stream.into(), r);
    }
}
fn get_user_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    capture_stream(s, a, r)
}
fn get_display_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    reject_not_allowed(s, r)
}
fn set_capture_handle_config(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handle = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .and_then(|o| v8::String::new(s, "handle").and_then(|k| o.get(s, k.into())))
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(s, v));
    if let Some(record) = s
        .get_slot_mut::<MediaDevicesStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.capture_handle = handle
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
