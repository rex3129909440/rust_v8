use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DocumentStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, DocumentRecord>,
}

#[derive(Clone)]
struct DocumentRecord {
    content_type: String,
    write_open: bool,
    write_buffer: String,
    current_script: Option<v8::Global<v8::Object>>,
    values: HashMap<String, v8::Global<v8::Value>>,
    handlers: HashMap<String, v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentStore::default());
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<DocumentStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Document",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::document_implementation_property::define(scope, prototype)?;
    super::document_url_property::define(scope, prototype)?;
    super::document_uri_property::define(scope, prototype)?;
    super::document_compat_mode_property::define(scope, prototype)?;
    super::document_character_set_property::define(scope, prototype)?;
    super::document_charset_property::define(scope, prototype)?;
    super::document_input_encoding_property::define(scope, prototype)?;
    super::document_content_type_property::define(scope, prototype)?;
    super::document_doctype_property::define(scope, prototype)?;
    super::document_document_element_property::define(scope, prototype)?;
    super::document_xml_encoding_property::define(scope, prototype)?;
    super::document_xml_version_property::define(scope, prototype)?;
    super::document_xml_standalone_property::define(scope, prototype)?;
    super::document_domain_property::define(scope, prototype)?;
    super::document_referrer_property::define(scope, prototype)?;
    super::document_cookie::define(scope, prototype)?;
    super::document_last_modified_property::define(scope, prototype)?;
    super::document_ready_state_property::define(scope, prototype)?;
    super::document_title_property::define(scope, prototype)?;
    super::document_dir_property::define(scope, prototype)?;
    super::document_body_property::define(scope, prototype)?;
    super::document_head_property::define(scope, prototype)?;
    super::document_images_property::define(scope, prototype)?;
    super::document_embeds_property::define(scope, prototype)?;
    super::document_plugins_property::define(scope, prototype)?;
    super::document_links_property::define(scope, prototype)?;
    super::document_forms_property::define(scope, prototype)?;
    super::document_scripts_property::define(scope, prototype)?;
    super::document_current_script_property::define(scope, prototype)?;
    super::document_default_view_property::define(scope, prototype)?;
    super::document_design_mode_property::define(scope, prototype)?;
    super::document_onreadystatechange::define(scope, prototype)?;
    super::document_anchors_property::define(scope, prototype)?;
    super::document_applets_property::define(scope, prototype)?;
    super::document_fg_color_property::define(scope, prototype)?;
    super::document_link_color_property::define(scope, prototype)?;
    super::document_vlink_color_property::define(scope, prototype)?;
    super::document_alink_color_property::define(scope, prototype)?;
    super::document_bg_color_property::define(scope, prototype)?;
    super::document_all_property::define(scope, prototype)?;
    super::document_scrolling_element_property::define(scope, prototype)?;
    super::document_onpointerlockchange::define(scope, prototype)?;
    super::document_onpointerlockerror::define(scope, prototype)?;
    super::document_hidden_property::define(scope, prototype)?;
    super::document_visibility_state_property::define(scope, prototype)?;
    super::document_was_discarded_property::define(scope, prototype)?;
    super::document_prerendering_property::define(scope, prototype)?;
    super::document_feature_policy_property::define(scope, prototype)?;
    super::document_webkit_visibility_state_property::define(scope, prototype)?;
    super::document_webkit_hidden_property::define(scope, prototype)?;
    super::document_onbeforecopy::define(scope, prototype)?;
    super::document_onbeforecut::define(scope, prototype)?;
    super::document_onbeforepaste::define(scope, prototype)?;
    super::document_onfreeze::define(scope, prototype)?;
    super::document_onprerenderingchange::define(scope, prototype)?;
    super::document_onresume::define(scope, prototype)?;
    super::document_onsearch::define(scope, prototype)?;
    super::document_onvisibilitychange::define(scope, prototype)?;
    super::document_timeline_property::define(scope, prototype)?;
    super::document_fullscreen_enabled_property::define(scope, prototype)?;
    super::document_fullscreen_property::define(scope, prototype)?;
    super::document_onfullscreenchange::define(scope, prototype)?;
    super::document_onfullscreenerror::define(scope, prototype)?;
    super::document_webkit_is_full_screen_property::define(scope, prototype)?;
    super::document_webkit_current_full_screen_element_property::define(scope, prototype)?;
    super::document_webkit_fullscreen_enabled_property::define(scope, prototype)?;
    super::document_webkit_fullscreen_element_property::define(scope, prototype)?;
    super::document_onwebkitfullscreenchange::define(scope, prototype)?;
    super::document_onwebkitfullscreenerror::define(scope, prototype)?;
    super::document_root_element_property::define(scope, prototype)?;
    super::document_picture_in_picture_enabled_property::define(scope, prototype)?;
    super::document_onabort::define(scope, prototype)?;
    super::document_onbeforeinput::define(scope, prototype)?;
    super::document_onbeforematch::define(scope, prototype)?;
    super::document_onbeforetoggle::define(scope, prototype)?;
    super::document_onblur::define(scope, prototype)?;
    super::document_oncancel::define(scope, prototype)?;
    super::document_oncanplay::define(scope, prototype)?;
    super::document_oncanplaythrough::define(scope, prototype)?;
    super::document_onchange::define(scope, prototype)?;
    super::document_onclick::define(scope, prototype)?;
    super::document_onclose::define(scope, prototype)?;
    super::document_oncommand::define(scope, prototype)?;
    super::document_oncontentvisibilityautostatechange::define(scope, prototype)?;
    super::document_oncontextlost::define(scope, prototype)?;
    super::document_oncontextmenu::define(scope, prototype)?;
    super::document_oncontextrestored::define(scope, prototype)?;
    super::document_oncuechange::define(scope, prototype)?;
    super::document_ondblclick::define(scope, prototype)?;
    super::document_ondrag::define(scope, prototype)?;
    super::document_ondragend::define(scope, prototype)?;
    super::document_ondragenter::define(scope, prototype)?;
    super::document_ondragleave::define(scope, prototype)?;
    super::document_ondragover::define(scope, prototype)?;
    super::document_ondragstart::define(scope, prototype)?;
    super::document_ondrop::define(scope, prototype)?;
    super::document_ondurationchange::define(scope, prototype)?;
    super::document_onemptied::define(scope, prototype)?;
    super::document_onended::define(scope, prototype)?;
    super::document_onerror::define(scope, prototype)?;
    super::document_onfocus::define(scope, prototype)?;
    super::document_onformdata::define(scope, prototype)?;
    super::document_oninput::define(scope, prototype)?;
    super::document_oninvalid::define(scope, prototype)?;
    super::document_onkeydown::define(scope, prototype)?;
    super::document_onkeypress::define(scope, prototype)?;
    super::document_onkeyup::define(scope, prototype)?;
    super::document_onload::define(scope, prototype)?;
    super::document_onloadeddata::define(scope, prototype)?;
    super::document_onloadedmetadata::define(scope, prototype)?;
    super::document_onloadstart::define(scope, prototype)?;
    super::document_onmousedown::define(scope, prototype)?;
    super::document_onmouseenter::define(scope, prototype)?;
    super::document_onmouseleave::define(scope, prototype)?;
    super::document_onmousemove::define(scope, prototype)?;
    super::document_onmouseout::define(scope, prototype)?;
    super::document_onmouseover::define(scope, prototype)?;
    super::document_onmouseup::define(scope, prototype)?;
    super::document_onmousewheel::define(scope, prototype)?;
    super::document_onpause::define(scope, prototype)?;
    super::document_onplay::define(scope, prototype)?;
    super::document_onplaying::define(scope, prototype)?;
    super::document_onprogress::define(scope, prototype)?;
    super::document_onratechange::define(scope, prototype)?;
    super::document_onreset::define(scope, prototype)?;
    super::document_onresize::define(scope, prototype)?;
    super::document_onscroll::define(scope, prototype)?;
    super::document_onscrollend::define(scope, prototype)?;
    super::document_onsecuritypolicyviolation::define(scope, prototype)?;
    super::document_onseeked::define(scope, prototype)?;
    super::document_onseeking::define(scope, prototype)?;
    super::document_onselect::define(scope, prototype)?;
    super::document_onslotchange::define(scope, prototype)?;
    super::document_onstalled::define(scope, prototype)?;
    super::document_onsubmit::define(scope, prototype)?;
    super::document_onsuspend::define(scope, prototype)?;
    super::document_ontimeupdate::define(scope, prototype)?;
    super::document_ontoggle::define(scope, prototype)?;
    super::document_onvolumechange::define(scope, prototype)?;
    super::document_onwaiting::define(scope, prototype)?;
    super::document_onwebkitanimationend::define(scope, prototype)?;
    super::document_onwebkitanimationiteration::define(scope, prototype)?;
    super::document_onwebkitanimationstart::define(scope, prototype)?;
    super::document_onwebkittransitionend::define(scope, prototype)?;
    super::document_onwheel::define(scope, prototype)?;
    super::document_onauxclick::define(scope, prototype)?;
    super::document_ongotpointercapture::define(scope, prototype)?;
    super::document_onlostpointercapture::define(scope, prototype)?;
    super::document_onpointerdown::define(scope, prototype)?;
    super::document_onpointermove::define(scope, prototype)?;
    super::document_onpointerup::define(scope, prototype)?;
    super::document_onpointercancel::define(scope, prototype)?;
    super::document_onpointerover::define(scope, prototype)?;
    super::document_onpointerout::define(scope, prototype)?;
    super::document_onpointerenter::define(scope, prototype)?;
    super::document_onpointerleave::define(scope, prototype)?;
    super::document_onselectstart::define(scope, prototype)?;
    super::document_onselectionchange::define(scope, prototype)?;
    super::document_onanimationcancel::define(scope, prototype)?;
    super::document_onanimationend::define(scope, prototype)?;
    super::document_onanimationiteration::define(scope, prototype)?;
    super::document_onanimationstart::define(scope, prototype)?;
    super::document_ontransitionrun::define(scope, prototype)?;
    super::document_ontransitionstart::define(scope, prototype)?;
    super::document_ontransitionend::define(scope, prototype)?;
    super::document_ontransitioncancel::define(scope, prototype)?;
    super::document_onbeforexrselect::define(scope, prototype)?;
    super::document_oncopy::define(scope, prototype)?;
    super::document_oncut::define(scope, prototype)?;
    super::document_onpaste::define(scope, prototype)?;
    super::document_children_property::define(scope, prototype)?;
    super::document_first_element_child_property::define(scope, prototype)?;
    super::document_last_element_child_property::define(scope, prototype)?;
    super::document_child_element_count_property::define(scope, prototype)?;
    super::document_active_element_property::define(scope, prototype)?;
    super::document_style_sheets_property::define(scope, prototype)?;
    super::document_pointer_lock_element_property::define(scope, prototype)?;
    super::document_fullscreen_element_property::define(scope, prototype)?;
    super::document_adopted_style_sheets_property::define(scope, prototype)?;
    super::document_picture_in_picture_element_property::define(scope, prototype)?;
    super::document_fonts_property::define(scope, prototype)?;
    super::document_adopt_node::define(scope, prototype)?;
    super::parent_node_append::define(scope, prototype)?;
    super::document_capture_events::define(scope, prototype)?;
    super::document_caret_position_from_point::define(scope, prototype)?;
    super::document_caret_range_from_point::define(scope, prototype)?;
    super::document_clear::define(scope, prototype)?;
    super::document_close::define(scope, prototype)?;
    super::document_create_attribute::define(scope, prototype)?;
    super::document_create_attribute_ns::define(scope, prototype)?;
    super::document_create_cdata_section::define(scope, prototype)?;
    super::document_create_comment::define(scope, prototype)?;
    super::document_create_document_fragment::define(scope, prototype)?;
    super::document_create_element::define(scope, prototype)?;
    super::document_create_element_ns::define(scope, prototype)?;
    super::document_create_event::define(scope, prototype)?;
    super::document_create_expression::define(scope, prototype)?;
    super::document_create_ns_resolver::define(scope, prototype)?;
    super::document_create_node_iterator::define(scope, prototype)?;
    super::document_create_processing_instruction::define(scope, prototype)?;
    super::document_create_range::define(scope, prototype)?;
    super::document_create_text_node::define(scope, prototype)?;
    super::document_create_tree_walker::define(scope, prototype)?;
    super::document_element_from_point::define(scope, prototype)?;
    super::document_elements_from_point::define(scope, prototype)?;
    super::document_evaluate::define(scope, prototype)?;
    super::document_exec_command::define(scope, prototype)?;
    super::document_exit_fullscreen::define(scope, prototype)?;
    super::document_exit_picture_in_picture::define(scope, prototype)?;
    super::document_exit_pointer_lock::define(scope, prototype)?;
    super::document_get_animations::define(scope, prototype)?;
    super::document_get_element_by_id::define(scope, prototype)?;
    super::document_get_elements_by_class_name::define(scope, prototype)?;
    super::document_get_elements_by_name::define(scope, prototype)?;
    super::document_get_elements_by_tag_name::define(scope, prototype)?;
    super::document_get_elements_by_tag_name_ns::define(scope, prototype)?;
    super::document_get_selection::define(scope, prototype)?;
    super::document_has_focus::define(scope, prototype)?;
    super::document_has_storage_access::define(scope, prototype)?;
    super::document_has_unpartitioned_cookie_access::define(scope, prototype)?;
    super::document_import_node::define(scope, prototype)?;
    super::parent_node_move_before::define(scope, prototype)?;
    super::document_open::define(scope, prototype)?;
    super::parent_node_prepend::define(scope, prototype)?;
    super::document_query_command_enabled::define(scope, prototype)?;
    super::document_query_command_indeterm::define(scope, prototype)?;
    super::document_query_command_state::define(scope, prototype)?;
    super::document_query_command_supported::define(scope, prototype)?;
    super::document_query_command_value::define(scope, prototype)?;
    super::document_query_selector::define(scope, prototype)?;
    super::document_query_selector_all::define(scope, prototype)?;
    super::document_release_events::define(scope, prototype)?;
    super::parent_node_replace_children::define(scope, prototype)?;
    super::document_request_storage_access::define(scope, prototype)?;
    super::document_request_storage_access_for::define(scope, prototype)?;
    super::document_start_view_transition::define(scope, prototype)?;
    super::document_webkit_cancel_full_screen::define(scope, prototype)?;
    super::document_webkit_exit_fullscreen::define(scope, prototype)?;
    super::document_write::define(scope, prototype)?;
    super::document_writeln::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::document_fragment_directive_property::define(scope, prototype)?;
    super::document_onpointerrawupdate::define(scope, prototype)?;
    super::document_browsing_topics::define(scope, prototype)?;
    super::document_has_private_token::define(scope, prototype)?;
    super::document_has_redemption_record::define(scope, prototype)?;
    super::document_active_view_transition_property::define(scope, prototype)?;
    super::document_onscrollsnapchange::define(scope, prototype)?;
    super::document_onscrollsnapchanging::define(scope, prototype)?;
    super::document_custom_element_registry_property::define(scope, prototype)?;
    super::document_aria_notify::define(scope, prototype)?;
    let unscopables = crate::webidl::new_unscopables(scope)?;
    crate::webidl::define_unscopable(scope, unscopables, "append")?;
    crate::webidl::define_unscopable(scope, unscopables, "fullscreen")?;
    crate::webidl::define_unscopable(scope, unscopables, "prepend")?;
    crate::webidl::define_unscopable(scope, unscopables, "replaceChildren")?;
    crate::webidl::attach_unscopables(scope, prototype, unscopables)?;
    let parent = super::node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DocumentStore>()
        .ok_or_else(|| "Document state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Document", constructor.into())
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    content_type: String,
) {
    if let Some(store) = scope.get_slot_mut::<DocumentStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            DocumentRecord {
                content_type,
                write_open: false,
                write_buffer: String::new(),
                current_script: None,
                values: HashMap::new(),
                handlers: HashMap::new(),
            },
        );
    }
}

pub(crate) fn parse_source(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    source: &str,
) -> Result<(), String> {
    if !is_document(scope, document) {
        return Err("The target is not a Document".to_owned());
    }
    if content_type(scope, document) == Some("text/html") {
        return super::document_html_parser::parse_page(scope, document, source);
    }
    for child in super::node::children(scope, document) {
        super::node::detach(scope, child);
    }
    let mut markup = source.to_owned();
    let lower = markup.to_ascii_lowercase();
    if let Some(start) = lower.find("<!doctype")
        && let Some(relative_end) = markup[start..].find('>')
    {
        let end = start + relative_end;
        let declaration = markup[start + 9..end].trim();
        let name = declaration
            .split_ascii_whitespace()
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or("html");
        let doctype = super::document_type::create(scope, name, "", "")?;
        super::node::set_owner_document(scope, doctype, document);
        super::node::insert_node(scope, document, doctype, 0)
            .map_err(|(_, message)| message.to_owned())?;
        markup.replace_range(start..=end, "");
    }
    let parsed = super::dom_html::parse_fragment(scope, document, &markup)?;
    let mut insertion = super::node::children(scope, document).len();
    for child in parsed {
        let child = v8::Local::new(scope, &child);
        let ignorable_whitespace = super::node::record(scope, child)
            .is_some_and(|record| record.node_type == 3)
            && super::character_data::data_if_character(scope, child)
                .is_some_and(|data| data.trim().is_empty());
        if ignorable_whitespace {
            continue;
        }
        super::node::insert_node(scope, document, child, insertion)
            .map_err(|(_, message)| message.to_owned())?;
        insertion += 1;
    }
    Ok(())
}

pub(crate) fn set_string_value(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        remember_value(scope, document, name, value.into());
    }
}

pub(crate) fn set_object_value(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Object>,
) {
    remember_value(scope, document, name, value.into());
}

pub(crate) fn clear_value(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
) {
    if let Some(record) = scope
        .get_slot_mut::<DocumentStore>()
        .and_then(|store| store.records.get_mut(&document.get_identity_hash().get()))
    {
        record.values.remove(name);
    }
}

pub(crate) fn set_write_open(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    open: bool,
) {
    if let Some(record) = scope
        .get_slot_mut::<DocumentStore>()
        .and_then(|store| store.records.get_mut(&document.get_identity_hash().get()))
    {
        record.write_open = open;
        if open {
            record.write_buffer.clear();
        }
    }
}

pub(crate) fn buffer_open_document_write(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    markup: &str,
) -> Option<String> {
    let record = scope
        .get_slot_mut::<DocumentStore>()?
        .records
        .get_mut(&document.get_identity_hash().get())?;
    if !record.write_open {
        return None;
    }
    record.write_buffer.push_str(markup);
    Some(record.write_buffer.clone())
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
    }
}

pub(crate) fn handler_value(
    scope: &v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Value>> {
    record(scope, document)?.handlers.get(name).cloned()
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> bool {
    let handler = value.is_object().then(|| v8::Global::new(scope, value));
    let present = handler.is_some();
    let Some(record) = scope
        .get_slot_mut::<DocumentStore>()
        .and_then(|store| store.records.get_mut(&document.get_identity_hash().get()))
    else {
        return false;
    };
    if let Some(handler) = handler {
        record.handlers.insert(name.to_owned(), handler);
    } else {
        record.handlers.remove(name);
    }
    super::event_target::set_attribute_handler(
        scope,
        document,
        name.strip_prefix("on").unwrap_or(name),
        present,
    );
    true
}

pub(crate) fn create_empty<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let document = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, document, prototype.into()) != Some(true) {
        return Err("cannot create Document".to_owned());
    }
    super::node::attach(scope, document, 9, "#document".to_owned(), None);
    attach(scope, document, "text/html".to_owned());
    Ok(document)
}

pub(crate) fn serialize_if_document(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    is_document(scope, object).then(|| super::dom_html::serialize_children(scope, object))
}

pub(crate) fn searchable_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> String {
    if is_document(scope, object) {
        super::node::text_content(scope, object)
    } else {
        String::new()
    }
}

pub(crate) fn remember_value(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    let value = v8::Global::new(scope, value);
    if let Some(document) = scope
        .get_slot_mut::<DocumentStore>()
        .and_then(|store| store.records.get_mut(&document.get_identity_hash().get()))
    {
        document.values.insert(name.to_owned(), value);
    }
}

pub(crate) fn forget_value(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
) {
    if let Some(document) = scope
        .get_slot_mut::<DocumentStore>()
        .and_then(|store| store.records.get_mut(&document.get_identity_hash().get()))
    {
        document.values.remove(name);
    }
}

pub(crate) fn document_child_elements<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::node::children(scope, document)
        .into_iter()
        .filter(|child| {
            super::node::record(scope, *child).is_some_and(|record| record.node_type == 1)
        })
        .collect()
}

pub(crate) fn document_descendants<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::dom_selector::descendants(scope, document)
}

pub(crate) fn create_svg_element<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    local_name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    match local_name {
        "a" => super::svg_a_element::create(scope, None),
        "animate" => super::svg_animate_element::create(scope, None),
        "animateMotion" => super::svg_animate_motion_element::create(scope, None),
        "animateTransform" => super::svg_animate_transform_element::create(scope, None),
        "circle" => super::svg_circle_element::create(scope, None),
        "clipPath" => super::svg_clip_path_element::create(scope, None),
        "defs" => super::svg_defs_element::create(scope, None),
        "desc" => super::svg_desc_element::create(scope, None),
        "ellipse" => super::svg_ellipse_element::create(scope, None),
        "feBlend" => super::svg_fe_blend_element::create(scope, None),
        "feColorMatrix" => super::svg_fe_color_matrix_element::create(scope, None),
        "feComponentTransfer" => super::svg_fe_component_transfer_element::create(scope, None),
        "feComposite" => super::svg_fe_composite_element::create(scope, None),
        "feConvolveMatrix" => super::svg_fe_convolve_matrix_element::create(scope, None),
        "feDiffuseLighting" => super::svg_fe_diffuse_lighting_element::create(scope, None),
        "feDisplacementMap" => super::svg_fe_displacement_map_element::create(scope, None),
        "feDistantLight" => super::svg_fe_distant_light_element::create(scope, None),
        "feDropShadow" => super::svg_fe_drop_shadow_element::create(scope, None),
        "feFlood" => super::svg_fe_flood_element::create(scope, None),
        "feFuncA" => super::svg_fe_func_a_element::create(scope, None),
        "feFuncB" => super::svg_fe_func_b_element::create(scope, None),
        "feFuncG" => super::svg_fe_func_g_element::create(scope, None),
        "feFuncR" => super::svg_fe_func_r_element::create(scope, None),
        "feGaussianBlur" => super::svg_fe_gaussian_blur_element::create(scope, None),
        "feImage" => super::svg_fe_image_element::create(scope, None),
        "feMerge" => super::svg_fe_merge_element::create(scope, None),
        "feMergeNode" => super::svg_fe_merge_node_element::create(scope, None),
        "feMorphology" => super::svg_fe_morphology_element::create(scope, None),
        "feOffset" => super::svg_fe_offset_element::create(scope, None),
        "fePointLight" => super::svg_fe_point_light_element::create(scope, None),
        "feSpecularLighting" => super::svg_fe_specular_lighting_element::create(scope, None),
        "feSpotLight" => super::svg_fe_spot_light_element::create(scope, None),
        "feTile" => super::svg_fe_tile_element::create(scope, None),
        "feTurbulence" => super::svg_fe_turbulence_element::create(scope, None),
        "filter" => super::svg_filter_element::create(scope, None),
        "foreignObject" => super::svg_foreign_object_element::create(scope, None),
        "g" => super::svg_g_element::create(scope, None),
        "image" => super::svg_image_element::create(scope, None),
        "line" => super::svg_line_element::create(scope, None),
        "linearGradient" => super::svg_linear_gradient_element::create(scope, None),
        "marker" => super::svg_marker_element::create(scope, None),
        "mask" => super::svg_mask_element::create(scope, None),
        "metadata" => super::svg_metadata_element::create(scope, None),
        "mpath" => super::svg_m_path_element::create(scope, None),
        "path" => super::svg_path_element::create(scope, None, 0.0),
        "pattern" => super::svg_pattern_element::create(scope, None),
        "polygon" => super::svg_polygon_element::create(scope, None),
        "polyline" => super::svg_polyline_element::create(scope, None),
        "radialGradient" => super::svg_radial_gradient_element::create(scope, None),
        "rect" => super::svg_rect_element::create(scope, None),
        "script" => super::svg_script_element::create(scope, None),
        "set" => super::svg_set_element::create(scope, None),
        "stop" => super::svg_stop_element::create(scope, None),
        "style" => super::svg_style_element::create(scope, None),
        "svg" => super::svg_svg_element::create(scope),
        "switch" => super::svg_switch_element::create(scope, None),
        "symbol" => super::svg_symbol_element::create(scope, None),
        "text" => super::svg_text_element::create(scope, None, ""),
        "textPath" => super::svg_text_path_element::create(scope, None, ""),
        "title" => super::svg_title_element::create(scope, None),
        "tspan" => super::svg_tspan_element::create(scope, None, ""),
        "use" => super::svg_use_element::create(scope, None),
        "view" => super::svg_view_element::create(scope, None),
        _ => super::svg_element::create(scope, local_name, None),
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Document': Illegal constructor");
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DocumentRecord> {
    scope
        .get_slot::<DocumentStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_document(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

pub(crate) fn stored_value(
    scope: &v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Value>> {
    record(scope, document)?.values.get(name).cloned()
}

pub(crate) fn current_script<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    record(scope, document)?
        .current_script
        .map(|script| v8::Local::new(scope, &script))
}

pub(crate) fn swap_current_script(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    script: Option<v8::Local<'_, v8::Object>>,
) -> Option<v8::Global<v8::Object>> {
    let script = script.map(|script| v8::Global::new(scope, script));
    scope
        .get_slot_mut::<DocumentStore>()
        .and_then(|store| store.records.get_mut(&document.get_identity_hash().get()))
        .and_then(|record| std::mem::replace(&mut record.current_script, script))
}

pub(crate) fn is_write_open(
    scope: &v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Option<bool> {
    Some(record(scope, document)?.write_open)
}

pub(crate) fn content_type(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<&'static str> {
    record(scope, object).map(|record| {
        if record.content_type == "text/html" {
            "text/html"
        } else if record.content_type == "application/xml" {
            "application/xml"
        } else {
            "other"
        }
    })
}

pub(crate) fn content_type_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.content_type)
}

pub(crate) fn set_content_type(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    content_type: String,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<DocumentStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.content_type = content_type;
    true
}

pub(crate) fn create_html_element_by_name<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    normalized: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    match normalized {
        "video" => super::html_video_element::create(scope, String::new()),
        "fencedframe" => super::html_fenced_frame_element::create(scope),
        "geolocation" => super::html_geolocation_element::create(scope),
        "ul" => super::html_u_list_element::create(scope),
        "track" => super::html_track_element::create(scope),
        "title" => super::html_title_element::create(scope),
        "time" => super::html_time_element::create(scope),
        "textarea" => super::html_text_area_element::create(scope),
        "template" => super::html_template_element::create(scope),
        "thead" => super::html_table_section_element::create(scope, "THEAD"),
        "tbody" => super::html_table_section_element::create(scope, "TBODY"),
        "tfoot" => super::html_table_section_element::create(scope, "TFOOT"),
        "tr" => super::html_table_row_element::create(scope),
        "table" => super::html_table_element::create(scope),
        "col" => super::html_table_col_element::create(scope, "COL"),
        "colgroup" => super::html_table_col_element::create(scope, "COLGROUP"),
        "td" => super::html_table_cell_element::create(scope, "TD"),
        "th" => super::html_table_cell_element::create(scope, "TH"),
        "caption" => super::html_table_caption_element::create(scope),
        "style" => super::html_style_element::create(scope),
        "span" => super::html_span_element::create(scope),
        "source" => super::html_source_element::create(scope),
        "slot" => super::html_slot_element::create(scope),
        "selectedcontent" => super::html_selected_content_element::create(scope),
        "select" => super::html_select_element::create(scope),
        "script" => super::html_script_element::create(scope),
        "q" => super::html_quote_element::create(scope, "Q"),
        "blockquote" => super::html_quote_element::create(scope, "BLOCKQUOTE"),
        "progress" => super::html_progress_element::create(scope),
        "pre" => super::html_pre_element::create(scope),
        "picture" => super::html_picture_element::create(scope),
        "param" => super::html_param_element::create(scope),
        "p" => super::html_paragraph_element::create(scope),
        "output" => super::html_output_element::create(scope),
        "optgroup" => super::html_opt_group_element::create(scope),
        "object" => super::html_object_element::create(scope),
        "ol" => super::html_o_list_element::create(scope),
        "ins" => super::html_mod_element::create(scope, "INS"),
        "del" => super::html_mod_element::create(scope, "DEL"),
        "meter" => super::html_meter_element::create(scope),
        "meta" => super::html_meta_element::create(scope),
        "menu" => super::html_menu_element::create(scope),
        "marquee" => super::html_marquee_element::create(scope),
        "map" => super::html_map_element::create(scope),
        "link" => super::html_link_element::create(scope),
        "legend" => super::html_legend_element::create(scope),
        "label" => super::html_label_element::create(scope),
        "li" => super::html_li_element::create(scope),
        "input" => super::html_input_element::create(scope),
        "iframe" => super::html_i_frame_element::create(scope),
        "html" => super::html_html_element::create(scope),
        "h1" => super::html_heading_element::create(scope, 1),
        "h2" => super::html_heading_element::create(scope, 2),
        "h3" => super::html_heading_element::create(scope, 3),
        "h4" => super::html_heading_element::create(scope, 4),
        "h5" => super::html_heading_element::create(scope, 5),
        "h6" => super::html_heading_element::create(scope, 6),
        "head" => super::html_head_element::create(scope),
        "hr" => super::html_hr_element::create(scope),
        "frameset" => super::html_frame_set_element::create(scope),
        "frame" => super::html_frame_element::create(scope),
        "form" => super::html_form_element::create(scope),
        "font" => super::html_font_element::create(scope),
        "fieldset" => super::html_field_set_element::create(scope),
        "embed" => super::html_embed_element::create(scope),
        "div" => super::html_div_element::create(scope),
        "dir" => super::html_directory_element::create(scope),
        "dialog" => super::html_dialog_element::create(scope),
        "details" => super::html_details_element::create(scope),
        "datalist" => super::html_data_list_element::create(scope),
        "data" => super::html_data_element::create(scope),
        "dl" => super::html_d_list_element::create(scope),
        "canvas" => super::html_canvas_element::create(scope),
        "button" => super::html_button_element::create(scope),
        "body" => super::html_body_element::create(scope),
        "base" => super::html_base_element::create(scope),
        "br" => super::html_br_element::create(scope),
        "area" => super::html_area_element::create(scope),
        "a" => super::html_anchor_element::create(scope),
        "audio" => super::html_audio_element::create(scope, String::new()),
        "img" => super::html_image_element::create(scope, 0, 0),
        "option" => {
            super::html_option_element::create(scope, String::new(), String::new(), false, false)
        }
        "listing" => super::html_pre_element::create_with_tag(scope, "LISTING"),
        "xmp" => super::html_pre_element::create_with_tag(scope, "XMP"),
        "abbr" | "acronym" | "address" | "article" | "aside" | "b" | "basefont" | "bdi" | "bdo"
        | "big" | "center" | "cite" | "code" | "dd" | "dfn" | "dt" | "em" | "figcaption"
        | "figure" | "footer" | "header" | "hgroup" | "i" | "kbd" | "main" | "mark" | "nav"
        | "nobr" | "noembed" | "noframes" | "noscript" | "plaintext" | "rb" | "rp" | "rt"
        | "rtc" | "ruby" | "s" | "samp" | "search" | "section" | "small" | "strike" | "strong"
        | "sub" | "summary" | "sup" | "tt" | "u" | "var" | "wbr" => {
            super::html_element::create(scope, &normalized)
        }
        _ if normalized.contains('-') => super::html_element::create(scope, normalized),
        _ => super::html_unknown_element::create(scope, normalized),
    }
}

pub(crate) fn valid_xml_name(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || matches!(first, '_' | ':')) {
        return false;
    }
    characters.all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '_' | ':' | '-' | '.')
    })
}

pub(crate) fn validate_qualified_name<'a>(
    namespace: Option<&str>,
    qualified_name: &'a str,
    attribute: bool,
) -> Result<(), (&'static str, &'a str)> {
    if !valid_xml_name(qualified_name)
        || qualified_name.matches(':').count() > 1
        || qualified_name.starts_with(':')
        || qualified_name.ends_with(':')
    {
        return Err(("InvalidCharacterError", "The qualified name is not valid"));
    }
    let prefix = qualified_name.split_once(':').map(|(prefix, _)| prefix);
    if prefix.is_some() && namespace.is_none_or(str::is_empty) {
        return Err((
            "NamespaceError",
            "A prefix cannot be used with a null namespace",
        ));
    }
    if prefix == Some("xml") && namespace != Some("http://www.w3.org/XML/1998/namespace") {
        return Err((
            "NamespaceError",
            "The xml prefix requires the XML namespace",
        ));
    }
    let is_xmlns = qualified_name == "xmlns" || prefix == Some("xmlns");
    if attribute && is_xmlns && namespace != Some("http://www.w3.org/2000/xmlns/") {
        return Err((
            "NamespaceError",
            "The xmlns name requires the XMLNS namespace",
        ));
    }
    Ok(())
}
