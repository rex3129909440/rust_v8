pub(crate) fn client_rects(
    scope: &v8::PinScope<'_, '_>,
    record: &super::abstract_range::RangeRecord,
) -> Vec<super::dom_rect_read_only::RectRecord> {
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    if start.strict_equals(end.into()) && record.start_offset == record.end_offset {
        return Vec::new();
    }
    let Some(common) = super::range::common_ancestor(scope, start, end) else {
        return Vec::new();
    };
    let mut rects = Vec::new();
    if super::node::record(scope, common)
        .is_some_and(|node| node.node_type == super::node::TEXT_NODE)
    {
        append_text_rects(scope, common, record, &mut rects);
        return rects;
    }
    let common_is_ruby = super::element::record(scope, common)
        .is_some_and(|element| element.tag_name.eq_ignore_ascii_case("RUBY"));
    for child in super::node::children(scope, common) {
        collect(scope, child, record, &mut rects, common_is_ruby);
    }
    rects
}

pub(crate) fn bounding_rect(
    rects: &[super::dom_rect_read_only::RectRecord],
) -> super::dom_rect_read_only::RectRecord {
    let Some(first) = rects.first().copied() else {
        return super::dom_rect_read_only::RectRecord {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
    };
    let mut left = first.x;
    let mut top = first.y;
    let mut right = first.x + first.width;
    let mut bottom = first.y + first.height;
    for rect in &rects[1..] {
        left = left.min(rect.x);
        top = top.min(rect.y);
        right = right.max(rect.x + rect.width);
        bottom = bottom.max(rect.y + rect.height);
    }
    super::dom_rect_read_only::RectRecord {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

fn collect(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    record: &super::abstract_range::RangeRecord,
    rects: &mut Vec<super::dom_rect_read_only::RectRecord>,
    include_ruby_annotation_text: bool,
) {
    let relation = relation(scope, node, record);
    if !relation.intersects {
        return;
    }
    if super::node::record(scope, node)
        .is_some_and(|record| record.node_type == super::node::TEXT_NODE)
    {
        append_text_rects(scope, node, record, rects);
        return;
    }
    if relation.contained && super::element::record(scope, node).is_some() {
        let tag = super::element::record(scope, node)
            .map(|element| element.tag_name)
            .unwrap_or_default();
        if super::element_layout::is_block_level(scope, node) {
            let layout = super::element_layout::compute(scope, node);
            if layout.rendered {
                rects.push(layout.rect());
            }
        } else {
            append_inline_element_rects(scope, node, rects);
        }
        if tag.eq_ignore_ascii_case("RT") && !include_ruby_annotation_text {
            return;
        }
    }
    for child in super::node::children(scope, node) {
        collect(scope, child, record, rects, include_ruby_annotation_text);
    }
}

fn append_inline_element_rects(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    rects: &mut Vec<super::dom_rect_read_only::RectRecord>,
) {
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("RUBY") || tag.eq_ignore_ascii_case("RT") {
        rects.extend(super::inline_text_layout::inline_element_rects(
            scope, element,
        ));
        return;
    }
    let mut text_identities = Vec::new();
    collect_descendant_text_identities(scope, element, &mut text_identities);
    if text_identities.is_empty() {
        return;
    }
    let Some(container) = super::inline_text_layout::containing_inline_box(scope, element) else {
        return;
    };
    let container_layout = super::element_layout::compute(scope, container);
    if !container_layout.rendered || container_layout.content_width <= 0.0 {
        return;
    }
    let scroll = super::element::record(scope, container)
        .map(|record| (record.scroll_left, record.scroll_top))
        .unwrap_or_default();
    let layout = super::inline_text_layout::layout_for_element(
        scope,
        container,
        container_layout.content_width,
        container_layout.x + container_layout.border_left + container_layout.padding_left
            - scroll.0,
        container_layout.y + container_layout.border_top + container_layout.padding_top - scroll.1,
    );
    rects.extend(super::inline_text_layout::node_set_rects(
        &layout,
        &text_identities,
    ));
}

fn collect_descendant_text_identities(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    output: &mut Vec<i32>,
) {
    for child in super::node::children(scope, node) {
        if super::node::record(scope, child)
            .is_some_and(|record| record.node_type == super::node::TEXT_NODE)
        {
            output.push(child.get_identity_hash().get());
        } else {
            collect_descendant_text_identities(scope, child, output);
        }
    }
}

fn append_text_rects(
    scope: &v8::PinScope<'_, '_>,
    text_node: v8::Local<'_, v8::Object>,
    range: &super::abstract_range::RangeRecord,
    rects: &mut Vec<super::dom_rect_read_only::RectRecord>,
) {
    let Some(container) = super::inline_text_layout::containing_inline_box(scope, text_node) else {
        return;
    };
    let container_layout = super::element_layout::compute(scope, container);
    if !container_layout.rendered || container_layout.content_width <= 0.0 {
        return;
    }
    let scroll = super::element::record(scope, container)
        .map(|record| (record.scroll_left, record.scroll_top))
        .unwrap_or_default();
    let layout = super::inline_text_layout::layout_for_element(
        scope,
        container,
        container_layout.content_width,
        container_layout.x + container_layout.border_left + container_layout.padding_left
            - scroll.0,
        container_layout.y + container_layout.border_top + container_layout.padding_top - scroll.1,
    );
    let start_container = v8::Local::new(scope, &range.start_container);
    let end_container = v8::Local::new(scope, &range.end_container);
    let text_length = super::node::record(scope, text_node)
        .and_then(|record| record.node_value)
        .map(|value| value.encode_utf16().count() as u32)
        .unwrap_or(0);
    let start = if text_node.strict_equals(start_container.into()) {
        range.start_offset.min(text_length)
    } else {
        0
    };
    let end = if text_node.strict_equals(end_container.into()) {
        range.end_offset.min(text_length)
    } else {
        text_length
    };
    if start >= end {
        return;
    }
    rects.extend(super::inline_text_layout::selection_rects(
        &layout,
        text_node.get_identity_hash().get(),
        start,
        end,
    ));
}

#[derive(Clone, Copy)]
struct Relation {
    intersects: bool,
    contained: bool,
}

fn relation(
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
