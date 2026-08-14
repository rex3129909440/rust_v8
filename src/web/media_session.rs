use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
struct MediaSessionRecord {
    metadata: Option<v8::Global<v8::Object>>,
    playback_state: String,
    actions: HashMap<String, v8::Global<v8::Function>>,
    camera_active: bool,
    microphone_active: bool,
    duration: Option<f64>,
    playback_rate: f64,
    position: f64,
}

#[derive(Default)]
pub(crate) struct MediaSessionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MediaSessionRecord>,
    native_objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaSessionStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaSession", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = scope
        .get_slot::<MediaSessionStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let c = crate::webidl::create_function(
        scope,
        "MediaSession",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_accessor(scope, p, "metadata", get_metadata, set_metadata)?;
    crate::webidl::define_accessor(
        scope,
        p,
        "playbackState",
        get_playback_state,
        set_playback_state,
    )?;
    crate::webidl::define_method(scope, p, "setActionHandler", 2, set_action_handler)?;
    crate::webidl::define_method(scope, p, "setCameraActive", 1, set_camera_active)?;
    crate::webidl::define_method(scope, p, "setMicrophoneActive", 1, set_microphone_active)?;
    crate::webidl::define_method(scope, p, "setPositionState", 0, set_position_state)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<MediaSessionStore>()
        .ok_or_else(|| "MediaSession state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create MediaSession".to_owned());
    }
    let id = o.get_identity_hash().get();
    let store = scope
        .get_slot_mut::<MediaSessionStore>()
        .ok_or_else(|| "MediaSession state was not prepared".to_owned())?;
    store.native_objects.insert(id);
    store.records.insert(
        id,
        MediaSessionRecord {
            playback_state: "none".to_owned(),
            playback_rate: 1.0,
            ..MediaSessionRecord::default()
        },
    );
    Ok(o)
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<MediaSessionRecord> {
    scope
        .get_slot::<MediaSessionStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    f: impl FnOnce(&mut MediaSessionRecord),
) -> bool {
    let Some(r) = scope
        .get_slot_mut::<MediaSessionStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    else {
        return false;
    };
    f(r);
    true
}
fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Failed to construct 'MediaSession': Illegal constructor")
}
fn get_metadata(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(s, a.this()) {
        Some(x) => match &x.metadata {
            Some(v) => r.set(v8::Local::new(s, v).into()),
            None => r.set(v8::null(s).into()),
        },
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
fn set_metadata(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0);
    let metadata = if value.is_null() {
        None
    } else {
        let Ok(o) = v8::Local::<v8::Object>::try_from(value) else {
            crate::webidl::throw_type_error(s, "metadata must be MediaMetadata or null");
            return;
        };
        if !super::media_metadata::is_instance(s, o) {
            crate::webidl::throw_type_error(s, "metadata must be MediaMetadata or null");
            return;
        }
        Some(v8::Global::new(s, o))
    };
    if !update(s, a.this(), |x| x.metadata = metadata) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_playback_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(s, &x.playback_state) {
        r.set(v.into())
    }
}
fn set_playback_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !matches!(value.as_str(), "none" | "paused" | "playing") {
        crate::webidl::throw_type_error(s, "Invalid MediaSessionPlaybackState");
        return;
    }
    if !update(s, a.this(), |x| x.playback_state = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_action_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.length() < 2 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'setActionHandler' on 'MediaSession': 2 arguments required.",
        );
        return;
    }
    let action = crate::webidl::value_to_string(s, a.get(0));
    let handler = if a.get(1).is_null() {
        None
    } else {
        let Ok(f) = v8::Local::<v8::Function>::try_from(a.get(1)) else {
            crate::webidl::throw_type_error(s, "MediaSession action handler must be a function");
            return;
        };
        Some(v8::Global::new(s, f))
    };
    if !update(s, a.this(), |x| {
        if let Some(handler) = handler {
            x.actions.insert(action, handler);
        } else {
            x.actions.remove(&action);
        }
    }) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_camera_active(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    set_device_active(s, a, &mut r, true)
}
fn set_microphone_active(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    set_device_active(s, a, &mut r, false)
}
fn set_device_active(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: &mut v8::ReturnValue<'_>,
    camera: bool,
) {
    let active = a.get(0).boolean_value(s);
    if !update(s, a.this(), |x| {
        if camera {
            x.camera_active = active
        } else {
            x.microphone_active = active
        }
    }) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let u = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, u.into()) {
        r.set(p.into())
    }
}
fn set_position_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    if a.length() == 0 || a.get(0).is_undefined() {
        let _ = update(s, a.this(), |x| x.duration = None);
        return;
    }
    let Ok(o) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "Position state must be an object");
        return;
    };
    let duration = number_property(s, o, "duration").unwrap_or(0.0);
    let rate = number_property(s, o, "playbackRate").unwrap_or(1.0);
    let position = number_property(s, o, "position").unwrap_or(0.0);
    if duration <= 0.0 || rate == 0.0 || position < 0.0 || position > duration {
        crate::webidl::throw_type_error(s, "Invalid media position state");
        return;
    }
    let _ = update(s, a.this(), |x| {
        x.duration = Some(duration);
        x.playback_rate = rate;
        x.position = position;
    });
}
fn number_property(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str) -> Option<f64> {
    let k = v8::String::new(s, n)?;
    o.get(s, k.into())?.number_value(s)
}
