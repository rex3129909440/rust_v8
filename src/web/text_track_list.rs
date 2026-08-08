use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextTrackListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TextTrackListRecord>,
}

#[derive(Clone)]
struct TextTrackListRecord {
    tracks: Vec<v8::Global<v8::Object>>,
    onchange: Option<v8::Global<v8::Value>>,
    onaddtrack: Option<v8::Global<v8::Value>>,
    onremovetrack: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextTrackListStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextTrackList", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TextTrackListStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "TextTrackList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "length", get_length)?;
    crate::webidl::define_accessor(scope, p, "onchange", get_onchange, set_onchange)?;
    crate::webidl::define_accessor(scope, p, "onaddtrack", get_onaddtrack, set_onaddtrack)?;
    crate::webidl::define_accessor(
        scope,
        p,
        "onremovetrack",
        get_onremovetrack,
        set_onremovetrack,
    )?;
    crate::webidl::define_method(scope, p, "getTrackById", 1, get_track_by_id)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_indexed_iterator(scope, p)?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, event_target)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<TextTrackListStore>()
        .ok_or_else(|| "TextTrackList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create TextTrackList".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<TextTrackListStore>()
        .ok_or_else(|| "TextTrackList state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TextTrackListRecord {
                tracks: Vec::new(),
                onchange: None,
                onaddtrack: None,
                onremovetrack: None,
            },
        );
    Ok(object)
}
pub(crate) fn append(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    track: v8::Local<'_, v8::Object>,
) -> bool {
    let index = scope
        .get_slot::<TextTrackListStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .map(|record| record.tracks.len());
    let Some(index) = index else {
        return false;
    };
    let Some(key) = v8::String::new(scope, &index.to_string()) else {
        return false;
    };
    if object.define_own_property(
        scope,
        key.into(),
        track.into(),
        v8::PropertyAttribute::READ_ONLY,
    ) != Some(true)
    {
        return false;
    }
    let track = v8::Global::new(scope, track);
    if let Some(record) = scope
        .get_slot_mut::<TextTrackListStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.tracks.push(track);
        true
    } else {
        false
    }
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TextTrackList': Illegal constructor",
    );
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TextTrackListRecord> {
    scope
        .get_slot::<TextTrackListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut TextTrackListRecord),
) {
    if let Some(r) = scope
        .get_slot_mut::<TextTrackListStore>()
        .and_then(|s| s.records.get_mut(&object.get_identity_hash().get()))
    {
        change(r)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(r) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, r.tracks.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_track_by_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(r) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let wanted = crate::webidl::value_to_string(scope, arguments.get(0));
    for value in r.tracks {
        let track = v8::Local::new(scope, &value);
        if super::text_track::id(scope, track).is_some_and(|id| id == wanted) {
            result.set(track.into());
            return;
        }
    }
    result.set(v8::null(scope).into());
}
fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TextTrackListRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(r) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = select(&r) {
        result.set(v8::Local::new(scope, &v))
    } else {
        result.set(v8::null(scope).into())
    }
}
fn handler(
    scope: &v8::PinScope<'_, '_>,
    v: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    if v.is_null() || v.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, v))
    }
}
fn get_onchange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |v| v.onchange.clone())
}
fn get_onaddtrack(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |v| v.onaddtrack.clone())
}
fn get_onremovetrack(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |v| v.onremovetrack.clone())
}
fn set_onchange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = handler(s, a.get(0));
    update(s, a.this(), |r| r.onchange = v)
}
fn set_onaddtrack(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = handler(s, a.get(0));
    update(s, a.this(), |r| r.onaddtrack = v)
}
fn set_onremovetrack(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = handler(s, a.get(0));
    update(s, a.this(), |r| r.onremovetrack = v)
}
