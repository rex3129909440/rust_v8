#[derive(Clone, Copy)]
struct Relation {
    intersects: bool,
    contained: bool,
}

pub(crate) fn clone_contents<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    range: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let record = super::abstract_range::record(scope, range)
        .ok_or_else(|| "Illegal invocation".to_owned())?;
    build_fragment(scope, &record)
}

pub(crate) fn extract_contents<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    range: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let record = super::abstract_range::record(scope, range)
        .ok_or_else(|| "Illegal invocation".to_owned())?;
    let fragment = build_fragment(scope, &record)?;
    delete_record_contents(scope, range, &record);
    Ok(fragment)
}

pub(crate) fn delete_contents(
    scope: &mut v8::PinScope<'_, '_>,
    range: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let record = super::abstract_range::record(scope, range)
        .ok_or_else(|| "Illegal invocation".to_owned())?;
    delete_record_contents(scope, range, &record);
    Ok(())
}

pub(crate) fn has_partially_contained_non_text(
    scope: &v8::PinScope<'_, '_>,
    range: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(record) = super::abstract_range::record(scope, range) else {
        return false;
    };
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    let Some(common) = common_ancestor(scope, start, end) else {
        return false;
    };
    partially_contained_non_text_in(scope, common, &record)
}

fn build_fragment<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: &super::abstract_range::RangeRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let fragment = super::document_fragment::create(scope)?;
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    if start.strict_equals(end.into()) && record.start_offset == record.end_offset {
        set_fragment_owner(scope, fragment, record);
        return Ok(fragment);
    }
    if start.strict_equals(end.into())
        && super::character_data::data_if_character(scope, start).is_some()
    {
        let clone = clone_character_slice(scope, start, record.start_offset, record.end_offset)?;
        super::node::insert_node(scope, fragment, clone, 0)
            .map_err(|(_, message)| message.to_owned())?;
        set_fragment_owner(scope, fragment, record);
        return Ok(fragment);
    }
    let common = common_ancestor(scope, start, end)
        .ok_or_else(|| "The range boundary points have different roots".to_owned())?;
    for child in super::node::children(scope, common) {
        if let Some(clone) = clone_selected_node(scope, child, record)? {
            let index = super::node::children(scope, fragment).len();
            super::node::insert_node(scope, fragment, clone, index)
                .map_err(|(_, message)| message.to_owned())?;
        }
    }
    set_fragment_owner(scope, fragment, record);
    Ok(fragment)
}

fn set_fragment_owner(
    scope: &mut v8::PinScope<'_, '_>,
    fragment: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
) {
    let start = v8::Local::new(scope, &record.start_container);
    if let Some(document) = owner_document(scope, start) {
        super::node::set_owner_document_recursive(scope, fragment, document);
    }
}

fn clone_selected_node<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    node: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
) -> Result<Option<v8::Local<'s, v8::Object>>, String> {
    let relation = relation_to_range(scope, node, record);
    if !relation.intersects {
        return Ok(None);
    }
    if relation.contained {
        return super::node::clone_object(scope, node, true).map(Some);
    }
    if super::character_data::data_if_character(scope, node).is_some() {
        let length = character_length(scope, node);
        let start = v8::Local::new(scope, &record.start_container);
        let end = v8::Local::new(scope, &record.end_container);
        let low = if node.strict_equals(start.into()) {
            record.start_offset
        } else {
            0
        };
        let high = if node.strict_equals(end.into()) {
            record.end_offset
        } else {
            length
        };
        return clone_character_slice(scope, node, low, high).map(Some);
    }
    let clone = super::node::clone_object(scope, node, false)?;
    for child in super::node::children(scope, node) {
        if let Some(child_clone) = clone_selected_node(scope, child, record)? {
            let index = super::node::children(scope, clone).len();
            super::node::insert_node(scope, clone, child_clone, index)
                .map_err(|(_, message)| message.to_owned())?;
        }
    }
    Ok(Some(clone))
}

fn clone_character_slice<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
    start: u32,
    end: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let data = super::character_data::data_if_character(scope, source).unwrap_or_default();
    let units: Vec<u16> = data.encode_utf16().collect();
    let low = (start as usize).min(units.len());
    let high = (end as usize).min(units.len()).max(low);
    let clone = super::node::clone_object(scope, source, false)?;
    let _ = super::character_data::set_data_if_character(
        scope,
        clone,
        String::from_utf16_lossy(&units[low..high]),
    );
    Ok(clone)
}

fn delete_record_contents(
    scope: &mut v8::PinScope<'_, '_>,
    range: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
) {
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    if start.strict_equals(end.into()) {
        if super::character_data::data_if_character(scope, start).is_some() {
            delete_character_slice(scope, start, record.start_offset, record.end_offset);
        } else {
            let children = super::node::children(scope, start);
            let low = (record.start_offset as usize).min(children.len());
            let high = (record.end_offset as usize).min(children.len()).max(low);
            for child in &children[low..high] {
                super::node::detach(scope, *child);
            }
        }
        collapse_to_start(scope, range, record);
        return;
    }
    let Some(common) = common_ancestor(scope, start, end) else {
        return;
    };
    delete_selected_descendants(scope, common, record);
    collapse_to_start(scope, range, record);
}

fn delete_selected_descendants(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
) {
    for child in super::node::children(scope, parent) {
        let relation = relation_to_range(scope, child, record);
        if relation.contained {
            super::node::detach(scope, child);
            continue;
        }
        if !relation.intersects {
            continue;
        }
        if super::character_data::data_if_character(scope, child).is_some() {
            let start = v8::Local::new(scope, &record.start_container);
            let end = v8::Local::new(scope, &record.end_container);
            let low = if child.strict_equals(start.into()) {
                record.start_offset
            } else {
                0
            };
            let high = if child.strict_equals(end.into()) {
                record.end_offset
            } else {
                character_length(scope, child)
            };
            delete_character_slice(scope, child, low, high);
        } else {
            delete_selected_descendants(scope, child, record);
        }
    }
}

fn delete_character_slice(
    scope: &mut v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    start: u32,
    end: u32,
) {
    let data = super::character_data::data_if_character(scope, node).unwrap_or_default();
    let units: Vec<u16> = data.encode_utf16().collect();
    let low = (start as usize).min(units.len());
    let high = (end as usize).min(units.len()).max(low);
    let mut remaining = units[..low].to_vec();
    remaining.extend_from_slice(&units[high..]);
    let _ = super::character_data::set_data_if_character(
        scope,
        node,
        String::from_utf16_lossy(&remaining),
    );
}

fn collapse_to_start(
    scope: &mut v8::PinScope<'_, '_>,
    range: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
) {
    let start = record.start_container.clone();
    let offset = record.start_offset;
    super::abstract_range::update(scope, range, |value| {
        value.start_container = start.clone();
        value.end_container = start;
        value.start_offset = offset;
        value.end_offset = offset;
    });
}

fn relation_to_range(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
) -> Relation {
    let Some(parent) = super::node::parent(scope, node) else {
        return Relation {
            intersects: false,
            contained: false,
        };
    };
    let Some(index) = super::node::children(scope, parent)
        .iter()
        .position(|candidate| candidate.strict_equals(node.into()))
    else {
        return Relation {
            intersects: false,
            contained: false,
        };
    };
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    let node_start_to_end =
        super::range::compare_boundaries(scope, parent, index as u32, end, record.end_offset);
    let node_end_to_start = super::range::compare_boundaries(
        scope,
        parent,
        index as u32 + 1,
        start,
        record.start_offset,
    );
    let intersects = node_start_to_end.is_some_and(|order| order < 0)
        && node_end_to_start.is_some_and(|order| order > 0);
    let start_to_node_start =
        super::range::compare_boundaries(scope, start, record.start_offset, parent, index as u32);
    let node_end_to_end =
        super::range::compare_boundaries(scope, parent, index as u32 + 1, end, record.end_offset);
    Relation {
        intersects,
        contained: intersects
            && start_to_node_start.is_some_and(|order| order <= 0)
            && node_end_to_end.is_some_and(|order| order <= 0),
    }
}

fn partially_contained_non_text_in(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
) -> bool {
    for child in super::node::children(scope, parent) {
        let relation = relation_to_range(scope, child, record);
        if relation.intersects && !relation.contained {
            let is_text =
                super::node::record(scope, child).is_some_and(|value| value.node_type == 3);
            if !is_text && super::character_data::data_if_character(scope, child).is_none() {
                return true;
            }
            if partially_contained_non_text_in(scope, child, record) {
                return true;
            }
        }
    }
    false
}

fn character_length(scope: &v8::PinScope<'_, '_>, node: v8::Local<'_, v8::Object>) -> u32 {
    super::character_data::data_if_character(scope, node)
        .map(|data| data.encode_utf16().count() as u32)
        .unwrap_or(0)
}

fn common_ancestor<'s>(
    scope: &v8::PinScope<'s, '_>,
    start: v8::Local<'s, v8::Object>,
    end: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut current = Some(start);
    while let Some(candidate) = current {
        let mut end_cursor = Some(end);
        while let Some(end_candidate) = end_cursor {
            if candidate.strict_equals(end_candidate.into()) {
                return Some(candidate);
            }
            end_cursor = super::node::parent(scope, end_candidate);
        }
        current = super::node::parent(scope, candidate);
    }
    None
}

fn owner_document<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    if super::document::is_document(scope, node) {
        Some(node)
    } else {
        super::node::record(scope, node)
            .and_then(|record| record.owner_document)
            .map(|document| v8::Local::new(scope, &document))
    }
}
