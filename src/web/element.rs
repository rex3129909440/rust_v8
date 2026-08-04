use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct ElementStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, ElementRecord>,
}

#[derive(Clone)]
pub(crate) struct ElementRecord {
    pub tag_name: String,
    pub namespace_uri: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub attribute_namespaces: HashMap<String, Option<String>>,
    reflected: HashMap<String, v8::Global<v8::Value>>,
    pub handlers: HashMap<String, v8::Global<v8::Value>>,
    pub class_list: Option<v8::Global<v8::Object>>,
    pub part_list: Option<v8::Global<v8::Object>>,
    children_collection: Option<v8::Global<v8::Object>>,
    pub(crate) shadow_root: Option<v8::Global<v8::Object>>,
    pub scroll_top: f64,
    pub scroll_left: f64,
    pointer_captures: HashSet<i32>,
}

#[derive(Clone)]
pub(crate) struct AttributeSnapshot {
    pub name: String,
    pub value: String,
    pub namespace_uri: Option<String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Element", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ElementStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Element",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::element_namespace_uri_property::define(scope, prototype)?;
    super::element_prefix_property::define(scope, prototype)?;
    super::element_local_name_property::define(scope, prototype)?;
    super::element_tag_name_property::define(scope, prototype)?;
    super::element_id_property::define(scope, prototype)?;
    super::element_class_name_property::define(scope, prototype)?;
    super::element_class_list_property::define(scope, prototype)?;
    super::element_slot_property::define(scope, prototype)?;
    super::element_attributes_property::define(scope, prototype)?;
    super::element_shadow_root_property::define(scope, prototype)?;
    super::element_part_property::define(scope, prototype)?;
    super::element_assigned_slot::define(scope, prototype)?;
    super::element_inner_html::define(scope, prototype)?;
    super::element_outer_html::define(scope, prototype)?;
    super::element_scroll_top_property::define(scope, prototype)?;
    super::element_scroll_left_property::define(scope, prototype)?;
    super::element_scroll_width::define(scope, prototype)?;
    super::element_scroll_height::define(scope, prototype)?;
    super::element_client_top::define(scope, prototype)?;
    super::element_client_left::define(scope, prototype)?;
    super::element_client_width::define(scope, prototype)?;
    super::element_client_height::define(scope, prototype)?;
    super::element_on_before_copy::define(scope, prototype)?;
    super::element_on_before_cut::define(scope, prototype)?;
    super::element_on_before_paste::define(scope, prototype)?;
    super::element_on_search::define(scope, prototype)?;
    super::element_element_timing::define(scope, prototype)?;
    super::element_on_fullscreen_change::define(scope, prototype)?;
    super::element_on_fullscreen_error::define(scope, prototype)?;
    super::element_on_webkit_fullscreen_change::define(scope, prototype)?;
    super::element_on_webkit_fullscreen_error::define(scope, prototype)?;
    super::element_role::define(scope, prototype)?;
    super::element_aria_atomic::define(scope, prototype)?;
    super::element_aria_auto_complete::define(scope, prototype)?;
    super::element_aria_busy::define(scope, prototype)?;
    super::element_aria_braille_label::define(scope, prototype)?;
    super::element_aria_braille_role_description::define(scope, prototype)?;
    super::element_aria_checked::define(scope, prototype)?;
    super::element_aria_col_count::define(scope, prototype)?;
    super::element_aria_col_index::define(scope, prototype)?;
    super::element_aria_col_span::define(scope, prototype)?;
    super::element_aria_current::define(scope, prototype)?;
    super::element_aria_description::define(scope, prototype)?;
    super::element_aria_disabled::define(scope, prototype)?;
    super::element_aria_expanded::define(scope, prototype)?;
    super::element_aria_has_popup::define(scope, prototype)?;
    super::element_aria_hidden::define(scope, prototype)?;
    super::element_aria_invalid::define(scope, prototype)?;
    super::element_aria_key_shortcuts::define(scope, prototype)?;
    super::element_aria_label::define(scope, prototype)?;
    super::element_aria_level::define(scope, prototype)?;
    super::element_aria_live::define(scope, prototype)?;
    super::element_aria_modal::define(scope, prototype)?;
    super::element_aria_multi_line::define(scope, prototype)?;
    super::element_aria_multi_selectable::define(scope, prototype)?;
    super::element_aria_orientation::define(scope, prototype)?;
    super::element_aria_placeholder::define(scope, prototype)?;
    super::element_aria_pos_in_set::define(scope, prototype)?;
    super::element_aria_pressed::define(scope, prototype)?;
    super::element_aria_read_only::define(scope, prototype)?;
    super::element_aria_relevant::define(scope, prototype)?;
    super::element_aria_required::define(scope, prototype)?;
    super::element_aria_role_description::define(scope, prototype)?;
    super::element_aria_row_count::define(scope, prototype)?;
    super::element_aria_row_index::define(scope, prototype)?;
    super::element_aria_row_span::define(scope, prototype)?;
    super::element_aria_selected::define(scope, prototype)?;
    super::element_aria_set_size::define(scope, prototype)?;
    super::element_aria_sort::define(scope, prototype)?;
    super::element_aria_value_max::define(scope, prototype)?;
    super::element_aria_value_min::define(scope, prototype)?;
    super::element_aria_value_now::define(scope, prototype)?;
    super::element_aria_value_text::define(scope, prototype)?;
    super::element_children::define(scope, prototype)?;
    super::element_first_element_child::define(scope, prototype)?;
    super::element_last_element_child::define(scope, prototype)?;
    super::element_child_element_count::define(scope, prototype)?;
    super::element_previous_element_sibling::define(scope, prototype)?;
    super::element_next_element_sibling::define(scope, prototype)?;
    super::element_after::define(scope, prototype)?;
    super::element_animate::define(scope, prototype)?;
    super::parent_node_append::define(scope, prototype)?;
    super::element_attach_shadow::define(scope, prototype)?;
    super::element_before::define(scope, prototype)?;
    super::element_check_visibility::define(scope, prototype)?;
    super::element_closest::define(scope, prototype)?;
    super::element_computed_style_map::define(scope, prototype)?;
    super::element_get_animations::define(scope, prototype)?;
    super::element_get_attribute::define(scope, prototype)?;
    super::element_get_attribute_ns::define(scope, prototype)?;
    super::element_get_attribute_names::define(scope, prototype)?;
    super::element_get_attribute_node::define(scope, prototype)?;
    super::element_get_attribute_node_ns::define(scope, prototype)?;
    super::element_get_bounding_client_rect::define(scope, prototype)?;
    super::element_get_client_rects::define(scope, prototype)?;
    super::element_get_elements_by_class_name::define(scope, prototype)?;
    super::element_get_elements_by_tag_name::define(scope, prototype)?;
    super::element_get_elements_by_tag_name_ns::define(scope, prototype)?;
    super::element_get_html::define(scope, prototype)?;
    super::element_has_attribute::define(scope, prototype)?;
    super::element_has_attribute_ns::define(scope, prototype)?;
    super::element_has_attributes::define(scope, prototype)?;
    super::element_has_pointer_capture::define(scope, prototype)?;
    super::element_insert_adjacent_element::define(scope, prototype)?;
    super::element_insert_adjacent_html::define(scope, prototype)?;
    super::element_insert_adjacent_text::define(scope, prototype)?;
    super::element_matches::define(scope, prototype)?;
    super::parent_node_move_before::define(scope, prototype)?;
    super::parent_node_prepend::define(scope, prototype)?;
    super::element_query_selector::define(scope, prototype)?;
    super::element_query_selector_all::define(scope, prototype)?;
    super::element_release_pointer_capture::define(scope, prototype)?;
    super::element_remove::define(scope, prototype)?;
    super::element_remove_attribute::define(scope, prototype)?;
    super::element_remove_attribute_ns::define(scope, prototype)?;
    super::element_remove_attribute_node::define(scope, prototype)?;
    super::parent_node_replace_children::define(scope, prototype)?;
    super::element_replace_with::define(scope, prototype)?;
    super::element_request_fullscreen::define(scope, prototype)?;
    super::element_request_pointer_lock::define(scope, prototype)?;
    super::element_scroll::define(scope, prototype)?;
    super::element_scroll_by::define(scope, prototype)?;
    super::element_scroll_into_view::define(scope, prototype)?;
    super::element_scroll_into_view_if_needed::define(scope, prototype)?;
    super::element_scroll_to::define(scope, prototype)?;
    super::element_set_attribute::define(scope, prototype)?;
    super::element_set_attribute_ns::define(scope, prototype)?;
    super::element_set_attribute_node::define(scope, prototype)?;
    super::element_set_attribute_node_ns::define(scope, prototype)?;
    super::element_set_html_unsafe::define(scope, prototype)?;
    super::element_set_pointer_capture::define(scope, prototype)?;
    super::element_toggle_attribute::define(scope, prototype)?;
    super::element_webkit_matches_selector::define(scope, prototype)?;
    super::element_webkit_request_full_screen::define(scope, prototype)?;
    super::element_webkit_request_fullscreen::define(scope, prototype)?;
    super::element_current_css_zoom::define(scope, prototype)?;
    super::element_custom_element_registry::define(scope, prototype)?;
    super::element_active_view_transition::define(scope, prototype)?;
    super::element_aria_col_index_text::define(scope, prototype)?;
    super::element_aria_row_index_text::define(scope, prototype)?;
    super::element_aria_active_descendant_element::define(scope, prototype)?;
    super::element_aria_controls_elements::define(scope, prototype)?;
    super::element_aria_described_by_elements::define(scope, prototype)?;
    super::element_aria_details_elements::define(scope, prototype)?;
    super::element_aria_error_message_elements::define(scope, prototype)?;
    super::element_aria_flow_to_elements::define(scope, prototype)?;
    super::element_aria_labelled_by_elements::define(scope, prototype)?;
    super::element_aria_notify::define(scope, prototype)?;
    super::element_pseudo::define(scope, prototype)?;
    super::element_set_html::define(scope, prototype)?;
    super::element_start_view_transition::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let unscopables = crate::webidl::new_unscopables(scope)?;
    crate::webidl::define_unscopable(scope, unscopables, "after")?;
    crate::webidl::define_unscopable(scope, unscopables, "append")?;
    crate::webidl::define_unscopable(scope, unscopables, "before")?;
    crate::webidl::define_unscopable(scope, unscopables, "prepend")?;
    crate::webidl::define_unscopable(scope, unscopables, "remove")?;
    crate::webidl::define_unscopable(scope, unscopables, "replaceChildren")?;
    crate::webidl::define_unscopable(scope, unscopables, "replaceWith")?;
    crate::webidl::define_unscopable(scope, unscopables, "slot")?;
    crate::webidl::attach_unscopables(scope, prototype, unscopables)?;
    let parent = super::node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ElementStore>()
        .ok_or_else(|| "Element state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    tag_name: String,
    namespace_uri: Option<String>,
) {
    super::node::attach(scope, object, 1, tag_name.clone(), None);
    if let Some(store) = scope.get_slot_mut::<ElementStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            ElementRecord {
                tag_name,
                namespace_uri,
                attributes: Vec::new(),
                attribute_namespaces: HashMap::new(),
                reflected: HashMap::new(),
                handlers: HashMap::new(),
                class_list: None,
                part_list: None,
                children_collection: None,
                shadow_root: None,
                scroll_top: 0.0,
                scroll_left: 0.0,
                pointer_captures: HashSet::new(),
            },
        );
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    tag_name: String,
    namespace_uri: Option<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Element".to_owned());
    }
    attach(scope, object, tag_name, namespace_uri);
    Ok(object)
}

pub(crate) fn clone_shallow<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let record = record(scope, source).ok_or_else(|| "Element record is unavailable".to_owned())?;
    let local_name = record
        .tag_name
        .rsplit(':')
        .next()
        .unwrap_or(&record.tag_name)
        .to_ascii_lowercase();
    let clone = match record.namespace_uri.as_deref() {
        Some("http://www.w3.org/1999/xhtml") => {
            super::document::create_html_element_by_name(scope, &local_name)?
        }
        Some("http://www.w3.org/2000/svg") => {
            super::document::create_svg_element(scope, &local_name)?
        }
        Some("http://www.w3.org/1998/Math/MathML") => {
            super::math_ml_element::create(scope, local_name)?
        }
        namespace => create(scope, record.tag_name.clone(), namespace.map(str::to_owned))?,
    };
    set_qualified_name(scope, clone, record.tag_name.clone());
    for attribute in attributes_snapshot(scope, source).unwrap_or_default() {
        set_attribute_full(
            scope,
            clone,
            attribute.name,
            attribute.value,
            attribute.namespace_uri,
        );
    }
    Ok(clone)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ElementRecord> {
    scope
        .get_slot::<ElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn cached_reflected_value(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Value>> {
    record(scope, element)?.reflected.get(name).cloned()
}

pub(crate) fn set_reflected_value(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<v8::Local<'_, v8::Value>>,
) -> bool {
    let value = value.map(|value| v8::Global::new(scope, value));
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    else {
        return false;
    };
    if let Some(value) = value {
        record.reflected.insert(name.to_owned(), value);
    } else {
        record.reflected.remove(name);
    }
    true
}

pub(crate) fn cached_children_collection(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    record(scope, element)?.children_collection
}

pub(crate) fn cache_children_collection(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    collection: v8::Local<'_, v8::Object>,
) -> bool {
    let collection = v8::Global::new(scope, collection);
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    else {
        return false;
    };
    record.children_collection = Some(collection);
    true
}

pub(crate) fn cache_class_list(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    list: v8::Local<'_, v8::Object>,
) -> bool {
    let list = v8::Global::new(scope, list);
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    else {
        return false;
    };
    record.class_list = Some(list);
    true
}

pub(crate) fn cache_part_list(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    list: v8::Local<'_, v8::Object>,
) -> bool {
    let list = v8::Global::new(scope, list);
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    else {
        return false;
    };
    record.part_list = Some(list);
    true
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> bool {
    let handler = value.is_object().then(|| v8::Global::new(scope, value));
    let present = handler.is_some();
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
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
        element,
        name.strip_prefix("on").unwrap_or(name),
        present,
    );
    true
}

pub(crate) fn set_scroll_position(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    left: f64,
    top: f64,
    relative: bool,
) -> bool {
    let Some(current) = record(scope, element) else {
        return false;
    };
    let metrics = super::element_layout::scroll_metrics(scope, element);
    let maximum_left = (metrics.scroll_width - metrics.client_width).max(0.0);
    let maximum_top = (metrics.scroll_height - metrics.client_height).max(0.0);
    let left = if left.is_finite() { left } else { 0.0 };
    let top = if top.is_finite() { top } else { 0.0 };
    let next_left = (if relative {
        current.scroll_left + left
    } else {
        left
    })
    .clamp(0.0, maximum_left);
    let next_top = (if relative {
        current.scroll_top + top
    } else {
        top
    })
    .clamp(0.0, maximum_top);
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    else {
        return false;
    };
    record.scroll_left = next_left;
    record.scroll_top = next_top;
    notify_scrolled_subtree(scope, element);
    true
}

fn notify_scrolled_subtree(scope: &mut v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) {
    for child in super::node::children(scope, element) {
        if super::element::record(scope, child).is_some() {
            super::intersection_observer::notify_target_change(scope, child);
            notify_scrolled_subtree(scope, child);
        }
    }
}

pub(crate) fn set_pointer_capture_state(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    pointer_id: i32,
    captured: bool,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    else {
        return false;
    };
    if captured {
        record.pointer_captures.insert(pointer_id);
    } else {
        record.pointer_captures.remove(&pointer_id);
    }
    true
}

pub(crate) fn has_pointer_capture_state(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    pointer_id: i32,
) -> Option<bool> {
    Some(
        record(scope, element)?
            .pointer_captures
            .contains(&pointer_id),
    )
}

pub(crate) fn set_shadow_root(
    scope: &mut v8::PinScope<'_, '_>,
    host: v8::Local<'_, v8::Object>,
    root: v8::Local<'_, v8::Object>,
) {
    let root = v8::Global::new(scope, root);
    if let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&host.get_identity_hash().get()))
    {
        record.shadow_root = Some(root);
    }
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

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Element': Illegal constructor");
}

fn attribute(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let record = record(scope, object)?;
    let html = record.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml");
    record
        .attributes
        .into_iter()
        .find(|(candidate, _)| {
            if html {
                candidate.eq_ignore_ascii_case(name)
            } else {
                candidate == name
            }
        })
        .map(|(_, value)| value)
}

pub(crate) fn attribute_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    attribute(scope, object, name)
}

pub(crate) fn reflected_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    record(scope, object)?;
    Some(attribute(scope, object, name).unwrap_or_default())
}

pub(crate) fn set_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: String,
) -> bool {
    set_attribute_value(scope, object, name.to_owned(), value)
}

pub(crate) fn reflected_boolean(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<bool> {
    record(scope, object)?;
    Some(attribute(scope, object, name).is_some())
}

pub(crate) fn set_reflected_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: bool,
) -> bool {
    if value {
        set_attribute_value(scope, object, name.to_owned(), String::new())
    } else {
        remove_attribute_value(scope, object, name);
        true
    }
}

pub(crate) fn resolved_url_attribute(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let value = attribute(scope, object, name)?;
    let base = element_base_url(scope, object);
    ::url::Url::parse(&value)
        .or_else(|_| ::url::Url::parse(&base).and_then(|base| base.join(&value)))
        .map(|url| url.as_str().to_owned())
        .ok()
        .or(Some(value))
}

pub(crate) fn element_base_url(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> String {
    if let Some(document) = super::node::owner_document(scope, object)
        && let Some(value) = super::document::stored_value(scope, document, "URL")
    {
        let value = v8::Local::new(scope, &value);
        return crate::webidl::value_to_string(scope, value);
    }
    crate::page_init::base_url(scope)
}

pub(crate) fn reflected_element<'s>(
    scope: &v8::PinScope<'s, '_>,
    owner: v8::Local<'s, v8::Object>,
    attribute: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    record(scope, owner)?;
    let id = attribute_value(scope, owner, attribute)?;
    if id.is_empty() {
        return None;
    }
    let candidates = if super::node::is_connected(scope, owner) {
        super::node::owner_document(scope, owner)
            .map(|document| super::document::document_descendants(scope, document))
            .unwrap_or_default()
    } else {
        let mut root = owner;
        while let Some(parent) = super::node::parent(scope, root) {
            root = parent;
        }
        let mut candidates = super::dom_selector::descendants(scope, root);
        candidates.insert(0, root);
        candidates
    };
    candidates
        .into_iter()
        .find(|candidate| attribute_value(scope, *candidate, "id").as_deref() == Some(id.as_str()))
}

pub(crate) fn set_qualified_name(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    qualified_name: String,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.tag_name = qualified_name.clone();
    super::node::set_stored_node_name(scope, object, qualified_name)
}

pub(crate) fn set_attribute_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: String,
    value: String,
) -> bool {
    set_attribute_full(scope, object, name, value, None)
}

pub(crate) fn remove_attribute_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    remove_attribute_full(scope, object, None, name)
}

pub(crate) fn attributes_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<AttributeSnapshot>> {
    let record = record(scope, object)?;
    Some(
        record
            .attributes
            .into_iter()
            .map(|(name, value)| AttributeSnapshot {
                namespace_uri: record.attribute_namespaces.get(&name).cloned().flatten(),
                name,
                value,
            })
            .collect(),
    )
}

pub(crate) fn set_attribute_from_attr(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: String,
    value: String,
    namespace_uri: Option<String>,
) -> bool {
    set_attribute_full(scope, object, name, value, namespace_uri)
}

pub(crate) fn set_attribute_full(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: String,
    value: String,
    namespace_uri: Option<String>,
) -> bool {
    let dataset_name = name.clone();
    let dataset_value = value.clone();
    let old_value = {
        let Some(record) = scope
            .get_slot_mut::<ElementStore>()
            .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
        else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return false;
        };
        let local_name = name.rsplit(':').next().unwrap_or(&name);
        let matched_name = record.attributes.iter().find_map(|(candidate, _)| {
            let candidate_namespace = record
                .attribute_namespaces
                .get(candidate)
                .cloned()
                .flatten();
            let candidate_local = candidate.rsplit(':').next().unwrap_or(candidate);
            (candidate_namespace == namespace_uri && candidate_local == local_name)
                .then(|| candidate.clone())
        });
        let old_value = if let Some(matched_name) = matched_name {
            let Some((stored_name, current)) = record
                .attributes
                .iter_mut()
                .find(|(candidate, _)| *candidate == matched_name)
            else {
                return false;
            };
            let old_value = Some(current.clone());
            record.attribute_namespaces.remove(stored_name);
            *stored_name = name.clone();
            *current = value;
            old_value
        } else {
            record.attributes.push((name.clone(), value));
            None
        };
        record
            .attribute_namespaces
            .insert(name.clone(), namespace_uri.clone());
        old_value
    };
    super::html_element::sync_dataset(scope, object, &dataset_name, Some(&dataset_value));
    super::html_element::sync_style_attribute(scope, object, &dataset_name, Some(&dataset_value));
    super::dom_token_list::sync_binding_for_attribute(scope, object, &dataset_name, &dataset_value);
    super::named_node_map::sync_existing(scope, object);
    super::mutation_observer::enqueue_attribute_change(
        scope,
        object,
        name,
        namespace_uri,
        old_value,
    );
    if dataset_name.eq_ignore_ascii_case("slot") || dataset_name.eq_ignore_ascii_case("name") {
        super::html_slot_element::notify_assignment_change(scope, object);
    }
    super::html_i_frame_element::attribute_changed(
        scope,
        object,
        &dataset_name,
        Some(&dataset_value),
    );
    super::html_input_element::attribute_changed(
        scope,
        object,
        &dataset_name,
        Some(&dataset_value),
    );
    super::html_form_element::attribute_changed(scope, object, &dataset_name, Some(&dataset_value));
    super::html_select_element::attribute_changed(
        scope,
        object,
        &dataset_name,
        Some(&dataset_value),
    );
    super::html_text_area_element::attribute_changed(
        scope,
        object,
        &dataset_name,
        Some(&dataset_value),
    );
    super::html_option_element::attribute_changed(
        scope,
        object,
        &dataset_name,
        Some(&dataset_value),
    );
    super::html_image_element::attribute_changed(
        scope,
        object,
        &dataset_name,
        Some(&dataset_value),
    );
    super::html_media_element::attribute_changed(
        scope,
        object,
        &dataset_name,
        Some(&dataset_value),
    );
    super::html_anchor_element::attribute_changed(scope, object, &dataset_name);
    super::html_button_element::attribute_changed(scope, object, &dataset_name);
    super::html_area_element::attribute_changed(scope, object, &dataset_name);
    super::svg_a_element::attribute_changed(scope, object, &dataset_name);
    super::html_all_collection::refresh_all(scope);
    super::resize_observer::notify_target_change(scope, object);
    super::intersection_observer::notify_target_change(scope, object);
    true
}

pub(crate) fn remove_attribute_full(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    namespace_uri: Option<&str>,
    requested_name: &str,
) -> bool {
    let snapshot = attributes_snapshot(scope, object);
    let Some(snapshot) = snapshot else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    let html = record(scope, object).is_some_and(|record| {
        record.namespace_uri.as_deref() == Some("http://www.w3.org/1999/xhtml")
    });
    let matched = snapshot.into_iter().find(|attribute| {
        let local_name = attribute.name.rsplit(':').next().unwrap_or(&attribute.name);
        let name_matches = if namespace_uri.is_some() {
            local_name == requested_name
        } else if html {
            attribute.name.eq_ignore_ascii_case(requested_name)
        } else {
            attribute.name == requested_name
        };
        name_matches && namespace_uri == attribute.namespace_uri.as_deref()
    });
    let Some(matched) = matched else {
        return false;
    };
    if let Some(record) = scope
        .get_slot_mut::<ElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.attributes.retain(|(name, _)| name != &matched.name);
        record.attribute_namespaces.remove(&matched.name);
    }
    super::html_element::sync_dataset(scope, object, &matched.name, None);
    super::html_element::sync_style_attribute(scope, object, &matched.name, None);
    super::dom_token_list::sync_binding_for_attribute(scope, object, &matched.name, "");
    super::named_node_map::sync_existing(scope, object);
    let matched_name = matched.name.clone();
    super::mutation_observer::enqueue_attribute_change(
        scope,
        object,
        matched.name,
        matched.namespace_uri,
        Some(matched.value),
    );
    if matched_name.eq_ignore_ascii_case("slot") || matched_name.eq_ignore_ascii_case("name") {
        super::html_slot_element::notify_assignment_change(scope, object);
    }
    super::html_i_frame_element::attribute_changed(scope, object, &matched_name, None);
    super::html_input_element::attribute_changed(scope, object, &matched_name, None);
    super::html_form_element::attribute_changed(scope, object, &matched_name, None);
    super::html_select_element::attribute_changed(scope, object, &matched_name, None);
    super::html_text_area_element::attribute_changed(scope, object, &matched_name, None);
    super::html_option_element::attribute_changed(scope, object, &matched_name, None);
    super::html_image_element::attribute_changed(scope, object, &matched_name, None);
    super::html_media_element::attribute_changed(scope, object, &matched_name, None);
    super::html_anchor_element::attribute_changed(scope, object, &matched_name);
    super::html_button_element::attribute_changed(scope, object, &matched_name);
    super::html_area_element::attribute_changed(scope, object, &matched_name);
    super::svg_a_element::attribute_changed(scope, object, &matched_name);
    super::html_all_collection::refresh_all(scope);
    super::resize_observer::notify_target_change(scope, object);
    super::intersection_observer::notify_target_change(scope, object);
    true
}

pub(crate) fn update_document_focus(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    focused: bool,
) {
    let Some(document) = super::node::record(scope, object)
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document))
    else {
        return;
    };
    if focused {
        super::document::remember_value(scope, document, "activeElement", object.into());
        return;
    }
    let was_active = super::document::stored_value(scope, document, "activeElement")
        .is_some_and(|active| v8::Local::new(scope, &active).strict_equals(object.into()));
    if was_active {
        super::document::forget_value(scope, document, "activeElement");
    }
}

pub(crate) fn call_attribute_map_method(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    method_name: &str,
    values: &[v8::Local<'_, v8::Value>],
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, element).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(map) = super::named_node_map::create_for_element(scope, element) else {
        crate::webidl::throw_type_error(scope, "Cannot access Element attributes");
        return;
    };
    let Some(key) = v8::String::new(scope, method_name) else {
        return;
    };
    let Some(method) = map
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    if let Some(value) = method.call(scope, map.into(), values) {
        result.set(value);
    }
}

pub(crate) fn throw_selector_error(scope: &mut v8::PinScope<'_, '_>, message: String) {
    match super::dom_exception::create(scope, message, "SyntaxError".to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
