use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct StyleSheetStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
struct Record {
    sheet_type: String,
    href: Option<String>,
    owner_node: Option<v8::Global<v8::Object>>,
    parent: Option<v8::Global<v8::Object>>,
    title: Option<String>,
    media: v8::Global<v8::Value>,
    disabled: bool,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StyleSheetStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "StyleSheet", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StyleSheetStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "StyleSheet",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, p, "href", get_href)?;
    crate::webidl::define_readonly_accessor(scope, p, "ownerNode", get_owner_node)?;
    crate::webidl::define_readonly_accessor(scope, p, "parentStyleSheet", get_parent_style_sheet)?;
    crate::webidl::define_readonly_accessor(scope, p, "title", get_title)?;
    crate::webidl::define_accessor(scope, p, "media", get_media, set_media)?;
    crate::webidl::define_accessor(scope, p, "disabled", get_disabled, set_disabled)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StyleSheetStore>()
        .ok_or_else(|| "StyleSheet state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    href: Option<String>,
    title: Option<String>,
    media: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create StyleSheet".to_owned());
    }
    attach(scope, o, href, title, media, false);
    Ok(o)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    href: Option<String>,
    title: Option<String>,
    media: v8::Local<'_, v8::Value>,
    disabled: bool,
) {
    let record = Record {
        sheet_type: "text/css".to_owned(),
        href,
        owner_node: None,
        parent: None,
        title,
        media: v8::Global::new(scope, media),
        disabled,
    };
    scope
        .get_slot_mut::<StyleSheetStore>()
        .expect("StyleSheet state")
        .records
        .insert(object.get_identity_hash().get(), record);
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'StyleSheet': Illegal constructor",
    );
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<StyleSheetStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(v) = scope
        .get_slot_mut::<StyleSheetStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn set_owner_node(
    scope: &mut v8::PinScope<'_, '_>,
    sheet: v8::Local<'_, v8::Object>,
    owner: v8::Local<'_, v8::Object>,
) -> bool {
    let owner = v8::Global::new(scope, owner);
    let Some(record) = scope
        .get_slot_mut::<StyleSheetStore>()
        .and_then(|store| store.records.get_mut(&sheet.get_identity_hash().get()))
    else {
        return false;
    };
    record.owner_node = Some(owner);
    true
}

pub(crate) fn is_disabled(scope: &v8::PinScope<'_, '_>, sheet: v8::Local<'_, v8::Object>) -> bool {
    record(scope, sheet).is_some_and(|record| record.disabled)
}

pub(crate) fn set_disabled_value(
    scope: &mut v8::PinScope<'_, '_>,
    sheet: v8::Local<'_, v8::Object>,
    disabled: bool,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<StyleSheetStore>()
        .and_then(|store| store.records.get_mut(&sheet.get_identity_hash().get()))
    else {
        return false;
    };
    record.disabled = disabled;
    true
}

pub(crate) fn set_media_text(
    scope: &mut v8::PinScope<'_, '_>,
    sheet: v8::Local<'_, v8::Object>,
    media_text: &str,
) -> bool {
    let Some(record) = record(scope, sheet) else {
        return false;
    };
    let media = v8::Local::new(scope, &record.media);
    let Ok(media) = v8::Local::<v8::Object>::try_from(media) else {
        return false;
    };
    super::media_list::set_text(scope, media, media_text)
}
fn string_result(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<&str>,
    r: &mut v8::ReturnValue<'_>,
) {
    if let Some(v) = value.and_then(|v| v8::String::new(scope, v)) {
        r.set(v.into())
    } else {
        r.set(v8::null(scope).into())
    }
}
fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, &v.sheet_type) {
        r.set(s.into())
    }
}
fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    string_result(scope, v.href.as_deref(), &mut r)
}
fn get_title(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    string_result(scope, v.title.as_deref(), &mut r)
}
fn get_owner_node(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(o) = v.owner_node {
        r.set(v8::Local::new(scope, &o).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
fn get_parent_style_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(o) = v.parent {
        r.set(v8::Local::new(scope, &o).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
fn get_media(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.media))
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_media(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = v8::Global::new(scope, a.get(0));
    update(scope, a.this(), |r| r.media = v)
}
fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.disabled).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(scope);
    update(scope, a.this(), |r| r.disabled = v)
}
