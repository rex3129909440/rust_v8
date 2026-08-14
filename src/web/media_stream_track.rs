use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaStreamTrackStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TrackRecord>,
    next_id: u64,
}

#[derive(Clone)]
struct TrackRecord {
    kind: String,
    id: String,
    label: String,
    enabled: bool,
    muted: bool,
    ready_state: String,
    onmute: Option<v8::Global<v8::Value>>,
    onunmute: Option<v8::Global<v8::Value>>,
    onended: Option<v8::Global<v8::Value>>,
    stats: Option<v8::Global<v8::Object>>,
    content_hint: String,
    oncapturehandlechange: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaStreamTrackStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaStreamTrack", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaStreamTrackStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "MediaStreamTrack",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "kind", get_kind)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "label", get_label)?;
    crate::webidl::define_accessor(scope, prototype, "enabled", get_enabled, set_enabled)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "muted", get_muted)?;
    crate::webidl::define_accessor(scope, prototype, "onmute", get_onmute, set_onmute)?;
    crate::webidl::define_accessor(scope, prototype, "onunmute", get_onunmute, set_onunmute)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readyState", get_ready_state)?;
    crate::webidl::define_accessor(scope, prototype, "onended", get_onended, set_onended)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "stats", get_stats)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "contentHint",
        get_content_hint,
        set_content_hint,
    )?;
    crate::webidl::define_method(scope, prototype, "applyConstraints", 0, apply_constraints)?;
    crate::webidl::define_method(scope, prototype, "clone", 0, clone_track)?;
    crate::webidl::define_method(scope, prototype, "getCapabilities", 0, get_capabilities)?;
    crate::webidl::define_method(scope, prototype, "getConstraints", 0, get_constraints)?;
    crate::webidl::define_method(scope, prototype, "getSettings", 0, get_settings)?;
    crate::webidl::define_method(scope, prototype, "stop", 0, stop)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncapturehandlechange",
        get_oncapturehandlechange,
        set_oncapturehandlechange,
    )?;
    crate::webidl::define_method(scope, prototype, "getCaptureHandle", 0, get_capture_handle)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaStreamTrackStore>()
        .ok_or_else(|| "MediaStreamTrack state was not prepared".to_owned())?
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
        "Failed to construct 'MediaStreamTrack': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: &str,
    label: Option<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaStreamTrack".to_owned());
    }
    attach(scope, object, kind, label)?;
    Ok(object)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    kind: &str,
    label: Option<String>,
) -> Result<(), String> {
    if kind != "audio" && kind != "video" {
        return Err("Invalid track kind".to_owned());
    }
    super::event_target::attach(scope, object);
    let next_id = {
        let store = scope
            .get_slot_mut::<MediaStreamTrackStore>()
            .ok_or_else(|| "MediaStreamTrack state was not prepared".to_owned())?;
        store.next_id += 1;
        store.next_id
    };
    let id = format!("{:08x}-0000-4000-8000-{:012x}", next_id, next_id);
    let stats = if kind == "audio" {
        let stats = super::media_stream_track_audio_stats::create(scope)?;
        Some(v8::Global::new(scope, stats))
    } else {
        None
    };
    let label = label.unwrap_or_else(|| id.clone());
    scope
        .get_slot_mut::<MediaStreamTrackStore>()
        .ok_or_else(|| "MediaStreamTrack state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TrackRecord {
                kind: kind.to_owned(),
                id,
                label,
                enabled: true,
                muted: false,
                ready_state: "live".to_owned(),
                onmute: None,
                onunmute: None,
                onended: None,
                stats,
                content_hint: String::new(),
                oncapturehandlechange: None,
            },
        );
    Ok(())
}

pub(crate) fn is_track(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    super::structured_clone::inherits_platform_interface(scope, object, "MediaStreamTrack")
}

pub(crate) fn kind(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.kind)
}

pub(crate) fn clone_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let record =
        record(scope, object).ok_or_else(|| "MediaStreamTrack object required".to_owned())?;
    let track = create(scope, &record.kind, Some(record.label))?;
    if let Some(clone) = scope
        .get_slot_mut::<MediaStreamTrackStore>()
        .and_then(|store| store.records.get_mut(&track.get_identity_hash().get()))
    {
        clone.enabled = record.enabled;
        clone.muted = record.muted;
        clone.ready_state = record.ready_state;
        clone.content_hint = record.content_hint;
    }
    Ok(track)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<TrackRecord> {
    scope
        .get_slot::<MediaStreamTrackStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TrackRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_kind(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.kind);
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.id);
}
fn get_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.label);
}
fn get_ready_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.ready_state);
}
fn get_content_hint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.content_hint);
}

fn get_enabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.enabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_enabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<MediaStreamTrackStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.enabled = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_muted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.muted).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn normalized_handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    value.is_function().then(|| v8::Global::new(scope, value))
}

fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TrackRecord) -> Option<&v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut TrackRecord, Option<v8::Global<v8::Value>>),
) {
    let handler = normalized_handler(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<MediaStreamTrackStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        update(record, handler);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_onmute(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onmute.as_ref());
}
fn set_onmute(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onmute = v);
}
fn get_onunmute(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onunmute.as_ref());
}
fn set_onunmute(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onunmute = v);
}
fn get_onended(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onended.as_ref());
}
fn set_onended(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onended = v);
}
fn get_oncapturehandlechange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.oncapturehandlechange.as_ref());
}
fn set_oncapturehandlechange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.oncapturehandlechange = v);
}

fn get_stats(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(stats) = record.stats {
        result.set(v8::Local::new(scope, &stats).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_content_hint(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<MediaStreamTrackStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let accepted = if record.kind == "audio" {
        value.is_empty() || value == "speech" || value == "speech-recognition" || value == "music"
    } else {
        value.is_empty() || value == "motion" || value == "detail" || value == "text"
    };
    if accepted {
        record.content_hint = value;
    }
}

fn apply_constraints(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "MediaStreamTrack",
            "applyConstraints",
            result,
        );
        return;
    };
    let constraints = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let empty = constraints
        .and_then(|object| {
            object.get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
        })
        .is_none_or(|names| names.length() == 0);
    if empty {
        let undefined = v8::undefined(scope);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
            result.set(promise.into());
        }
        return;
    }
    let constraint = if record.kind == "video" { "width" } else { "" };
    match super::overconstrained_error::create(
        scope,
        constraint.to_owned(),
        "Constraints cannot be satisfied".to_owned(),
    ) {
        Ok(error) => {
            if let Ok(promise) = super::writable_stream::rejected_promise(scope, error.into()) {
                result.set(promise.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn clone_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match clone_object(scope, arguments.this()) {
        Ok(track) => result.set(track.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_capabilities(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    if record.kind == "audio" {
        define_array(scope, object, "autoGainControl", &[]);
        define_array(scope, object, "deviceId", &[]);
        define_array(scope, object, "echoCancellation", &[]);
        define_array(scope, object, "noiseSuppression", &[]);
        define_array(scope, object, "voiceIsolation", &[]);
    } else {
        define_array(scope, object, "deviceId", &[]);
        define_array(scope, object, "facingMode", &[]);
        define_array(scope, object, "resizeMode", &["none", "crop-and-scale"]);
    }
    result.set(object.into());
}

fn get_constraints(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Object::new(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_settings(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    define_string(scope, object, "deviceId", &record.id);
    if record.kind == "audio" {
        define_number(scope, object, "sampleSize", 16.0);
    } else {
        define_string(scope, object, "resizeMode", "none");
    }
    result.set(object.into());
}

fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<MediaStreamTrackStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.ready_state = "ended".to_owned();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_capture_handle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn define_array(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    values: &[&str],
) {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), array.into());
    }
}

fn define_string(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let (Some(key), Some(value)) = (v8::String::new(scope, name), v8::String::new(scope, value))
    {
        let _ = object.set(scope, key.into(), value.into());
    }
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
