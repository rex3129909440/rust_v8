use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextTrackStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TextTrackRecord>,
}

#[derive(Clone)]
struct TextTrackRecord {
    kind: String,
    label: String,
    language: String,
    id: String,
    mode: String,
    cues: v8::Global<v8::Object>,
    active_cues: v8::Global<v8::Object>,
    on_cue_change: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextTrackStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextTrack", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextTrackStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TextTrack",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "kind", get_kind)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "label", get_label)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "language", get_language)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_accessor(scope, prototype, "mode", get_mode, set_mode)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "cues", get_cues)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "activeCues", get_active_cues)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncuechange",
        get_on_cue_change,
        set_on_cue_change,
    )?;
    crate::webidl::define_method(scope, prototype, "addCue", 1, add_cue)?;
    crate::webidl::define_method(scope, prototype, "removeCue", 1, remove_cue)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TextTrackStore>()
        .ok_or_else(|| "TextTrack state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: String,
    label: String,
    language: String,
    id: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let track = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, track, prototype.into()) != Some(true) {
        return Err("cannot create TextTrack".to_owned());
    }
    let cues = super::text_track_cue_list::create(scope)?;
    let active_cues = super::text_track_cue_list::create(scope)?;
    let record = TextTrackRecord {
        kind,
        label,
        language,
        id,
        mode: "disabled".to_owned(),
        cues: v8::Global::new(scope, cues),
        active_cues: v8::Global::new(scope, active_cues),
        on_cue_change: None,
    };
    super::event_target::attach(scope, track);
    scope
        .get_slot_mut::<TextTrackStore>()
        .ok_or_else(|| "TextTrack state was not prepared".to_owned())?
        .records
        .insert(track.get_identity_hash().get(), record);
    Ok(track)
}

pub(crate) fn id(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.id)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TextTrack': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TextTrackRecord> {
    scope
        .get_slot::<TextTrackStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut TextTrackRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<TextTrackStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TextTrackRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

fn get_kind(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.kind);
}
fn get_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.label);
}
fn get_language(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.language);
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.id);
}
fn get_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.mode);
}

fn set_mode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mode = crate::webidl::value_to_string(scope, arguments.get(0));
    if !matches!(mode.as_str(), "disabled" | "hidden" | "showing") {
        crate::webidl::throw_type_error(scope, "Invalid TextTrack mode");
        return;
    }
    update(scope, arguments.this(), |record| record.mode = mode);
}

fn return_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    active: bool,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = if active {
        &record.active_cues
    } else {
        &record.cues
    };
    result.set(v8::Local::new(scope, list).into());
}

fn get_cues(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_list(s, a, r, false);
}
fn get_active_cues(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_list(s, a, r, true);
}

fn get_on_cue_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.on_cue_change {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_on_cue_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let handler = if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    };
    update(scope, arguments.this(), |record| {
        record.on_cue_change = handler
    });
}

fn add_cue(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(cue) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "addCue requires a TextTrackCue");
        return;
    };
    if super::text_track_cue::id(scope, cue).is_none() {
        crate::webidl::throw_type_error(scope, "addCue requires a TextTrackCue");
        return;
    }
    let cues = v8::Local::new(scope, &record.cues);
    super::text_track_cue_list::add(scope, cues, cue);
    super::text_track_cue::set_track(scope, cue, Some(arguments.this()));
    if record.mode != "disabled" {
        let active = v8::Local::new(scope, &record.active_cues);
        super::text_track_cue_list::add(scope, active, cue);
    }
}

fn remove_cue(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(cue) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "removeCue requires a TextTrackCue");
        return;
    };
    let cues = v8::Local::new(scope, &record.cues);
    if !super::text_track_cue_list::remove(scope, cues, cue) {
        crate::webidl::throw_type_error(scope, "The cue is not part of this track");
        return;
    }
    let active = v8::Local::new(scope, &record.active_cues);
    super::text_track_cue_list::remove(scope, active, cue);
    super::text_track_cue::set_track(scope, cue, None);
}
