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
    for child in super::node::children(scope, common) {
        collect(scope, child, record, &mut rects);
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
) {
    let relation = relation(scope, node, record);
    if !relation.intersects {
        return;
    }
    if relation.contained && super::element::record(scope, node).is_some() {
        let layout = super::element_layout::compute(scope, node);
        if layout.rendered {
            rects.push(layout.rect());
        }
        return;
    }
    for child in super::node::children(scope, node) {
        collect(scope, child, record, rects);
    }
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
