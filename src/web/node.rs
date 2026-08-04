use std::collections::HashMap;

pub(crate) const ELEMENT_NODE: i32 = 1;
pub(crate) const ATTRIBUTE_NODE: i32 = 2;
pub(crate) const TEXT_NODE: i32 = 3;
pub(crate) const CDATA_SECTION_NODE: i32 = 4;
const ENTITY_REFERENCE_NODE: i32 = 5;
const ENTITY_NODE: i32 = 6;
pub(crate) const PROCESSING_INSTRUCTION_NODE: i32 = 7;
pub(crate) const COMMENT_NODE: i32 = 8;
pub(crate) const DOCUMENT_NODE: i32 = 9;
pub(crate) const DOCUMENT_TYPE_NODE: i32 = 10;
const DOCUMENT_FRAGMENT_NODE: i32 = 11;
const NOTATION_NODE: i32 = 12;
pub(crate) const DOCUMENT_POSITION_DISCONNECTED: i32 = 1;
pub(crate) const DOCUMENT_POSITION_PRECEDING: i32 = 2;
pub(crate) const DOCUMENT_POSITION_FOLLOWING: i32 = 4;
pub(crate) const DOCUMENT_POSITION_CONTAINS: i32 = 8;
pub(crate) const DOCUMENT_POSITION_CONTAINED_BY: i32 = 16;
pub(crate) const DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: i32 = 32;

#[derive(Default)]
pub(crate) struct NodeStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, NodeRecord>,
}

#[derive(Clone)]
pub(crate) struct NodeRecord {
    pub node_type: i32,
    pub node_name: String,
    pub node_value: Option<String>,
    pub parent: Option<v8::Global<v8::Object>>,
    pub children: Vec<v8::Global<v8::Object>>,
    pub owner_document: Option<v8::Global<v8::Object>>,
    pub child_nodes: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Node", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NodeStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Node",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::node_node_type_property::define(scope, prototype)?;
    super::node_node_name_property::define(scope, prototype)?;
    super::node_base_uri_property::define(scope, prototype)?;
    super::node_is_connected_property::define(scope, prototype)?;
    super::node_owner_document_property::define(scope, prototype)?;
    super::node_parent_node_property::define(scope, prototype)?;
    super::node_parent_element_property::define(scope, prototype)?;
    super::node_child_nodes_property::define(scope, prototype)?;
    super::node_first_child_property::define(scope, prototype)?;
    super::node_last_child_property::define(scope, prototype)?;
    super::node_previous_sibling_property::define(scope, prototype)?;
    super::node_next_sibling_property::define(scope, prototype)?;
    super::node_node_value_property::define(scope, prototype)?;
    super::node_text_content_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    super::node_append_child::define(scope, prototype)?;
    super::node_clone_node::define(scope, prototype)?;
    super::node_compare_document_position::define(scope, prototype)?;
    super::node_contains::define(scope, prototype)?;
    super::node_get_root_node::define(scope, prototype)?;
    super::node_has_child_nodes::define(scope, prototype)?;
    super::node_insert_before::define(scope, prototype)?;
    super::node_is_default_namespace::define(scope, prototype)?;
    super::node_is_equal_node::define(scope, prototype)?;
    super::node_is_same_node::define(scope, prototype)?;
    super::node_lookup_namespace_uri::define(scope, prototype)?;
    super::node_lookup_prefix::define(scope, prototype)?;
    super::node_normalize::define(scope, prototype)?;
    super::node_remove_child::define(scope, prototype)?;
    super::node_replace_child::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NodeStore>()
        .ok_or_else(|| "Node state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "ELEMENT_NODE", ELEMENT_NODE)?;
    crate::webidl::define_constant(scope, object, "ATTRIBUTE_NODE", ATTRIBUTE_NODE)?;
    crate::webidl::define_constant(scope, object, "TEXT_NODE", TEXT_NODE)?;
    crate::webidl::define_constant(scope, object, "CDATA_SECTION_NODE", CDATA_SECTION_NODE)?;
    crate::webidl::define_constant(
        scope,
        object,
        "ENTITY_REFERENCE_NODE",
        ENTITY_REFERENCE_NODE,
    )?;
    crate::webidl::define_constant(scope, object, "ENTITY_NODE", ENTITY_NODE)?;
    crate::webidl::define_constant(
        scope,
        object,
        "PROCESSING_INSTRUCTION_NODE",
        PROCESSING_INSTRUCTION_NODE,
    )?;
    crate::webidl::define_constant(scope, object, "COMMENT_NODE", COMMENT_NODE)?;
    crate::webidl::define_constant(scope, object, "DOCUMENT_NODE", DOCUMENT_NODE)?;
    crate::webidl::define_constant(scope, object, "DOCUMENT_TYPE_NODE", DOCUMENT_TYPE_NODE)?;
    crate::webidl::define_constant(
        scope,
        object,
        "DOCUMENT_FRAGMENT_NODE",
        DOCUMENT_FRAGMENT_NODE,
    )?;
    crate::webidl::define_constant(scope, object, "NOTATION_NODE", NOTATION_NODE)?;
    crate::webidl::define_constant(
        scope,
        object,
        "DOCUMENT_POSITION_DISCONNECTED",
        DOCUMENT_POSITION_DISCONNECTED,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "DOCUMENT_POSITION_PRECEDING",
        DOCUMENT_POSITION_PRECEDING,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "DOCUMENT_POSITION_FOLLOWING",
        DOCUMENT_POSITION_FOLLOWING,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "DOCUMENT_POSITION_CONTAINS",
        DOCUMENT_POSITION_CONTAINS,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "DOCUMENT_POSITION_CONTAINED_BY",
        DOCUMENT_POSITION_CONTAINED_BY,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC",
        DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC,
    )
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    node_type: i32,
    node_name: String,
    node_value: Option<String>,
) {
    super::event_target::attach(scope, object);
    if let Some(store) = scope.get_slot_mut::<NodeStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            NodeRecord {
                node_type,
                node_name,
                node_value,
                parent: None,
                children: Vec::new(),
                owner_document: None,
                child_nodes: None,
            },
        );
    }
}

pub(crate) fn set_stored_node_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: Option<String>,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<NodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.node_value = value;
    true
}

pub(crate) fn set_stored_node_name(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: String,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<NodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.node_name = value;
    true
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NodeRecord> {
    scope
        .get_slot::<NodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn cache_child_nodes(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    list: v8::Local<'_, v8::Object>,
) {
    let list = v8::Global::new(scope, list);
    if let Some(record) = scope
        .get_slot_mut::<NodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.child_nodes = Some(list);
    }
}

pub(crate) fn set_owner_document(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    document: v8::Local<'_, v8::Object>,
) -> bool {
    let stored_document = v8::Global::new(scope, document);
    let updated = if let Some(record) = scope
        .get_slot_mut::<NodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.owner_document = Some(stored_document);
        true
    } else {
        false
    };
    if updated {
        super::html_template_element::update_owner_document(scope, object, document);
    }
    updated
}

pub(crate) fn set_owner_document_recursive(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    document: v8::Local<'_, v8::Object>,
) -> bool {
    if !set_owner_document(scope, object, document) {
        return false;
    }
    for child in children(scope, object) {
        set_owner_document_recursive(scope, child, document);
    }
    true
}

pub(crate) fn parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    record(scope, object)?
        .parent
        .map(|value| v8::Local::new(scope, &value))
}

pub(crate) fn owner_document<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    record(scope, object)?
        .owner_document
        .map(|document| v8::Local::new(scope, &document))
}

pub(crate) fn children<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    record(scope, object)
        .map(|record| {
            record
                .children
                .iter()
                .map(|value| v8::Local::new(scope, value))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn is_connected(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(mut current_record) = record(scope, object) else {
        return false;
    };
    let mut node = object;
    let mut connected = current_record.node_type == DOCUMENT_NODE;
    loop {
        let next = current_record
            .parent
            .as_ref()
            .map(|parent| v8::Local::new(scope, parent))
            .or_else(|| super::shadow_root::host(scope, node));
        let Some(next) = next else {
            break;
        };
        let Some(next_record) = record(scope, next) else {
            break;
        };
        connected |= next_record.node_type == DOCUMENT_NODE;
        node = next;
        current_record = next_record;
    }
    connected
}

pub(crate) fn text_content(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> String {
    node_text(scope, object)
}

pub(crate) fn insert_child(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    child: v8::Local<'_, v8::Object>,
    index: usize,
) -> bool {
    insert_node(scope, parent, child, index).is_ok()
}

pub(crate) fn insert_node(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    child: v8::Local<'_, v8::Object>,
    index: usize,
) -> Result<(), (&'static str, &'static str)> {
    if record(scope, parent).is_none() || record(scope, child).is_none() {
        return Err(("TypeError", "The parent and child must be Nodes"));
    }
    let parent_record = record(scope, parent).expect("parent Node record");
    let child_record = record(scope, child).expect("child Node record");
    if !matches!(
        parent_record.node_type,
        ELEMENT_NODE | DOCUMENT_NODE | DOCUMENT_FRAGMENT_NODE
    ) || matches!(child_record.node_type, ATTRIBUTE_NODE | DOCUMENT_NODE)
    {
        return Err((
            "HierarchyRequestError",
            "The operation would yield an incorrect node tree",
        ));
    }
    if is_descendant(scope, child, parent) {
        return Err((
            "HierarchyRequestError",
            "The new child is an ancestor of the parent",
        ));
    }
    if child_record.node_type == DOCUMENT_FRAGMENT_NODE {
        let fragment_children = children(scope, child);
        if parent_record.node_type == DOCUMENT_NODE {
            let element_count = fragment_children
                .iter()
                .filter(|node| {
                    record(scope, **node).is_some_and(|node| node.node_type == ELEMENT_NODE)
                })
                .count();
            let has_text = fragment_children
                .iter()
                .any(|node| record(scope, *node).is_some_and(|node| node.node_type == TEXT_NODE));
            let existing_elements = parent_record
                .children
                .iter()
                .filter(|node| {
                    record(scope, v8::Local::new(scope, *node))
                        .is_some_and(|node| node.node_type == ELEMENT_NODE)
                })
                .count();
            if has_text || element_count > 1 || existing_elements + element_count > 1 {
                return Err((
                    "HierarchyRequestError",
                    "The Document cannot accept the fragment",
                ));
            }
        }
        let mut insertion = index;
        for fragment_child in fragment_children {
            insert_node(scope, parent, fragment_child, insertion)?;
            insertion += 1;
        }
        return Ok(());
    }
    if parent_record.node_type == DOCUMENT_NODE {
        if child_record.node_type == TEXT_NODE {
            return Err((
                "HierarchyRequestError",
                "Text nodes cannot be children of a Document",
            ));
        }
        if child_record.node_type == ELEMENT_NODE {
            let other_element = parent_record.children.iter().any(|node| {
                let node = v8::Local::new(scope, node);
                node.get_identity_hash().get() != child.get_identity_hash().get()
                    && record(scope, node).is_some_and(|node| node.node_type == ELEMENT_NODE)
            });
            if other_element {
                return Err((
                    "HierarchyRequestError",
                    "Only one document element is allowed",
                ));
            }
        }
        if child_record.node_type == DOCUMENT_TYPE_NODE {
            let other_doctype = parent_record.children.iter().any(|node| {
                let node = v8::Local::new(scope, node);
                node.get_identity_hash().get() != child.get_identity_hash().get()
                    && record(scope, node).is_some_and(|node| node.node_type == DOCUMENT_TYPE_NODE)
            });
            if other_doctype {
                return Err(("HierarchyRequestError", "Only one document type is allowed"));
            }
        }
    }
    let old_parent = record(scope, child)
        .and_then(|record| record.parent)
        .map(|parent| v8::Local::new(scope, &parent));
    let old_index = old_parent.and_then(|old_parent| {
        (old_parent.get_identity_hash().get() == parent.get_identity_hash().get()).then(|| {
            children(scope, parent)
                .iter()
                .position(|node| node.get_identity_hash().get() == child.get_identity_hash().get())
        })?
    });
    let adjusted_index = old_index
        .filter(|old_index| *old_index < index)
        .map_or(index, |index| index.saturating_sub(1));
    if parent_record.node_type == DOCUMENT_NODE {
        let child_identity = child.get_identity_hash().get();
        let prospective = children(scope, parent)
            .into_iter()
            .filter(|node| node.get_identity_hash().get() != child_identity)
            .collect::<Vec<_>>();
        let position = adjusted_index.min(prospective.len());
        if child_record.node_type == DOCUMENT_TYPE_NODE
            && prospective[..position]
                .iter()
                .any(|node| record(scope, *node).is_some_and(|node| node.node_type == ELEMENT_NODE))
        {
            return Err((
                "HierarchyRequestError",
                "The DocumentType must precede the document element",
            ));
        }
        if child_record.node_type == ELEMENT_NODE
            && prospective[position..].iter().any(|node| {
                record(scope, *node).is_some_and(|node| node.node_type == DOCUMENT_TYPE_NODE)
            })
        {
            return Err((
                "HierarchyRequestError",
                "The document element must follow the DocumentType",
            ));
        }
    }
    detach(scope, child);
    let siblings = children(scope, parent);
    let position = adjusted_index.min(siblings.len());
    let previous_sibling = position
        .checked_sub(1)
        .and_then(|index| siblings.get(index).copied());
    let next_sibling = siblings.get(position).copied();
    let parent_global = v8::Global::new(scope, parent);
    let child_global = v8::Global::new(scope, child);
    let insertion_document = if parent_record.node_type == DOCUMENT_NODE {
        Some(v8::Global::new(scope, parent))
    } else {
        parent_record.owner_document.clone()
    };
    if let Some(child_record) = scope
        .get_slot_mut::<NodeStore>()
        .and_then(|store| store.records.get_mut(&child.get_identity_hash().get()))
    {
        child_record.parent = Some(parent_global);
    }
    let inserted = if let Some(parent_record) = scope
        .get_slot_mut::<NodeStore>()
        .and_then(|store| store.records.get_mut(&parent.get_identity_hash().get()))
    {
        parent_record.children.insert(position, child_global);
        true
    } else {
        false
    };
    if !inserted {
        return Err(("TypeError", "The insertion failed"));
    }
    super::abstract_range::adjust_for_insertion(scope, parent, position, 1);
    super::mutation_observer::enqueue_child_list(
        scope,
        parent,
        vec![child],
        Vec::new(),
        previous_sibling,
        next_sibling,
    );
    super::html_slot_element::notify_assignment_change(scope, parent);
    if let Some(document) = insertion_document {
        let document = v8::Local::new(scope, &document);
        set_owner_document_recursive(scope, child, document);
    }
    super::html_all_collection::refresh_all(scope);
    super::html_i_frame_element::notify_connected_tree(scope, child);
    super::html_style_element::notify_connected_tree(scope, child);
    super::html_style_element::notify_tree_mutation(scope, parent);
    super::html_link_element::notify_connected_tree(scope, child);
    super::document_style_sheets_property::refresh_for_node(scope, parent);
    super::html_script_element::notify_connected_tree(scope, child);
    super::resize_observer::notify_target_change(scope, child);
    super::intersection_observer::notify_target_change(scope, child);
    Ok(())
}

pub(crate) fn detach(scope: &mut v8::PinScope<'_, '_>, child: v8::Local<'_, v8::Object>) -> bool {
    let old_parent = parent(scope, child);
    if let Some(parent) = old_parent {
        let siblings = children(scope, parent);
        let index = siblings.iter().position(|candidate| {
            candidate.get_identity_hash().get() == child.get_identity_hash().get()
        });
        if let Some(index) = index {
            super::abstract_range::adjust_for_removal(scope, parent, child, index);
        }
        let previous_sibling = index
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| siblings.get(index).copied());
        let next_sibling = index.and_then(|index| siblings.get(index + 1).copied());
        super::node_iterator::adjust_for_removal(scope, child);
        let identity = child.get_identity_hash().get();
        let retained = record(scope, parent)
            .map(|record| {
                record
                    .children
                    .into_iter()
                    .filter(|value| {
                        v8::Local::new(scope, value).get_identity_hash().get() != identity
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if let Some(record) = scope
            .get_slot_mut::<NodeStore>()
            .and_then(|store| store.records.get_mut(&parent.get_identity_hash().get()))
        {
            record.children = retained;
        }
        super::mutation_observer::enqueue_child_list(
            scope,
            parent,
            Vec::new(),
            vec![child],
            previous_sibling,
            next_sibling,
        );
        super::html_slot_element::notify_assignment_change(scope, parent);
    }
    let detached = if let Some(record) = scope
        .get_slot_mut::<NodeStore>()
        .and_then(|store| store.records.get_mut(&child.get_identity_hash().get()))
    {
        record.parent = None;
        true
    } else {
        false
    };
    super::html_all_collection::refresh_all(scope);
    super::html_i_frame_element::notify_disconnected_tree(scope, child);
    super::html_style_element::notify_disconnected_tree(scope, child);
    super::html_link_element::notify_disconnected_tree(scope, child);
    if let Some(parent) = old_parent {
        super::html_style_element::notify_tree_mutation(scope, parent);
        super::document_style_sheets_property::refresh_for_node(scope, parent);
    } else {
        super::document_style_sheets_property::refresh_for_node(scope, child);
    }
    super::resize_observer::notify_target_change(scope, child);
    super::intersection_observer::notify_target_change(scope, child);
    detached
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Node': Illegal constructor");
}

pub(crate) fn node_text(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> String {
    let Some(v) = record(scope, object) else {
        return String::new();
    };
    if matches!(
        v.node_type,
        TEXT_NODE | CDATA_SECTION_NODE | PROCESSING_INSTRUCTION_NODE | COMMENT_NODE
    ) && let Some(data) = super::character_data::data_if_character(scope, object)
    {
        return data;
    }
    if let Some(value) = v.node_value {
        return value;
    }
    v.children
        .iter()
        .filter_map(|child| {
            let child = v8::Local::new(scope, child);
            let record = record(scope, child)?;
            (!matches!(record.node_type, PROCESSING_INSTRUCTION_NODE | COMMENT_NODE))
                .then(|| node_text(scope, child))
        })
        .collect()
}
pub(crate) fn clone_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
    deep: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let record = record(scope, source).ok_or_else(|| "The value is not a Node".to_owned())?;
    let clone = match record.node_type {
        ELEMENT_NODE => super::element::clone_shallow(scope, source)?,
        ATTRIBUTE_NODE => {
            let attribute = super::attr::record(scope, source)
                .ok_or_else(|| "Attr state missing".to_owned())?;
            super::attr::create(
                scope,
                attribute.name,
                attribute.value,
                attribute.namespace_uri,
                None,
            )?
        }
        TEXT_NODE => super::text::create(
            scope,
            super::character_data::data_if_character(scope, source).unwrap_or_default(),
        )?,
        CDATA_SECTION_NODE => super::cdata_section::create(
            scope,
            super::character_data::data_if_character(scope, source).unwrap_or_default(),
        )?,
        PROCESSING_INSTRUCTION_NODE => super::processing_instruction::create(
            scope,
            record.node_name.clone(),
            super::character_data::data_if_character(scope, source).unwrap_or_default(),
        )?,
        COMMENT_NODE => super::comment::create(
            scope,
            super::character_data::data_if_character(scope, source).unwrap_or_default(),
        )?,
        DOCUMENT_FRAGMENT_NODE => super::document_fragment::create(scope)?,
        _ => {
            let constructor = ensure_constructor(scope)?;
            let prototype = crate::webidl::prototype(scope, constructor)?;
            let object = v8::Object::new(scope);
            if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true)
            {
                return Err("Cannot clone Node".to_owned());
            }
            attach(
                scope,
                object,
                record.node_type,
                record.node_name.clone(),
                record.node_value.clone(),
            );
            object
        }
    };
    if let Some(document) = record
        .owner_document
        .as_ref()
        .map(|document| v8::Local::new(scope, document))
    {
        set_owner_document(scope, clone, document);
    }
    if deep {
        let source_children = effective_children(scope, source);
        let clone_parent = super::html_template_element::record(scope, clone)
            .map(|record| v8::Local::new(scope, &record.content))
            .unwrap_or(clone);
        for child in source_children {
            let child_clone = clone_object(scope, child, true)?;
            let index = children(scope, clone_parent).len();
            insert_node(scope, clone_parent, child_clone, index)
                .map_err(|(_, message)| message.to_owned())?;
        }
    }
    Ok(clone)
}

fn effective_children<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::html_template_element::record(scope, node)
        .map(|record| children(scope, v8::Local::new(scope, &record.content)))
        .unwrap_or_else(|| children(scope, node))
}
pub(crate) fn is_descendant<'s>(
    scope: &v8::PinScope<'s, '_>,
    ancestor: v8::Local<'s, v8::Object>,
    mut node: v8::Local<'s, v8::Object>,
) -> bool {
    let id = ancestor.get_identity_hash().get();
    loop {
        if node.get_identity_hash().get() == id {
            return true;
        }
        let Some(next) = parent(scope, node) else {
            return false;
        };
        node = next;
    }
}
fn namespace_context<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let node_record = record(scope, node)?;
    if node_record.node_type == ELEMENT_NODE {
        return Some(node);
    }
    if node_record.node_type == DOCUMENT_NODE {
        return children(scope, node).into_iter().find(|child| {
            record(scope, *child).is_some_and(|child| child.node_type == ELEMENT_NODE)
        });
    }
    if node_record.node_type == ATTRIBUTE_NODE {
        return super::attr::record(scope, node)?
            .owner_element
            .map(|owner| v8::Local::new(scope, &owner));
    }
    let mut ancestor = parent(scope, node);
    while let Some(candidate) = ancestor {
        if record(scope, candidate).is_some_and(|candidate| candidate.node_type == ELEMENT_NODE) {
            return Some(candidate);
        }
        ancestor = parent(scope, candidate);
    }
    None
}

pub(crate) fn locate_namespace_uri(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    prefix: Option<&str>,
) -> Option<String> {
    if prefix == Some("xml") {
        return Some("http://www.w3.org/XML/1998/namespace".to_owned());
    }
    if prefix == Some("xmlns") {
        return Some("http://www.w3.org/2000/xmlns/".to_owned());
    }
    let mut element = namespace_context(scope, node);
    while let Some(current) = element {
        let record = super::element::record(scope, current)?;
        let element_prefix = record.tag_name.split_once(':').map(|(prefix, _)| prefix);
        if element_prefix == prefix && record.namespace_uri.is_some() {
            return record.namespace_uri;
        }
        let declaration = match prefix {
            Some(prefix) => format!("xmlns:{prefix}"),
            None => "xmlns".to_owned(),
        };
        if let Some(value) = super::element::attribute_value(scope, current, &declaration) {
            return (!value.is_empty()).then_some(value);
        }
        element = parent(scope, current)
            .filter(|parent| super::element::record(scope, *parent).is_some());
    }
    None
}

pub(crate) fn locate_prefix(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    namespace: &str,
) -> Option<String> {
    if namespace == "http://www.w3.org/XML/1998/namespace" {
        return Some("xml".to_owned());
    }
    if namespace == "http://www.w3.org/2000/xmlns/" {
        return Some("xmlns".to_owned());
    }
    let mut element = namespace_context(scope, node);
    while let Some(current) = element {
        let record = super::element::record(scope, current)?;
        if record.namespace_uri.as_deref() == Some(namespace)
            && let Some((prefix, _)) = record.tag_name.split_once(':')
        {
            return Some(prefix.to_owned());
        }
        for attribute in super::element::attributes_snapshot(scope, current).unwrap_or_default() {
            if attribute.value == namespace
                && let Some(prefix) = attribute.name.strip_prefix("xmlns:")
            {
                return Some(prefix.to_owned());
            }
        }
        element = parent(scope, current)
            .filter(|parent| super::element::record(scope, *parent).is_some());
    }
    None
}
pub(crate) fn normalize_node(scope: &mut v8::PinScope<'_, '_>, node: v8::Local<'_, v8::Object>) {
    let values = children(scope, node);
    let mut previous: Option<v8::Local<v8::Object>> = None;
    for child in values {
        if let Some(data) = super::text::data_if_text(scope, child) {
            if data.is_empty() {
                detach(scope, child);
            } else if let Some(previous) = previous
                && let Some(existing) = super::text::data_if_text(scope, previous)
            {
                let existing_length = existing.encode_utf16().count() as u32;
                let _ = super::character_data::replace_data_units(
                    scope,
                    previous,
                    existing_length,
                    0,
                    &data,
                );
                super::abstract_range::adjust_for_text_merge(
                    scope,
                    child,
                    previous,
                    existing_length,
                );
                detach(scope, child);
            } else {
                previous = Some(child);
            }
        } else {
            previous = None;
            normalize_node(scope, child);
        }
    }
}

pub(crate) fn root_node<'s>(
    scope: &v8::PinScope<'s, '_>,
    mut node: v8::Local<'s, v8::Object>,
) -> v8::Local<'s, v8::Object> {
    while let Some(parent) = parent(scope, node) {
        node = parent;
    }
    node
}

pub(crate) fn tree_order<'s>(
    scope: &v8::PinScope<'s, '_>,
    root: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    fn visit<'s>(
        scope: &v8::PinScope<'s, '_>,
        node: v8::Local<'s, v8::Object>,
        output: &mut Vec<v8::Local<'s, v8::Object>>,
    ) {
        output.push(node);
        for child in children(scope, node) {
            visit(scope, child, output);
        }
    }
    let mut output = Vec::new();
    visit(scope, root, &mut output);
    output
}

pub(crate) fn equal_nodes(
    scope: &v8::PinScope<'_, '_>,
    left: v8::Local<'_, v8::Object>,
    right: v8::Local<'_, v8::Object>,
) -> bool {
    let (Some(left_record), Some(right_record)) = (record(scope, left), record(scope, right))
    else {
        return false;
    };
    if left_record.node_type != right_record.node_type
        || left_record.node_name != right_record.node_name
        || left_record.node_value != right_record.node_value
    {
        return false;
    }
    if left_record.node_type == ELEMENT_NODE {
        let left_attributes = super::element::attributes_snapshot(scope, left).unwrap_or_default();
        let right_attributes =
            super::element::attributes_snapshot(scope, right).unwrap_or_default();
        if left_attributes.len() != right_attributes.len()
            || left_attributes.iter().any(|left_attribute| {
                !right_attributes.iter().any(|right_attribute| {
                    left_attribute.name == right_attribute.name
                        && left_attribute.value == right_attribute.value
                        && left_attribute.namespace_uri == right_attribute.namespace_uri
                })
            })
        {
            return false;
        }
    }
    let left_template = super::html_template_element::record(scope, left).is_some();
    let right_template = super::html_template_element::record(scope, right).is_some();
    if left_template != right_template {
        return false;
    }
    let left_children = effective_children(scope, left);
    let right_children = effective_children(scope, right);
    left_children.len() == right_children.len()
        && left_children
            .iter()
            .zip(right_children.iter())
            .all(|(left, right)| equal_nodes(scope, *left, *right))
}
pub(crate) fn throw_dom_exception(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), name.to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(_) => crate::webidl::throw_type_error(scope, message),
    }
}
