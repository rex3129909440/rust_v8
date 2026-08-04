use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub(crate) struct LinkRecord {
    pub(crate) disabled: bool,
    pub(crate) href: String,
    pub(crate) cross_origin: Option<String>,
    pub(crate) rel_list: v8::Global<v8::Object>,
    pub(crate) media: String,
    pub(crate) href_lang: String,
    pub(crate) link_type: String,
    pub(crate) destination: String,
    pub(crate) referrer_policy: String,
    pub(crate) sizes: v8::Global<v8::Object>,
    pub(crate) fetch_priority: String,
    pub(crate) image_srcset: String,
    pub(crate) image_sizes: String,
    pub(crate) charset: String,
    pub(crate) rev: String,
    pub(crate) target: String,
    pub(crate) integrity: String,
    pub(crate) blocking: v8::Global<v8::Object>,
    pub(crate) sheet: Option<v8::Global<v8::Object>>,
}

struct PendingLinkEvent {
    context: v8::Global<v8::Context>,
    element: v8::Global<v8::Object>,
    event_type: &'static str,
}

#[derive(Default)]
pub(crate) struct HtmlLinkElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, LinkRecord>,
    pending: VecDeque<PendingLinkEvent>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlLinkElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLLinkElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlLinkElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLLinkElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_link_element_disabled_property::define(scope, prototype)?;
    super::html_link_element_href_property::define(scope, prototype)?;
    super::html_link_element_cross_origin_property::define(scope, prototype)?;
    super::html_link_element_rel_property::define(scope, prototype)?;
    super::html_link_element_rel_list_property::define(scope, prototype)?;
    super::html_link_element_media_property::define(scope, prototype)?;
    super::html_link_element_hreflang_property::define(scope, prototype)?;
    super::html_link_element_type_property::define(scope, prototype)?;
    super::html_link_element_as_property::define(scope, prototype)?;
    super::html_link_element_referrer_policy_property::define(scope, prototype)?;
    super::html_link_element_sizes_property::define(scope, prototype)?;
    super::html_link_element_fetch_priority_property::define(scope, prototype)?;
    super::html_link_element_image_srcset_property::define(scope, prototype)?;
    super::html_link_element_image_sizes_property::define(scope, prototype)?;
    super::html_link_element_charset_property::define(scope, prototype)?;
    super::html_link_element_rev_property::define(scope, prototype)?;
    super::html_link_element_target_property::define(scope, prototype)?;
    super::html_link_element_sheet_property::define(scope, prototype)?;
    super::html_link_element_integrity_property::define(scope, prototype)?;
    super::html_link_element_blocking_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlLinkElementStore>()
        .ok_or_else(|| "HTMLLinkElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLLinkElement".to_owned());
    }
    super::html_element::attach(scope, object, "LINK");
    let rel_list = super::dom_token_list::create(scope, "")?;
    let sizes = super::dom_token_list::create(scope, "")?;
    let blocking = super::dom_token_list::create(scope, "")?;
    let record = LinkRecord {
        disabled: false,
        href: String::new(),
        cross_origin: None,
        rel_list: v8::Global::new(scope, rel_list),
        media: String::new(),
        href_lang: String::new(),
        link_type: String::new(),
        destination: String::new(),
        referrer_policy: String::new(),
        sizes: v8::Global::new(scope, sizes),
        fetch_priority: "auto".to_owned(),
        image_srcset: String::new(),
        image_sizes: String::new(),
        charset: String::new(),
        rev: String::new(),
        target: String::new(),
        integrity: String::new(),
        blocking: v8::Global::new(scope, blocking),
        sheet: None,
    };
    scope
        .get_slot_mut::<HtmlLinkElementStore>()
        .ok_or_else(|| "HTMLLinkElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
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
) -> Option<LinkRecord> {
    scope
        .get_slot::<HtmlLinkElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut LinkRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlLinkElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&LinkRecord) -> &str,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut LinkRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    update(scope, a.this(), |record| change(record, value));
}
pub(crate) fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, record.disabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    update(scope, a.this(), |record| record.disabled = value);
}
pub(crate) fn get_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::resolved_url_attribute(s, a.this(), "href").unwrap_or_default();
    if let Some(value) = v8::String::new(s, &value) {
        let mut r = r;
        r.set(value.into());
    }
}
pub(crate) fn set_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "href", value);
    refresh_connected(s, a.this());
}
pub(crate) fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(cross_origin) = record.cross_origin {
            if let Some(value) = v8::String::new(scope, &cross_origin) {
                r.set(value.into());
            }
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if a.get(0).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(0)))
    };
    update(scope, a.this(), |record| record.cross_origin = value);
}
pub(crate) fn get_rel(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        let list = v8::Local::new(scope, &record.rel_list);
        let value = super::dom_token_list::string_value(scope, list).unwrap_or_default();
        if let Some(value) = v8::String::new(scope, &value) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_rel(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let list = v8::Local::new(scope, &record.rel_list);
        let _ = super::dom_token_list::set_string_value(scope, list, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_rel_list(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &record.rel_list).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_rel_list(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let list = v8::Local::new(scope, &record.rel_list);
        super::dom_token_list::set_string_value(scope, list, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.media);
}
pub(crate) fn set_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.media = v);
}
pub(crate) fn get_href_lang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.href_lang);
}
pub(crate) fn set_href_lang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.href_lang = v);
}
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.link_type);
}
pub(crate) fn set_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.link_type = v);
}
pub(crate) fn get_as(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.destination);
}
pub(crate) fn set_as(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.destination = v);
}
pub(crate) fn get_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.referrer_policy);
}
pub(crate) fn set_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.referrer_policy = v);
}
pub(crate) fn get_sizes(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &record.sizes).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_sizes(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let sizes = v8::Local::new(scope, &record.sizes);
        super::dom_token_list::set_string_value(scope, sizes, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_fetch_priority(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.fetch_priority);
}
pub(crate) fn set_fetch_priority(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    let value = if matches!(value.as_str(), "high" | "low" | "auto") {
        value
    } else {
        "auto".to_owned()
    };
    update(s, a.this(), |x| x.fetch_priority = value);
}
pub(crate) fn get_image_srcset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.image_srcset);
}
pub(crate) fn set_image_srcset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.image_srcset = v);
}
pub(crate) fn get_image_sizes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.image_sizes);
}
pub(crate) fn set_image_sizes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.image_sizes = v);
}
pub(crate) fn get_charset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.charset);
}
pub(crate) fn set_charset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.charset = v);
}
pub(crate) fn get_rev(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.rev);
}
pub(crate) fn set_rev(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.rev = v);
}
pub(crate) fn get_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.target);
}
pub(crate) fn set_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.target = v);
}
pub(crate) fn get_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_integrity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.integrity);
}
pub(crate) fn set_integrity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.integrity = v);
}
pub(crate) fn get_blocking(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &record.blocking).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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

fn rel_is_stylesheet(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    super::element::attribute_value(scope, element, "rel")
        .unwrap_or_default()
        .split_ascii_whitespace()
        .any(|token| token.eq_ignore_ascii_case("stylesheet"))
}

fn queue_event(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    event_type: &'static str,
) {
    let event = PendingLinkEvent {
        context: v8::Global::new(scope, scope.get_current_context()),
        element: v8::Global::new(scope, element),
        event_type,
    };
    if let Some(store) = scope.get_slot_mut::<HtmlLinkElementStore>() {
        store.pending.push_back(event);
    }
}

pub(crate) fn refresh_connected(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) {
    if record(scope, element).is_none() {
        return;
    }
    if !super::node::is_connected(scope, element) || !rel_is_stylesheet(scope, element) {
        if let Some(record) = scope
            .get_slot_mut::<HtmlLinkElementStore>()
            .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
        {
            record.sheet = None;
        }
        super::document_style_sheets_property::refresh_for_node(scope, element);
        return;
    }
    let href = super::element::resolved_url_attribute(scope, element, "href").unwrap_or_default();
    if href.is_empty() {
        return;
    }
    let loaded = super::worker_script_source::load_with_initiator(scope, &href, None, "link");
    let (sheet, event_type) = match loaded {
        Ok(resource) => {
            let media =
                super::element::attribute_value(scope, element, "media").unwrap_or_default();
            let disabled = record(scope, element).is_some_and(|record| record.disabled);
            (
                super::css_style_sheet::create_for_owner(
                    scope,
                    element,
                    Some(resource.url),
                    &media,
                    disabled,
                    &resource.source,
                )
                .ok(),
                "load",
            )
        }
        Err(_) => (None, "error"),
    };
    let sheet = sheet.map(|sheet| v8::Global::new(scope, sheet));
    if let Some(record) = scope
        .get_slot_mut::<HtmlLinkElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    {
        record.sheet = sheet;
    }
    super::document_style_sheets_property::refresh_for_node(scope, element);
    queue_event(scope, element, event_type);
}

pub(crate) fn notify_connected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if record(scope, root).is_some() {
        refresh_connected(scope, root);
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
        .get_slot_mut::<HtmlLinkElementStore>()
        .and_then(|store| store.records.get_mut(&root.get_identity_hash().get()))
    {
        record.sheet = None;
    }
    for child in super::node::children(scope, root) {
        notify_disconnected_tree(scope, child);
    }
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let pending = scope
        .get_slot_mut::<HtmlLinkElementStore>()
        .and_then(|store| store.pending.pop_front());
    let Some(pending) = pending else {
        return false;
    };
    let context = v8::Local::new(scope, &pending.context);
    let event_scope = &mut v8::ContextScope::new(scope, context);
    let element = v8::Local::new(event_scope, &pending.element);
    if let Ok(event) = super::event::create(event_scope, pending.event_type) {
        super::event_target::dispatch(event_scope, element, event);
    }
    true
}
