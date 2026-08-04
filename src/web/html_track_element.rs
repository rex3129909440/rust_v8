use std::collections::HashMap;

pub(crate) const NONE: i32 = 0;
pub(crate) const LOADING: i32 = 1;
pub(crate) const LOADED: i32 = 2;
pub(crate) const ERROR: i32 = 3;

#[derive(Default)]
pub(crate) struct HtmlTrackElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TrackRecord>,
}

#[derive(Clone)]
pub(crate) struct TrackRecord {
    pub(crate) kind: String,
    pub(crate) src: String,
    pub(crate) srclang: String,
    pub(crate) label: String,
    pub(crate) default_enabled: bool,
    pub(crate) ready_state: i32,
    pub(crate) track: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlTrackElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLTrackElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlTrackElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLTrackElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_track_element_kind_property::define(scope, prototype)?;
    super::html_track_element_src_property::define(scope, prototype)?;
    super::html_track_element_srclang_property::define(scope, prototype)?;
    super::html_track_element_label_property::define(scope, prototype)?;
    super::html_track_element_default_property::define(scope, prototype)?;
    super::html_track_element_ready_state_property::define(scope, prototype)?;
    super::html_track_element_track_property::define(scope, prototype)?;
    define_constant(scope, prototype, "NONE", NONE)?;
    define_constant(scope, prototype, "LOADING", LOADING)?;
    define_constant(scope, prototype, "LOADED", LOADED)?;
    define_constant(scope, prototype, "ERROR", ERROR)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constant(scope, constructor.into(), "NONE", NONE)?;
    define_constant(scope, constructor.into(), "LOADING", LOADING)?;
    define_constant(scope, constructor.into(), "LOADED", LOADED)?;
    define_constant(scope, constructor.into(), "ERROR", ERROR)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlTrackElementStore>()
        .ok_or_else(|| "HTMLTrackElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLTrackElement".to_owned());
    }
    super::html_element::attach(scope, object, "TRACK");
    let track = super::text_track::create(
        scope,
        "subtitles".to_owned(),
        String::new(),
        String::new(),
        String::new(),
    )?;
    let track = v8::Global::new(scope, track);
    scope
        .get_slot_mut::<HtmlTrackElementStore>()
        .ok_or_else(|| "HTMLTrackElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TrackRecord {
                kind: "subtitles".to_owned(),
                src: String::new(),
                srclang: String::new(),
                label: String::new(),
                default_enabled: false,
                ready_state: NONE,
                track,
            },
        );
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn define_constant(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    name: &str,
    value: i32,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    match target.define_own_property(
        scope,
        key.into(),
        v8::Integer::new(scope, value).into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
    ) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot define HTMLTrackElement.{name}")),
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TrackRecord> {
    scope
        .get_slot::<HtmlTrackElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn string_getter(
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

pub(crate) fn string_setter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut TrackRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlTrackElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        update(record, value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_kind(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_getter(s, a, r, |x| &x.kind);
}
pub(crate) fn set_kind(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    string_setter(s, a, |x, v| x.kind = v);
}
pub(crate) fn get_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_getter(s, a, r, |x| &x.src);
}
pub(crate) fn set_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    string_setter(s, a, |x, v| {
        x.src = v;
        x.ready_state = if x.src.is_empty() { NONE } else { LOADING };
    });
}
pub(crate) fn get_srclang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_getter(s, a, r, |x| &x.srclang);
}
pub(crate) fn set_srclang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    string_setter(s, a, |x, v| x.srclang = v);
}
pub(crate) fn get_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_getter(s, a, r, |x| &x.label);
}
pub(crate) fn set_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    string_setter(s, a, |x, v| x.label = v);
}

pub(crate) fn get_default(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.default_enabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_default(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTrackElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.default_enabled = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_ready_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.ready_state).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.track).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
