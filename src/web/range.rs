pub(crate) const START_TO_START: i32 = 0;
pub(crate) const START_TO_END: i32 = 1;
pub(crate) const END_TO_END: i32 = 2;
pub(crate) const END_TO_START: i32 = 3;

#[derive(Default)]
pub(crate) struct RangeStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RangeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Range", constructor.into())
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let range = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, range, prototype.into()) != Some(true) {
        return Err("cannot create Range".to_owned());
    }
    super::abstract_range::attach_live(scope, range, document, 0, document, 0);
    Ok(range)
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RangeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Range",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::range_common_ancestor_container_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    super::range_clone_contents::define(scope, prototype)?;
    super::range_clone_range::define(scope, prototype)?;
    super::range_collapse::define(scope, prototype)?;
    super::range_compare_boundary_points::define(scope, prototype)?;
    super::range_compare_point::define(scope, prototype)?;
    super::range_create_contextual_fragment::define(scope, prototype)?;
    super::range_delete_contents::define(scope, prototype)?;
    super::range_detach::define(scope, prototype)?;
    super::range_expand::define(scope, prototype)?;
    super::range_extract_contents::define(scope, prototype)?;
    super::range_get_bounding_client_rect::define(scope, prototype)?;
    super::range_get_client_rects::define(scope, prototype)?;
    super::range_insert_node::define(scope, prototype)?;
    super::range_intersects_node::define(scope, prototype)?;
    super::range_is_point_in_range::define(scope, prototype)?;
    super::range_select_node::define(scope, prototype)?;
    super::range_select_node_contents::define(scope, prototype)?;
    super::range_set_end::define(scope, prototype)?;
    super::range_set_end_after::define(scope, prototype)?;
    super::range_set_end_before::define(scope, prototype)?;
    super::range_set_start::define(scope, prototype)?;
    super::range_set_start_after::define(scope, prototype)?;
    super::range_set_start_before::define(scope, prototype)?;
    super::range_surround_contents::define(scope, prototype)?;
    super::range_to_string::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::abstract_range::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RangeStore>()
        .ok_or_else(|| "Range state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "START_TO_START", START_TO_START)?;
    crate::webidl::define_constant(scope, object, "START_TO_END", START_TO_END)?;
    crate::webidl::define_constant(scope, object, "END_TO_END", END_TO_END)?;
    crate::webidl::define_constant(scope, object, "END_TO_START", END_TO_START)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Range': use new");
        return;
    }
    let Some(document) = super::document_global::value(scope) else {
        crate::webidl::throw_type_error(scope, "The current Document is unavailable");
        return;
    };
    super::abstract_range::attach_live(scope, arguments.this(), document, 0, document, 0);
    result.set(arguments.this().into());
}

pub(crate) fn create_from_record<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: super::abstract_range::RangeRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Range".to_owned());
    }
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    super::abstract_range::attach_live(
        scope,
        object,
        start,
        record.start_offset,
        end,
        record.end_offset,
    );
    Ok(object)
}

pub(crate) fn record_or_throw(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<super::abstract_range::RangeRecord> {
    let record = super::abstract_range::record(scope, object);
    if record.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    record
}

pub(crate) fn set_relative(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
    start: bool,
    after: bool,
) {
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    let Some(parent) = super::node::parent(scope, node) else {
        super::node::throw_dom_exception(scope, "InvalidNodeTypeError", "The node has no parent");
        return;
    };
    let children = super::node::children(scope, parent);
    let Some(index) = children
        .iter()
        .position(|child| child.strict_equals(node.into()))
    else {
        return;
    };
    let offset = (index + usize::from(after)) as u32;
    let parent = v8::Global::new(scope, parent);
    if !super::abstract_range::update(scope, arguments.this(), |range| {
        if start {
            range.start_container = parent;
            range.start_offset = offset;
        } else {
            range.end_container = parent;
            range.end_offset = offset;
        }
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn boundary_arguments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<(v8::Global<v8::Object>, u32)> {
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The first argument is not a Node");
        return None;
    };
    let offset = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let Some(length) = boundary_length(scope, node) else {
        crate::webidl::throw_type_error(scope, "The first argument is not a Node");
        return None;
    };
    if super::node::record(scope, node).is_some_and(|record| record.node_type == 10) {
        super::node::throw_dom_exception(
            scope,
            "InvalidNodeTypeError",
            "DocumentType nodes cannot be range boundary containers",
        );
        return None;
    }
    if offset > length {
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            "The offset is larger than the node's length",
        );
        return None;
    }
    Some((v8::Global::new(scope, node), offset))
}

pub(crate) fn boundary_length(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
) -> Option<u32> {
    super::node::record(scope, node)?;
    if let Some(text) = super::character_data::data_if_character(scope, node) {
        Some(text.encode_utf16().count() as u32)
    } else {
        Some(super::node::children(scope, node).len() as u32)
    }
}

pub(crate) fn common_ancestor<'s>(
    scope: &v8::PinScope<'s, '_>,
    start: v8::Local<'s, v8::Object>,
    end: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut ancestors = Vec::new();
    let mut current = Some(start);
    while let Some(node) = current {
        ancestors.push(node);
        current = super::node::parent(scope, node);
    }
    let mut current = Some(end);
    while let Some(node) = current {
        if let Some(common) = ancestors
            .iter()
            .find(|ancestor| ancestor.strict_equals(node.into()))
        {
            return Some(*common);
        }
        current = super::node::parent(scope, node);
    }
    None
}

pub(crate) fn root<'s>(
    scope: &v8::PinScope<'s, '_>,
    mut node: v8::Local<'s, v8::Object>,
) -> v8::Local<'s, v8::Object> {
    while let Some(parent) = super::node::parent(scope, node) {
        node = parent;
    }
    node
}

pub(crate) fn text_length(scope: &v8::PinScope<'_, '_>, node: v8::Local<'_, v8::Object>) -> usize {
    if let Some(text) = super::character_data::data_if_character(scope, node) {
        text.encode_utf16().count()
    } else {
        super::node::children(scope, node)
            .into_iter()
            .map(|child| text_length(scope, child))
            .sum()
    }
}

pub(crate) fn boundary_index(
    scope: &v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
    container: v8::Local<'_, v8::Object>,
    offset: u32,
) -> Option<usize> {
    fn walk(
        scope: &v8::PinScope<'_, '_>,
        node: v8::Local<'_, v8::Object>,
        container: v8::Local<'_, v8::Object>,
        offset: u32,
        cursor: &mut usize,
    ) -> Option<usize> {
        if node.strict_equals(container.into()) {
            if super::character_data::data_if_character(scope, node).is_some() {
                return Some(*cursor + offset as usize);
            }
            let children = super::node::children(scope, node);
            let count = (offset as usize).min(children.len());
            let prefix: usize = children[..count]
                .iter()
                .map(|child| text_length(scope, *child))
                .sum();
            return Some(*cursor + prefix);
        }
        if let Some(text) = super::character_data::data_if_character(scope, node) {
            *cursor += text.encode_utf16().count();
            return None;
        }
        for child in super::node::children(scope, node) {
            if let Some(found) = walk(scope, child, container, offset, cursor) {
                return Some(found);
            }
        }
        None
    }
    let mut cursor = 0;
    walk(scope, root, container, offset, &mut cursor)
}

pub(crate) fn compare_boundaries(
    scope: &v8::PinScope<'_, '_>,
    a_node: v8::Local<'_, v8::Object>,
    a_offset: u32,
    b_node: v8::Local<'_, v8::Object>,
    b_offset: u32,
) -> Option<i32> {
    let root_a = root(scope, a_node);
    let root_b = root(scope, b_node);
    if !root_a.strict_equals(root_b.into()) {
        return None;
    }
    if a_node.strict_equals(b_node.into()) {
        return Some(a_offset.cmp(&b_offset) as i32);
    }
    if is_ancestor(scope, a_node, b_node) {
        let child = direct_child_under(scope, a_node, b_node)?;
        let index = super::node::children(scope, a_node)
            .iter()
            .position(|candidate| candidate.strict_equals(child.into()))?
            as u32;
        return Some(if a_offset <= index { -1 } else { 1 });
    }
    if is_ancestor(scope, b_node, a_node) {
        let child = direct_child_under(scope, b_node, a_node)?;
        let index = super::node::children(scope, b_node)
            .iter()
            .position(|candidate| candidate.strict_equals(child.into()))?
            as u32;
        return Some(if index < b_offset { -1 } else { 1 });
    }
    let common = common_ancestor(scope, a_node, b_node)?;
    let a_child = direct_child_under(scope, common, a_node)?;
    let b_child = direct_child_under(scope, common, b_node)?;
    let children = super::node::children(scope, common);
    let a_index = children
        .iter()
        .position(|candidate| candidate.strict_equals(a_child.into()))?;
    let b_index = children
        .iter()
        .position(|candidate| candidate.strict_equals(b_child.into()))?;
    Some(if a_index < b_index { -1 } else { 1 })
}

fn is_ancestor<'s>(
    scope: &v8::PinScope<'s, '_>,
    ancestor: v8::Local<'s, v8::Object>,
    mut node: v8::Local<'s, v8::Object>,
) -> bool {
    while let Some(parent) = super::node::parent(scope, node) {
        if parent.strict_equals(ancestor.into()) {
            return true;
        }
        node = parent;
    }
    false
}

fn direct_child_under<'s>(
    scope: &v8::PinScope<'s, '_>,
    ancestor: v8::Local<'s, v8::Object>,
    mut node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    while let Some(parent) = super::node::parent(scope, node) {
        if parent.strict_equals(ancestor.into()) {
            return Some(node);
        }
        node = parent;
    }
    None
}

pub(crate) fn selected_text(
    scope: &v8::PinScope<'_, '_>,
    range: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = super::abstract_range::record(scope, range)?;
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    let tree_root = root(scope, start);
    if !tree_root.strict_equals(root(scope, end).into()) {
        return Some(String::new());
    }
    let all = collect_text(scope, tree_root);
    let start_index = boundary_index(scope, tree_root, start, record.start_offset)?;
    let end_index = boundary_index(scope, tree_root, end, record.end_offset)?;
    let units: Vec<u16> = all.encode_utf16().collect();
    let low = start_index.min(end_index).min(units.len());
    let high = start_index.max(end_index).min(units.len());
    Some(String::from_utf16_lossy(&units[low..high]))
}

fn collect_text(scope: &v8::PinScope<'_, '_>, node: v8::Local<'_, v8::Object>) -> String {
    if let Some(text) = super::character_data::data_if_character(scope, node) {
        return text;
    }
    let mut output = String::new();
    for child in super::node::children(scope, node) {
        output.push_str(&collect_text(scope, child));
    }
    output
}

pub(crate) fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
    }
}
