use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct BodyRecord {
    pub(crate) text: String,
    pub(crate) link: String,
    pub(crate) v_link: String,
    pub(crate) a_link: String,
    pub(crate) bg_color: String,
    pub(crate) background: String,
    pub(crate) handlers: HashMap<&'static str, v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct HtmlBodyElementStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, BodyRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlBodyElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLBodyElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<HtmlBodyElementStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLBodyElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_body_element_text_property::define(scope, prototype)?;
    super::html_body_element_link_property::define(scope, prototype)?;
    super::html_body_element_v_link_property::define(scope, prototype)?;
    super::html_body_element_a_link_property::define(scope, prototype)?;
    super::html_body_element_bg_color_property::define(scope, prototype)?;
    super::html_body_element_background_property::define(scope, prototype)?;
    super::html_body_element_onblur_property::define(scope, prototype)?;
    super::html_body_element_onerror_property::define(scope, prototype)?;
    super::html_body_element_onfocus_property::define(scope, prototype)?;
    super::html_body_element_onload_property::define(scope, prototype)?;
    super::html_body_element_onresize_property::define(scope, prototype)?;
    super::html_body_element_onscroll_property::define(scope, prototype)?;
    super::html_body_element_onafterprint_property::define(scope, prototype)?;
    super::html_body_element_onbeforeprint_property::define(scope, prototype)?;
    super::html_body_element_onbeforeunload_property::define(scope, prototype)?;
    super::html_body_element_onhashchange_property::define(scope, prototype)?;
    super::html_body_element_onlanguagechange_property::define(scope, prototype)?;
    super::html_body_element_onmessage_property::define(scope, prototype)?;
    super::html_body_element_onmessageerror_property::define(scope, prototype)?;
    super::html_body_element_onoffline_property::define(scope, prototype)?;
    super::html_body_element_ononline_property::define(scope, prototype)?;
    super::html_body_element_onpagehide_property::define(scope, prototype)?;
    super::html_body_element_onpageshow_property::define(scope, prototype)?;
    super::html_body_element_onpopstate_property::define(scope, prototype)?;
    super::html_body_element_onrejectionhandled_property::define(scope, prototype)?;
    super::html_body_element_onstorage_property::define(scope, prototype)?;
    super::html_body_element_onunhandledrejection_property::define(scope, prototype)?;
    super::html_body_element_onunload_property::define(scope, prototype)?;
    super::html_body_element_ongamepadconnected_property::define(scope, prototype)?;
    super::html_body_element_ongamepaddisconnected_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlBodyElementStore>()
        .ok_or_else(|| "HTMLBodyElement state was not prepared".to_owned())?
        .constructors
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
        return Err("cannot create HTMLBodyElement".to_owned());
    }
    super::html_element::attach(scope, object, "BODY");
    scope
        .get_slot_mut::<HtmlBodyElementStore>()
        .ok_or_else(|| "HTMLBodyElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), BodyRecord::default());
    Ok(object)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BodyRecord> {
    scope
        .get_slot::<HtmlBodyElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&BodyRecord) -> &str,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            r.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut BodyRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlBodyElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        change(record, value)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.text)
}
pub(crate) fn set_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.text = v)
}
pub(crate) fn get_link(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.link)
}
pub(crate) fn set_link(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.link = v)
}
pub(crate) fn get_v_link(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.v_link)
}
pub(crate) fn set_v_link(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.v_link = v)
}
pub(crate) fn get_a_link(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.a_link)
}
pub(crate) fn set_a_link(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.a_link = v)
}
pub(crate) fn get_bg_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.bg_color)
}
pub(crate) fn set_bg_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.bg_color = v)
}
pub(crate) fn get_background(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.background)
}
pub(crate) fn set_background(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.background = v)
}
pub(crate) fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    name: &'static str,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(value) = record.handlers.get(name) {
            r.set(v8::Local::new(scope, value))
        } else {
            r.set(v8::null(scope).into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    name: &'static str,
) {
    let value = a.get(0);
    let stored = if value.is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    };
    if let Some(record) = scope
        .get_slot_mut::<HtmlBodyElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        if let Some(stored) = stored {
            record.handlers.insert(name, stored);
        } else {
            record.handlers.remove(name);
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_on_blur(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onblur")
}
pub(crate) fn set_on_blur(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onblur")
}
pub(crate) fn get_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onerror")
}
pub(crate) fn set_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onerror")
}
pub(crate) fn get_on_focus(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onfocus")
}
pub(crate) fn set_on_focus(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onfocus")
}
pub(crate) fn get_on_load(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onload")
}
pub(crate) fn set_on_load(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onload")
}
pub(crate) fn get_on_resize(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onresize")
}
pub(crate) fn set_on_resize(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onresize")
}
pub(crate) fn get_on_scroll(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onscroll")
}
pub(crate) fn set_on_scroll(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onscroll")
}
pub(crate) fn get_on_after_print(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onafterprint")
}
pub(crate) fn set_on_after_print(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onafterprint")
}
pub(crate) fn get_on_before_print(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onbeforeprint")
}
pub(crate) fn set_on_before_print(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onbeforeprint")
}
pub(crate) fn get_on_before_unload(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onbeforeunload")
}
pub(crate) fn set_on_before_unload(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onbeforeunload")
}
pub(crate) fn get_on_hash_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onhashchange")
}
pub(crate) fn set_on_hash_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onhashchange")
}
pub(crate) fn get_on_language_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onlanguagechange")
}
pub(crate) fn set_on_language_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onlanguagechange")
}
pub(crate) fn get_on_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onmessage")
}
pub(crate) fn set_on_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onmessage")
}
pub(crate) fn get_on_message_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onmessageerror")
}
pub(crate) fn set_on_message_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onmessageerror")
}
pub(crate) fn get_on_offline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onoffline")
}
pub(crate) fn set_on_offline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onoffline")
}
pub(crate) fn get_on_online(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "ononline")
}
pub(crate) fn set_on_online(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "ononline")
}
pub(crate) fn get_on_page_hide(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onpagehide")
}
pub(crate) fn set_on_page_hide(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onpagehide")
}
pub(crate) fn get_on_page_show(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onpageshow")
}
pub(crate) fn set_on_page_show(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onpageshow")
}
pub(crate) fn get_on_pop_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onpopstate")
}
pub(crate) fn set_on_pop_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onpopstate")
}
pub(crate) fn get_on_rejection_handled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onrejectionhandled")
}
pub(crate) fn set_on_rejection_handled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onrejectionhandled")
}
pub(crate) fn get_on_storage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onstorage")
}
pub(crate) fn set_on_storage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onstorage")
}
pub(crate) fn get_on_unhandled_rejection(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onunhandledrejection")
}
pub(crate) fn set_on_unhandled_rejection(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onunhandledrejection")
}
pub(crate) fn get_on_unload(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "onunload")
}
pub(crate) fn set_on_unload(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "onunload")
}
pub(crate) fn get_on_gamepad_connected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "ongamepadconnected")
}
pub(crate) fn set_on_gamepad_connected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "ongamepadconnected")
}
pub(crate) fn get_on_gamepad_disconnected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, "ongamepaddisconnected")
}
pub(crate) fn set_on_gamepad_disconnected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, "ongamepaddisconnected")
}
