use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct HtmlElementStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, HtmlElementRecord>,
    pub(crate) click_in_progress: HashSet<i32>,
}

pub(crate) fn begin_click_activation(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot_mut::<HtmlElementStore>()
        .is_some_and(|store| {
            store
                .click_in_progress
                .insert(object.get_identity_hash().get())
        })
}

pub(crate) fn finish_click_activation(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) {
    if let Some(store) = scope.get_slot_mut::<HtmlElementStore>() {
        store
            .click_in_progress
            .remove(&object.get_identity_hash().get());
    }
}

#[derive(Clone)]
pub(crate) struct HtmlElementRecord {
    pub(crate) strings: HashMap<String, String>,
    pub(crate) booleans: HashMap<String, bool>,
    pub(crate) handlers: HashMap<String, v8::Global<v8::Value>>,
    pub(crate) edit_context: Option<v8::Global<v8::Object>>,
    pub(crate) internals: Option<v8::Global<v8::Object>>,
    pub(crate) dataset: v8::Global<v8::Object>,
    pub(crate) style: v8::Global<v8::Object>,
    pub(crate) attribute_style_map: v8::Global<v8::Object>,
    pub(crate) tab_index: i32,
    pub(crate) focused: bool,
    pub(crate) popover_visible: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<HtmlElementStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }

    let constructor = crate::webidl::create_function(
        scope,
        "HTMLElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_element_title_property::define(scope, prototype)?;
    super::html_element_lang_property::define(scope, prototype)?;
    super::html_element_translate_property::define(scope, prototype)?;
    super::html_element_dir_property::define(scope, prototype)?;
    super::html_element_hidden_property::define(scope, prototype)?;
    super::html_element_inert_property::define(scope, prototype)?;
    super::html_element_access_key_property::define(scope, prototype)?;
    super::html_element_draggable_property::define(scope, prototype)?;
    super::html_element_spellcheck_property::define(scope, prototype)?;
    super::html_element_autocapitalize_property::define(scope, prototype)?;
    super::html_element_edit_context_property::define(scope, prototype)?;
    super::html_element_content_editable_property::define(scope, prototype)?;
    super::html_element_enter_key_hint_property::define(scope, prototype)?;
    super::html_element_is_content_editable_property::define(scope, prototype)?;
    super::html_element_input_mode_property::define(scope, prototype)?;
    super::html_element_virtual_keyboard_policy_property::define(scope, prototype)?;
    super::html_element_offset_parent_property::define(scope, prototype)?;
    super::html_element_offset_top_property::define(scope, prototype)?;
    super::html_element_offset_left_property::define(scope, prototype)?;
    super::html_element_offset_width_property::define(scope, prototype)?;
    super::html_element_offset_height_property::define(scope, prototype)?;
    super::html_element_popover_property::define(scope, prototype)?;
    super::html_element_inner_text_property::define(scope, prototype)?;
    super::html_element_outer_text_property::define(scope, prototype)?;
    super::html_element_writing_suggestions_property::define(scope, prototype)?;
    super::html_element_onabort_property::define(scope, prototype)?;
    super::html_element_onbeforeinput_property::define(scope, prototype)?;
    super::html_element_onbeforematch_property::define(scope, prototype)?;
    super::html_element_onbeforetoggle_property::define(scope, prototype)?;
    super::html_element_onblur_property::define(scope, prototype)?;
    super::html_element_oncancel_property::define(scope, prototype)?;
    super::html_element_oncanplay_property::define(scope, prototype)?;
    super::html_element_oncanplaythrough_property::define(scope, prototype)?;
    super::html_element_onchange_property::define(scope, prototype)?;
    super::html_element_onclick_property::define(scope, prototype)?;
    super::html_element_onclose_property::define(scope, prototype)?;
    super::html_element_oncommand_property::define(scope, prototype)?;
    super::html_element_oncontentvisibilityautostatechange_property::define(scope, prototype)?;
    super::html_element_oncontextlost_property::define(scope, prototype)?;
    super::html_element_oncontextmenu_property::define(scope, prototype)?;
    super::html_element_oncontextrestored_property::define(scope, prototype)?;
    super::html_element_oncuechange_property::define(scope, prototype)?;
    super::html_element_ondblclick_property::define(scope, prototype)?;
    super::html_element_ondrag_property::define(scope, prototype)?;
    super::html_element_ondragend_property::define(scope, prototype)?;
    super::html_element_ondragenter_property::define(scope, prototype)?;
    super::html_element_ondragleave_property::define(scope, prototype)?;
    super::html_element_ondragover_property::define(scope, prototype)?;
    super::html_element_ondragstart_property::define(scope, prototype)?;
    super::html_element_ondrop_property::define(scope, prototype)?;
    super::html_element_ondurationchange_property::define(scope, prototype)?;
    super::html_element_onemptied_property::define(scope, prototype)?;
    super::html_element_onended_property::define(scope, prototype)?;
    super::html_element_onerror_property::define(scope, prototype)?;
    super::html_element_onfocus_property::define(scope, prototype)?;
    super::html_element_onformdata_property::define(scope, prototype)?;
    super::html_element_oninput_property::define(scope, prototype)?;
    super::html_element_oninvalid_property::define(scope, prototype)?;
    super::html_element_onkeydown_property::define(scope, prototype)?;
    super::html_element_onkeypress_property::define(scope, prototype)?;
    super::html_element_onkeyup_property::define(scope, prototype)?;
    super::html_element_onload_property::define(scope, prototype)?;
    super::html_element_onloadeddata_property::define(scope, prototype)?;
    super::html_element_onloadedmetadata_property::define(scope, prototype)?;
    super::html_element_onloadstart_property::define(scope, prototype)?;
    super::html_element_onmousedown_property::define(scope, prototype)?;
    super::html_element_onmouseenter_property::define(scope, prototype)?;
    super::html_element_onmouseleave_property::define(scope, prototype)?;
    super::html_element_onmousemove_property::define(scope, prototype)?;
    super::html_element_onmouseout_property::define(scope, prototype)?;
    super::html_element_onmouseover_property::define(scope, prototype)?;
    super::html_element_onmouseup_property::define(scope, prototype)?;
    super::html_element_onmousewheel_property::define(scope, prototype)?;
    super::html_element_onpause_property::define(scope, prototype)?;
    super::html_element_onplay_property::define(scope, prototype)?;
    super::html_element_onplaying_property::define(scope, prototype)?;
    super::html_element_onprogress_property::define(scope, prototype)?;
    super::html_element_onratechange_property::define(scope, prototype)?;
    super::html_element_onreset_property::define(scope, prototype)?;
    super::html_element_onresize_property::define(scope, prototype)?;
    super::html_element_onscroll_property::define(scope, prototype)?;
    super::html_element_onscrollend_property::define(scope, prototype)?;
    super::html_element_onsecuritypolicyviolation_property::define(scope, prototype)?;
    super::html_element_onseeked_property::define(scope, prototype)?;
    super::html_element_onseeking_property::define(scope, prototype)?;
    super::html_element_onselect_property::define(scope, prototype)?;
    super::html_element_onslotchange_property::define(scope, prototype)?;
    super::html_element_onstalled_property::define(scope, prototype)?;
    super::html_element_onsubmit_property::define(scope, prototype)?;
    super::html_element_onsuspend_property::define(scope, prototype)?;
    super::html_element_ontimeupdate_property::define(scope, prototype)?;
    super::html_element_ontoggle_property::define(scope, prototype)?;
    super::html_element_onvolumechange_property::define(scope, prototype)?;
    super::html_element_onwaiting_property::define(scope, prototype)?;
    super::html_element_onwebkitanimationend_property::define(scope, prototype)?;
    super::html_element_onwebkitanimationiteration_property::define(scope, prototype)?;
    super::html_element_onwebkitanimationstart_property::define(scope, prototype)?;
    super::html_element_onwebkittransitionend_property::define(scope, prototype)?;
    super::html_element_onwheel_property::define(scope, prototype)?;
    super::html_element_onauxclick_property::define(scope, prototype)?;
    super::html_element_ongotpointercapture_property::define(scope, prototype)?;
    super::html_element_onlostpointercapture_property::define(scope, prototype)?;
    super::html_element_onpointerdown_property::define(scope, prototype)?;
    super::html_element_onpointermove_property::define(scope, prototype)?;
    super::html_element_onpointerup_property::define(scope, prototype)?;
    super::html_element_onpointercancel_property::define(scope, prototype)?;
    super::html_element_onpointerover_property::define(scope, prototype)?;
    super::html_element_onpointerout_property::define(scope, prototype)?;
    super::html_element_onpointerenter_property::define(scope, prototype)?;
    super::html_element_onpointerleave_property::define(scope, prototype)?;
    super::html_element_onselectstart_property::define(scope, prototype)?;
    super::html_element_onselectionchange_property::define(scope, prototype)?;
    super::html_element_onanimationcancel_property::define(scope, prototype)?;
    super::html_element_onanimationend_property::define(scope, prototype)?;
    super::html_element_onanimationiteration_property::define(scope, prototype)?;
    super::html_element_onanimationstart_property::define(scope, prototype)?;
    super::html_element_ontransitionrun_property::define(scope, prototype)?;
    super::html_element_ontransitionstart_property::define(scope, prototype)?;
    super::html_element_ontransitionend_property::define(scope, prototype)?;
    super::html_element_ontransitioncancel_property::define(scope, prototype)?;
    super::html_element_onbeforexrselect_property::define(scope, prototype)?;
    super::html_element_oncopy_property::define(scope, prototype)?;
    super::html_element_oncut_property::define(scope, prototype)?;
    super::html_element_onpaste_property::define(scope, prototype)?;
    super::html_element_dataset_property::define(scope, prototype)?;
    super::html_element_nonce_property::define(scope, prototype)?;
    super::html_element_autofocus_property::define(scope, prototype)?;
    super::html_element_tab_index_property::define(scope, prototype)?;
    super::html_element_style_property::define(scope, prototype)?;
    super::html_element_attribute_style_map_property::define(scope, prototype)?;
    super::html_element_attach_internals::define(scope, prototype)?;
    super::html_element_blur::define(scope, prototype)?;
    super::html_element_click::define(scope, prototype)?;
    super::html_element_focus::define(scope, prototype)?;
    super::html_element_hide_popover::define(scope, prototype)?;
    super::html_element_show_popover::define(scope, prototype)?;
    super::html_element_toggle_popover::define(scope, prototype)?;
    super::html_element_onscrollsnapchange_property::define(scope, prototype)?;
    super::html_element_onscrollsnapchanging_property::define(scope, prototype)?;
    super::html_element_focus_group_property::define(scope, prototype)?;
    super::html_element_focus_group_start_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::html_element_onpointerrawupdate_property::define(scope, prototype)?;
    for name in ["ontouchcancel", "ontouchend", "ontouchmove", "ontouchstart"] {
        super::html_element_touch_handlers::define(scope, prototype, name)?;
    }
    let parent = super::element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;

    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlElementStore>()
        .ok_or_else(|| "HTMLElement state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    tag_name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLElement".to_owned());
    }
    attach(scope, object, tag_name);
    Ok(object)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    tag_name: &str,
) {
    super::element::attach(
        scope,
        object,
        tag_name.to_ascii_uppercase(),
        Some("http://www.w3.org/1999/xhtml".to_owned()),
    );
    let dataset = super::dom_string_map::create(scope, object)
        .unwrap_or_else(|_| tagged_object(scope, "DOMStringMap"));
    let dataset = v8::Global::new(scope, dataset);
    let attribute_style_map =
        super::style_property_map::create(scope).expect("StylePropertyMap must be available");
    let style = super::css_style_declaration::create(scope, "", None, Some(attribute_style_map))
        .expect("CSSStyleDeclaration must be available");
    super::css_style_declaration::bind_owner(scope, style, object);
    let style = v8::Global::new(scope, style);
    let attribute_style_map = v8::Global::new(scope, attribute_style_map);
    let mut strings = HashMap::new();
    strings.insert("contentEditable".to_owned(), "inherit".to_owned());
    strings.insert("writingSuggestions".to_owned(), "true".to_owned());
    let mut booleans = HashMap::new();
    booleans.insert("translate".to_owned(), true);
    booleans.insert("spellcheck".to_owned(), true);
    scope
        .get_slot_mut::<HtmlElementStore>()
        .expect("HTMLElement state")
        .records
        .insert(
            object.get_identity_hash().get(),
            HtmlElementRecord {
                strings,
                booleans,
                handlers: HashMap::new(),
                edit_context: None,
                internals: None,
                dataset,
                style,
                attribute_style_map,
                tab_index: -1,
                focused: false,
                popover_visible: false,
            },
        );
    super::custom_element_registry::track_candidate(scope, object);
}

pub(crate) fn is_html_element(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<HtmlElementStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    let name = format!("on{event_type}");
    let handler = record(scope, target).and_then(|record| record.handlers.get(&name).cloned());
    if let Some(handler) = handler {
        let value = v8::Local::new(scope, &handler);
        if let Ok(function) = v8::Local::<v8::Function>::try_from(value) {
            v8::tc_scope!(let try_catch, scope);
            let _ = function.call(try_catch, target.into(), &[event.into()]);
        }
        return;
    }
    let Some(source) = super::element::attribute_value(scope, target, &name) else {
        return;
    };
    let wrapped = format!("(function(event){{\n{source}\n}})");
    let Some(source) = v8::String::new(scope, &wrapped) else {
        return;
    };
    v8::tc_scope!(let try_catch, scope);
    let Some(value) =
        v8::Script::compile(try_catch, source, None).and_then(|script| script.run(try_catch))
    else {
        return;
    };
    if let Ok(function) = v8::Local::<v8::Function>::try_from(value) {
        let _ = function.call(try_catch, target.into(), &[event.into()]);
    }
}

pub(crate) fn set_content_attribute_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    name: &str,
    source: &str,
) {
    if record(scope, target).is_none() {
        return;
    }
    let function_name = name.to_ascii_lowercase();
    let wrapped = format!("(function {function_name}(event){{\n{source}\n}})");
    let Some(source) = v8::String::new(scope, &wrapped) else {
        return;
    };
    v8::tc_scope!(let try_catch, scope);
    let handler = v8::Script::compile(try_catch, source, None)
        .and_then(|script| script.run(try_catch))
        .filter(|value| value.is_function())
        .map(|value| v8::Global::new(try_catch, value));
    if let Some(record) = try_catch
        .get_slot_mut::<HtmlElementStore>()
        .and_then(|store| store.records.get_mut(&target.get_identity_hash().get()))
    {
        if let Some(handler) = handler {
            record.handlers.insert(function_name, handler);
        } else {
            record.handlers.remove(&function_name);
        }
    }
}

pub(crate) fn clear_content_attribute_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    name: &str,
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlElementStore>()
        .and_then(|store| store.records.get_mut(&target.get_identity_hash().get()))
    {
        record.handlers.remove(&name.to_ascii_lowercase());
    }
}

pub(crate) fn sync_dataset(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    attribute_name: &str,
    value: Option<&str>,
) {
    let _ = (scope, object, attribute_name, value);
}

pub(crate) fn sync_style_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    attribute_name: &str,
    value: Option<&str>,
) {
    if !attribute_name.eq_ignore_ascii_case("style") {
        return;
    }
    let Some(record) = record(scope, object) else {
        return;
    };
    let style = v8::Local::new(scope, &record.style);
    super::css_style_declaration::set_text_from_attribute(scope, style, value.unwrap_or_default());
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::custom_element_registry::html_constructor(scope, arguments, result, None);
}

pub(crate) fn tagged_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    let tag = v8::Symbol::get_to_string_tag(scope);
    if let Some(value) = v8::String::new(scope, name) {
        let _ = object.define_own_property(
            scope,
            tag.into(),
            value.into(),
            v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
        );
    }
    object
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<HtmlElementRecord> {
    scope
        .get_slot::<HtmlElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn reflected_attribute_name(name: &str) -> Option<&'static str> {
    match name {
        "title" => Some("title"),
        "lang" => Some("lang"),
        "dir" => Some("dir"),
        "accessKey" => Some("accesskey"),
        "autocapitalize" => Some("autocapitalize"),
        "contentEditable" => Some("contenteditable"),
        "enterKeyHint" => Some("enterkeyhint"),
        "inputMode" => Some("inputmode"),
        "virtualKeyboardPolicy" => Some("virtualkeyboardpolicy"),
        "popover" => Some("popover"),
        "nonce" => Some("nonce"),
        "writingSuggestions" => Some("writingsuggestions"),
        _ => None,
    }
}

pub(crate) fn get_edit_context(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.edit_context.as_ref() {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_edit_context(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .map(|value| v8::Global::new(scope, value));
    if let Some(record) = scope.get_slot_mut::<HtmlElementStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.edit_context = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_is_content_editable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let editable = record
        .strings
        .get("contentEditable")
        .is_some_and(|value| value == "true" || value == "plaintext-only");
    result.set(v8::Boolean::new(scope, editable).into());
}

pub(crate) fn get_offset_parent(
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

pub(crate) fn get_zero(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_dataset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_object(scope, arguments, result, |record| &record.dataset);
}

pub(crate) fn get_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_object(scope, arguments, result, |record| &record.style);
}

pub(crate) fn get_attribute_style_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_object(scope, arguments, result, |record| {
        &record.attribute_style_map
    });
}

pub(crate) fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&HtmlElementRecord) -> &v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let style = v8::Local::new(scope, &record.style);
    if let Some(key) = v8::String::new(scope, "cssText")
        && let Some(value) = arguments.get(0).to_string(scope)
    {
        let _ = style.set(scope, key.into(), value.into());
    }
}

pub(crate) fn get_tab_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(scope, arguments.this(), "tabindex")
        .and_then(|value| parse_tab_index(&value))
        .unwrap_or_else(|| default_tab_index(scope, arguments.this()));
    result.set(v8::Integer::new(scope, value).into());
}

pub(crate) fn set_tab_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).int32_value(scope).unwrap_or(0);
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::element::set_reflected_string(scope, arguments.this(), "tabindex", value.to_string());
}

fn parse_tab_index(value: &str) -> Option<i32> {
    let bytes = value.as_bytes();
    let mut position = bytes.iter().position(|byte| !byte.is_ascii_whitespace())?;
    let negative = match bytes.get(position) {
        Some(b'-') => {
            position += 1;
            true
        }
        Some(b'+') => {
            position += 1;
            false
        }
        _ => false,
    };
    let start = position;
    while bytes.get(position).is_some_and(u8::is_ascii_digit) {
        position += 1;
    }
    if position == start {
        return None;
    }
    let magnitude = value[start..position].parse::<i64>().ok()?;
    let signed = if negative { -magnitude } else { magnitude };
    i32::try_from(signed).ok()
}

fn default_tab_index(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> i32 {
    let Some(element) = super::element::record(scope, object) else {
        return -1;
    };
    if matches!(
        element.tag_name.as_str(),
        "A" | "AREA" | "BUTTON" | "IFRAME" | "INPUT" | "OBJECT" | "SELECT" | "TEXTAREA"
    ) || super::element::attribute_value(scope, object, "contenteditable")
        .is_some_and(|value| !value.eq_ignore_ascii_case("false"))
    {
        0
    } else {
        -1
    }
}

pub(crate) fn attach_internals(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let existing = record(scope, arguments.this()).map(|record| record.internals);
    let Some(existing) = existing else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if existing.is_some() {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Failed to execute 'attachInternals' on 'HTMLElement': ElementInternals for the specified element was already attached.",
        );
        return;
    }
    if !super::custom_element_registry::is_custom(scope, arguments.this()) {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Failed to execute 'attachInternals' on 'HTMLElement': Unable to attach ElementInternals to non-custom elements.",
        );
        return;
    }
    if super::custom_element_registry::internals_disabled(scope, arguments.this()) {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Failed to execute 'attachInternals' on 'HTMLElement': ElementInternals is disabled by disabledFeature static field.",
        );
        return;
    }
    match super::element_internals::create(scope, arguments.this(), None, None) {
        Ok(internals) => {
            let stored = v8::Global::new(scope, internals);
            if let Some(record) = scope.get_slot_mut::<HtmlElementStore>().and_then(|store| {
                store
                    .records
                    .get_mut(&arguments.this().get_identity_hash().get())
            }) {
                record.internals = Some(stored);
            }
            result.set(internals.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn blur(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = v8::Global::new(scope, arguments.this());
    if let Err(message) = blur_with_events(scope, target) {
        crate::webidl::throw_type_error(scope, &message);
    }
}

pub(crate) fn focus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = v8::Global::new(scope, arguments.this());
    if let Err(message) = focus_with_events(scope, target) {
        crate::webidl::throw_type_error(scope, &message);
    }
}

pub(crate) fn focus_with_events(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Global<v8::Object>,
) -> Result<bool, String> {
    let object = v8::Local::new(scope, &object);
    if !is_programmatically_focusable(scope, object) {
        return Ok(false);
    }
    let Some(document) = super::node::record(scope, object)
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document))
    else {
        return Ok(false);
    };
    let previous = super::document::stored_value(scope, document, "activeElement")
        .and_then(|value| v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &value)).ok())
        .map(|previous| v8::Global::new(scope, previous));
    if previous
        .as_ref()
        .is_some_and(|previous| v8::Local::new(scope, previous).strict_equals(object.into()))
    {
        return Ok(false);
    }
    let object = v8::Global::new(scope, object);
    if let Some(previous) = previous.as_ref() {
        let previous_local = v8::Local::new(scope, previous);
        set_focused(scope, previous_local, false);
        dispatch_trusted_focus_event(scope, previous.clone(), "blur", false, Some(object.clone()))?;
        dispatch_trusted_focus_event(
            scope,
            previous.clone(),
            "focusout",
            true,
            Some(object.clone()),
        )?;
    }
    let object_local = v8::Local::new(scope, &object);
    set_focused(scope, object_local, true);
    dispatch_trusted_focus_event(scope, object.clone(), "focus", false, previous.clone())?;
    dispatch_trusted_focus_event(scope, object, "focusin", true, previous)?;
    Ok(true)
}

pub(crate) fn blur_with_events(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Global<v8::Object>,
) -> Result<bool, String> {
    let object = v8::Local::new(scope, &object);
    let Some(document) = super::node::record(scope, object)
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document))
    else {
        return Ok(false);
    };
    let active = super::document::stored_value(scope, document, "activeElement")
        .and_then(|value| v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &value)).ok());
    if !active.is_some_and(|active| active.strict_equals(object.into())) {
        return Ok(false);
    }
    set_focused(scope, object, false);
    let object = v8::Global::new(scope, object);
    dispatch_trusted_focus_event(scope, object.clone(), "blur", false, None)?;
    dispatch_trusted_focus_event(scope, object, "focusout", true, None)?;
    Ok(true)
}

fn dispatch_trusted_focus_event(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Global<v8::Object>,
    event_type: &str,
    bubbles: bool,
    related_target: Option<v8::Global<v8::Object>>,
) -> Result<(), String> {
    let event =
        super::focus_event::create_with_data(scope, event_type, bubbles, true, related_target)?;
    super::event::set_trusted(scope, event, true);
    let target = v8::Local::new(scope, &target);
    super::event_target::dispatch(scope, target, event);
    Ok(())
}

pub(crate) fn is_programmatically_focusable(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(node) = super::node::record(scope, object) else {
        return false;
    };
    if !super::node::is_connected(scope, object)
        || !super::element_layout::compute(scope, object).rendered
        || super::element::attribute_value(scope, object, "disabled").is_some()
        || has_inert_ancestor(scope, object)
        || disabled_by_fieldset(scope, object, &node.node_name)
    {
        return false;
    }
    let visibility =
        super::get_computed_style_global::computed_property_value(scope, object, "visibility");
    if matches!(
        visibility.to_ascii_lowercase().as_str(),
        "hidden" | "collapse"
    ) {
        return false;
    }
    if node.node_name == "INPUT"
        && super::element::attribute_value(scope, object, "type")
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
    {
        return false;
    }
    if matches!(
        node.node_name.as_str(),
        "BUTTON" | "INPUT" | "SELECT" | "TEXTAREA" | "IFRAME" | "OBJECT"
    ) {
        return true;
    }
    if matches!(node.node_name.as_str(), "A" | "AREA")
        && super::element::attribute_value(scope, object, "href").is_some()
    {
        return true;
    }
    if super::element::attribute_value(scope, object, "contenteditable")
        .is_some_and(|value| !value.eq_ignore_ascii_case("false"))
    {
        return true;
    }
    super::element::attribute_value(scope, object, "tabindex")
        .and_then(|value| value.parse::<i32>().ok())
        .is_some()
}

pub(crate) fn is_sequentially_focusable(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    if !is_programmatically_focusable(scope, object) {
        return false;
    }
    super::element::attribute_value(scope, object, "tabindex")
        .and_then(|value| value.parse::<i32>().ok())
        .is_none_or(|value| value >= 0)
}

fn has_inert_ancestor(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    let mut current = Some(object);
    while let Some(candidate) = current {
        if super::element::record(scope, candidate).is_some()
            && super::element::attribute_value(scope, candidate, "inert").is_some()
        {
            return true;
        }
        current = super::node::parent(scope, candidate);
    }
    false
}

pub(crate) fn disabled_by_fieldset(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    node_name: &str,
) -> bool {
    if !matches!(node_name, "BUTTON" | "INPUT" | "SELECT" | "TEXTAREA") {
        return false;
    }
    let mut ancestor = super::node::parent(scope, object);
    while let Some(candidate) = ancestor {
        if super::html_field_set_element::record(scope, candidate).is_some_and(|record| {
            record.disabled
                || super::element::attribute_value(scope, candidate, "disabled").is_some()
        }) {
            let first_legend = super::node::children(scope, candidate)
                .into_iter()
                .find(|child| {
                    super::element::record(scope, *child)
                        .is_some_and(|record| record.tag_name == "LEGEND")
                });
            let inside_first_legend = first_legend.is_some_and(|legend| {
                let mut current = Some(object);
                while let Some(node) = current {
                    if node.get_identity_hash() == legend.get_identity_hash() {
                        return true;
                    }
                    if node.get_identity_hash() == candidate.get_identity_hash() {
                        break;
                    }
                    current = super::node::parent(scope, node);
                }
                false
            });
            if !inside_first_legend {
                return true;
            }
        }
        ancestor = super::node::parent(scope, candidate);
    }
    false
}

pub(crate) fn set_focused(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: bool,
) {
    let valid = scope.get_slot::<HtmlElementStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if value {
        clear_focus(scope);
        super::svg_element::clear_focus(scope);
        super::math_ml_element::clear_focus(scope);
    }
    if let Some(record) = scope
        .get_slot_mut::<HtmlElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.focused = value;
    }
    super::element::update_document_focus(scope, object, value);
}

pub(crate) fn clear_focus(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<HtmlElementStore>() {
        for record in store.records.values_mut() {
            record.focused = false;
        }
    }
}

pub(crate) fn click(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.handlers.get("onclick")
        && let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, handler))
    {
        let receiver: v8::Local<v8::Value> = arguments.this().into();
        let event = super::event::create(scope, "click")
            .map(v8::Local::<v8::Value>::from)
            .unwrap_or_else(|_| v8::undefined(scope).into());
        let _ = function.call(scope, receiver, &[event]);
        return;
    }
    if let Some(source) = super::element::attribute_value(scope, arguments.this(), "onclick") {
        let wrapped = format!("(function(event){{\n{source}\n}})");
        let Some(source) = v8::String::new(scope, &wrapped) else {
            return;
        };
        v8::tc_scope!(let try_catch, scope);
        let Some(value) =
            v8::Script::compile(try_catch, source, None).and_then(|script| script.run(try_catch))
        else {
            return;
        };
        let Ok(function) = v8::Local::<v8::Function>::try_from(value) else {
            return;
        };
        let event = super::event::create(try_catch, "click")
            .map(v8::Local::<v8::Value>::from)
            .unwrap_or_else(|_| v8::undefined(try_catch).into());
        let receiver: v8::Local<v8::Value> = arguments.this().into();
        let _ = function.call(try_catch, receiver, &[event]);
    }
}

pub(crate) fn hide_popover(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_popover_visible(scope, arguments.this(), false);
}

pub(crate) fn show_popover(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_popover_visible(scope, arguments.this(), true);
}

pub(crate) fn toggle_popover(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let force = arguments.get(0);
    let forced_value = (!force.is_undefined()).then(|| force.boolean_value(scope));
    let identity = arguments.this().get_identity_hash().get();
    let visible = if let Some(record) = scope
        .get_slot_mut::<HtmlElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.popover_visible = forced_value.unwrap_or(!record.popover_visible);
        Some(record.popover_visible)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        None
    };
    if let Some(visible) = visible {
        result.set(v8::Boolean::new(scope, visible).into());
    }
}

pub(crate) fn set_popover_visible(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: bool,
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.popover_visible = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
