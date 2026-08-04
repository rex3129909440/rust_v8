use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlStyleElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, StyleRecord>,
}

#[derive(Clone)]
pub(crate) struct StyleRecord {
    pub(crate) disabled: bool,
    pub(crate) media: String,
    pub(crate) style_type: String,
    pub(crate) sheet: Option<v8::Global<v8::Object>>,
    pub(crate) blocking: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlStyleElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLStyleElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlStyleElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLStyleElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_style_element_disabled_property::define(scope, prototype)?;
    super::html_style_element_media_property::define(scope, prototype)?;
    super::html_style_element_type_property::define(scope, prototype)?;
    super::html_style_element_sheet_property::define(scope, prototype)?;
    super::html_style_element_blocking_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlStyleElementStore>()
        .ok_or_else(|| "HTMLStyleElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLStyleElement".to_owned());
    }
    super::html_element::attach(scope, object, "STYLE");
    let blocking = super::dom_token_list::create(scope, "")?;
    let blocking = v8::Global::new(scope, blocking);
    scope
        .get_slot_mut::<HtmlStyleElementStore>()
        .ok_or_else(|| "HTMLStyleElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            StyleRecord {
                disabled: false,
                media: String::new(),
                style_type: String::new(),
                sheet: None,
                blocking,
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
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<StyleRecord> {
    scope
        .get_slot::<HtmlStyleElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        let disabled = record
            .sheet
            .as_ref()
            .map(|sheet| {
                let sheet = v8::Local::new(scope, sheet);
                super::style_sheet::is_disabled(scope, sheet)
            })
            .unwrap_or(record.disabled);
        r.set(v8::Boolean::new(scope, disabled).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(scope);
    if record(scope, a.this()).is_some() {
        if let Some(record) = scope
            .get_slot_mut::<HtmlStyleElementStore>()
            .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
        {
            record.disabled = v;
        }
        if let Some(sheet) = sheet(scope, a.this()) {
            super::style_sheet::set_disabled_value(scope, sheet, v);
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&StyleRecord) -> &str,
) {
    if let Some(x) = record(scope, a.this()) {
        if let Some(v) = v8::String::new(scope, select(&x)) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut StyleRecord, String),
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(x) = scope
        .get_slot_mut::<HtmlStyleElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        update(x, v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::reflected_string(s, a.this(), "media").unwrap_or_default();
    let mut r = r;
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into());
    }
}
pub(crate) fn set_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "media", value.clone());
    if let Some(record) = s
        .get_slot_mut::<HtmlStyleElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.media = value.clone();
    }
    if let Some(sheet) = sheet(s, a.this()) {
        super::style_sheet::set_media_text(s, sheet, &value);
    }
}
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::reflected_string(s, a.this(), "type").unwrap_or_default();
    let mut r = r;
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into());
    }
}
pub(crate) fn set_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "type", value);
    refresh_sheet(s, a.this(), true);
}
pub(crate) fn get_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        if let Some(sheet) = x.sheet {
            r.set(v8::Local::new(scope, &sheet).into())
        } else {
            r.set(v8::null(scope).into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_blocking(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &x.blocking).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn set_blocking(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let blocking = v8::Local::new(scope, &record.blocking);
        super::dom_token_list::set_string_value(scope, blocking, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn sheet<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    record(scope, element)?
        .sheet
        .map(|sheet| v8::Local::new(scope, &sheet))
}

fn refresh_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    replace_existing: bool,
) {
    if !super::node::is_connected(scope, element) {
        if let Some(record) = scope
            .get_slot_mut::<HtmlStyleElementStore>()
            .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
        {
            record.sheet = None;
        }
        return;
    }
    if !replace_existing && record(scope, element).is_some_and(|record| record.sheet.is_some()) {
        return;
    }
    let media = super::element::attribute_value(scope, element, "media").unwrap_or_default();
    let disabled = record(scope, element).is_some_and(|record| record.disabled);
    let text = super::node::text_content(scope, element);
    let sheet =
        super::css_style_sheet::create_for_owner(scope, element, None, &media, disabled, &text);
    if let Ok(sheet) = sheet {
        let sheet = v8::Global::new(scope, sheet);
        if let Some(record) = scope
            .get_slot_mut::<HtmlStyleElementStore>()
            .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
        {
            record.disabled = disabled;
            record.media = media;
            record.sheet = Some(sheet);
        }
    }
}

pub(crate) fn notify_connected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if record(scope, root).is_some() {
        refresh_sheet(scope, root, false);
    }
    for child in super::node::children(scope, root) {
        notify_connected_tree(scope, child);
    }
}

pub(crate) fn notify_disconnected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlStyleElementStore>()
        .and_then(|store| store.records.get_mut(&root.get_identity_hash().get()))
    {
        record.sheet = None;
    }
    for child in super::node::children(scope, root) {
        notify_disconnected_tree(scope, child);
    }
}

pub(crate) fn notify_tree_mutation(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
) {
    if record(scope, node).is_some() {
        refresh_sheet(scope, node, true);
        return;
    }
    if let Some(parent) = super::node::parent(scope, node) {
        notify_tree_mutation(scope, parent);
    }
}
