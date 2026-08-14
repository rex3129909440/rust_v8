use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct LayoutSnapshot {
    rect: super::dom_rect_read_only::RectRecord,
}

#[derive(Default)]
struct RealmRenderingState {
    painted_elements: HashSet<i32>,
    largest_contentful_paint_element: Option<i32>,
    layout: HashMap<i32, LayoutSnapshot>,
}

#[derive(Default)]
pub(crate) struct RenderingPerformanceState {
    realms: HashMap<i32, RealmRenderingState>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RenderingPerformanceState::default());
}

pub(crate) fn update<'s>(scope: &mut v8::PinScope<'s, '_>) {
    if super::window_view_state::inner_width(scope) <= 0.0
        || super::window_view_state::inner_height(scope) <= 0.0
    {
        return;
    }
    let Some(document) = super::document_global::value(scope) else {
        return;
    };
    let realm_id = crate::webidl::realm_id(scope);
    let now = super::performance::now_for_current_realm(scope)
        .unwrap_or(0.0)
        .max(0.0);
    let elements = super::document::document_descendants(scope, document);
    emit_element_timing_entries(scope, realm_id, now, &elements);
    emit_largest_contentful_paint(scope, realm_id, now, &elements);
    emit_layout_shift(scope, realm_id, now, &elements);
}

fn emit_element_timing_entries<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    realm_id: i32,
    now: f64,
    elements: &[v8::Local<'s, v8::Object>],
) {
    let already_painted = scope
        .get_slot::<RenderingPerformanceState>()
        .and_then(|store| store.realms.get(&realm_id))
        .map(|state| state.painted_elements.clone())
        .unwrap_or_default();
    let mut newly_painted = Vec::new();
    // Chromium reports painted image records before the rendered-text set for
    // the same rendering update. Preserve DOM order within each record class.
    for image_pass in [true, false] {
        for element in elements {
            let identity = element.get_identity_hash().get();
            if already_painted.contains(&identity) {
                continue;
            }
            let Some(identifier) =
                super::element::attribute_value(scope, *element, "elementtiming")
            else {
                continue;
            };
            let layout = super::element_layout::compute(scope, *element);
            if !layout.rendered || layout.border_width() <= 0.0 || layout.border_height() <= 0.0 {
                continue;
            }
            let tag_name = super::element::record(scope, *element)
                .map(|record| record.tag_name)
                .unwrap_or_default();
            let image = tag_name.eq_ignore_ascii_case("IMG");
            if image != image_pass {
                continue;
            }
            if image
                && super::html_image_element::record(scope, *element).is_none_or(|record| {
                    record.request_state != super::html_image_element::ImageRequestState::Loaded
                })
            {
                continue;
            }
            let (url, natural_width, natural_height, load_time) = if image {
                super::html_image_element::record(scope, *element)
                    .map(|record| {
                        (
                            record.current_src,
                            record.natural_width as i32,
                            record.natural_height as i32,
                            record.load_time,
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            super::element::resolved_url_attribute(scope, *element, "src")
                                .unwrap_or_default(),
                            0,
                            0,
                            now,
                        )
                    })
            } else {
                (String::new(), 0, 0, 0.0)
            };
            let id = super::element::attribute_value(scope, *element, "id").unwrap_or_default();
            let name = if image { "image-paint" } else { "text-paint" };
            let paint_rect = if image {
                layout.rect()
            } else {
                text_paint_rect(scope, *element, layout).unwrap_or_else(|| layout.rect())
            };
            let intersection_rect = pixel_snapped_visible_rect(
                paint_rect,
                super::window_view_state::inner_width(scope),
                super::window_view_state::inner_height(scope),
            );
            if let Ok(entry) = super::performance_element_timing::create(
                scope,
                name.to_owned(),
                now,
                now,
                load_time,
                intersection_rect,
                identifier,
                natural_width,
                natural_height,
                id,
                Some((*element).into()),
                url,
                now,
                now,
            ) {
                super::performance_observer::queue_entry(scope, entry, "element");
                newly_painted.push(identity);
            }
        }
    }
    if let Some(store) = scope.get_slot_mut::<RenderingPerformanceState>() {
        let state = store.realms.entry(realm_id).or_default();
        state.painted_elements.extend(newly_painted);
    }
}

fn emit_largest_contentful_paint<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    realm_id: i32,
    now: f64,
    elements: &[v8::Local<'s, v8::Object>],
) {
    let mut largest: Option<(v8::Local<'_, v8::Object>, f64, bool)> = None;
    for element in elements {
        let Some(record) = super::element::record(scope, *element) else {
            continue;
        };
        if matches!(
            record.tag_name.as_str(),
            "HTML" | "BODY" | "HEAD" | "SCRIPT" | "STYLE" | "TEMPLATE"
        ) {
            continue;
        }
        let image = record.tag_name.eq_ignore_ascii_case("IMG");
        if image
            && super::html_image_element::record(scope, *element).is_none_or(|record| {
                record.request_state != super::html_image_element::ImageRequestState::Loaded
            })
        {
            continue;
        }
        if !image && !has_nonempty_text(scope, *element) {
            continue;
        }
        let layout = super::element_layout::compute(scope, *element);
        if !layout.rendered {
            continue;
        }
        let area = if image {
            visible_pixel_area(
                layout.rect(),
                super::window_view_state::inner_width(scope),
                super::window_view_state::inner_height(scope),
            )
        } else {
            text_visible_pixel_area(scope, *element, layout)
        };
        if area <= 0.0 || largest.is_some_and(|(_, current_area, _)| current_area >= area) {
            continue;
        }
        largest = Some((*element, area, image));
    }
    let Some((element, area, image)) = largest else {
        return;
    };
    let identity = element.get_identity_hash().get();
    if scope
        .get_slot::<RenderingPerformanceState>()
        .and_then(|store| store.realms.get(&realm_id))
        .and_then(|state| state.largest_contentful_paint_element)
        == Some(identity)
    {
        return;
    }
    let id = super::element::attribute_value(scope, element, "id").unwrap_or_default();
    let url = if image {
        super::html_image_element::record(scope, element)
            .map(|record| record.current_src)
            .filter(|url| !url.is_empty())
            .or_else(|| super::element::resolved_url_attribute(scope, element, "src"))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let load_time = if image {
        super::html_image_element::record(scope, element)
            .map(|record| record.load_time)
            .unwrap_or(0.0)
    } else {
        0.0
    };
    if let Ok(entry) = super::largest_contentful_paint::create(
        scope,
        now,
        now,
        load_time,
        area.round().max(0.0) as u64,
        id,
        url,
        Some(element),
        now,
        now,
    ) {
        super::performance_observer::queue_entry(scope, entry, "largest-contentful-paint");
        if let Some(store) = scope.get_slot_mut::<RenderingPerformanceState>() {
            store
                .realms
                .entry(realm_id)
                .or_default()
                .largest_contentful_paint_element = Some(identity);
        }
    }
}

fn text_visible_pixel_area(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    layout: super::element_layout::LayoutBox,
) -> f64 {
    text_paint_rect(scope, element, layout)
        .map(|rect| {
            visible_pixel_area(
                rect,
                super::window_view_state::inner_width(scope),
                super::window_view_state::inner_height(scope),
            )
        })
        .unwrap_or(0.0)
}

fn text_paint_rect(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    layout: super::element_layout::LayoutBox,
) -> Option<super::dom_rect_read_only::RectRecord> {
    let scroll = super::element::record(scope, element)
        .map(|record| (record.scroll_left, record.scroll_top))
        .unwrap_or_default();
    let inline = super::inline_text_layout::layout_for_element(
        scope,
        element,
        layout.content_width,
        layout.x + layout.border_left + layout.padding_left - scroll.0,
        layout.y + layout.border_top + layout.padding_top - scroll.1,
    );
    let Some(first) = inline.paint_rects.first().copied() else {
        return None;
    };
    let bounds = inline
        .paint_rects
        .iter()
        .skip(1)
        .fold(first, |bounds, rect| {
            let left = bounds.x.min(rect.x);
            let top = bounds.y.min(rect.y);
            let right = (bounds.x + bounds.width).max(rect.x + rect.width);
            let bottom = (bounds.y + bounds.height).max(rect.y + rect.height);
            super::dom_rect_read_only::RectRecord {
                x: left,
                y: top,
                width: (right - left).max(0.0),
                height: (bottom - top).max(0.0),
            }
        });
    Some(bounds)
}

fn visible_pixel_area(
    rect: super::dom_rect_read_only::RectRecord,
    viewport_width: f64,
    viewport_height: f64,
) -> f64 {
    let rect = pixel_snapped_visible_rect(rect, viewport_width, viewport_height);
    rect.width * rect.height
}

fn pixel_snapped_visible_rect(
    rect: super::dom_rect_read_only::RectRecord,
    viewport_width: f64,
    viewport_height: f64,
) -> super::dom_rect_read_only::RectRecord {
    let left = rect.x.max(0.0).floor();
    let top = rect.y.max(0.0).floor();
    let right = (rect.x + rect.width).min(viewport_width).ceil();
    let bottom = (rect.y + rect.height).min(viewport_height).ceil();
    super::dom_rect_read_only::RectRecord {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

fn has_nonempty_text<'s>(scope: &v8::PinScope<'s, '_>, node: v8::Local<'s, v8::Object>) -> bool {
    for child in super::node::children(scope, node) {
        if let Some(record) = super::node::record(scope, child) {
            if record.node_type == super::node::TEXT_NODE
                && record
                    .node_value
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            {
                return true;
            }
        }
        if has_nonempty_text(scope, child) {
            return true;
        }
    }
    false
}

fn emit_layout_shift<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    realm_id: i32,
    now: f64,
    elements: &[v8::Local<'s, v8::Object>],
) {
    let previous = scope
        .get_slot::<RenderingPerformanceState>()
        .and_then(|store| store.realms.get(&realm_id))
        .map(|state| state.layout.clone())
        .unwrap_or_default();
    let mut current = HashMap::new();
    let mut shifts = Vec::new();
    for element in elements {
        let layout = super::element_layout::compute(scope, *element);
        if !layout.rendered {
            continue;
        }
        let identity = element.get_identity_hash().get();
        let rect = visual_layout_rect(scope, *element, layout);
        current.insert(identity, LayoutSnapshot { rect });
        let Some(old) = previous.get(&identity) else {
            continue;
        };
        if (old.rect.x - rect.x).abs() < f64::EPSILON && (old.rect.y - rect.y).abs() < f64::EPSILON
        {
            continue;
        }
        shifts.push((*element, old.rect, rect));
    }
    if let Some(store) = scope.get_slot_mut::<RenderingPerformanceState>() {
        store.realms.entry(realm_id).or_default().layout = current;
    }
    if previous.is_empty() || shifts.is_empty() {
        return;
    }
    let viewport_width = super::window_view_state::inner_width(scope);
    let viewport_height = super::window_view_state::inner_height(scope);
    let viewport_area = viewport_width * viewport_height;
    if viewport_area <= 0.0 {
        return;
    }
    // The impact fraction is the union of the pixel-snapped previous and
    // current visual regions.  Summing per-element swept bounding boxes
    // double-counts overlaps and incorrectly fills the gap for large moves.
    let mut impact_rects = Vec::new();
    let mut maximum_distance: f64 = 0.0;
    for (_, old, current) in &shifts {
        impact_rects.push(layout_shift_pixel_rect(
            *old,
            viewport_width,
            viewport_height,
        ));
        impact_rects.push(layout_shift_pixel_rect(
            *current,
            viewport_width,
            viewport_height,
        ));
        maximum_distance = maximum_distance
            .max((old.x - current.x).abs())
            .max((old.y - current.y).abs());
    }
    let impact_area = rectangle_union_area(&impact_rects);
    shifts.sort_by(|(_, old_a, current_a), (_, old_b, current_b)| {
        movement_impact_area(*old_b, *current_b)
            .total_cmp(&movement_impact_area(*old_a, *current_a))
    });
    let mut sources = Vec::new();
    for (element, old, current) in shifts.into_iter().take(5) {
        let Ok(previous_rect) = super::dom_rect_read_only::create(scope, old) else {
            continue;
        };
        let Ok(current_rect) = super::dom_rect_read_only::create(scope, current) else {
            continue;
        };
        if let Ok(source) = super::layout_shift_attribution::create(
            scope,
            Some(element),
            previous_rect,
            current_rect,
        ) {
            sources.push(source);
        }
    }
    let impact_fraction = (impact_area / viewport_area).clamp(0.0, 1.0);
    let distance_fraction =
        (maximum_distance / viewport_width.max(viewport_height)).clamp(0.0, 1.0);
    let value = impact_fraction * distance_fraction;
    if value <= 0.0 {
        return;
    }
    if let Ok(entry) = super::layout_shift::create(scope, now, value, false, 0.0, sources) {
        super::performance_observer::queue_entry(scope, entry, "layout-shift");
    }
}

fn visual_layout_rect(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    layout: super::element_layout::LayoutBox,
) -> super::dom_rect_read_only::RectRecord {
    let border = layout.rect();
    let Some(text) = text_paint_rect(scope, element, layout) else {
        return border;
    };
    let left = border.x.min(text.x);
    let top = border.y.min(text.y);
    let right = (border.x + border.width).max(text.x + text.width);
    let bottom = (border.y + border.height).max(text.y + text.height);
    super::dom_rect_read_only::RectRecord {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

fn layout_shift_pixel_rect(
    rect: super::dom_rect_read_only::RectRecord,
    viewport_width: f64,
    viewport_height: f64,
) -> super::dom_rect_read_only::RectRecord {
    // Layout Instability scores use nearest device-pixel bounds.  This is
    // deliberately distinct from paint visibility's outward floor/ceil snap:
    // a 166.421875px button contributes 166px, as Edge's score demonstrates.
    let left = rect.x.max(0.0).round();
    let top = rect.y.max(0.0).round();
    let right = (rect.x + rect.width).min(viewport_width).round();
    let bottom = (rect.y + rect.height).min(viewport_height).round();
    super::dom_rect_read_only::RectRecord {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

fn movement_impact_area(
    old: super::dom_rect_read_only::RectRecord,
    current: super::dom_rect_read_only::RectRecord,
) -> f64 {
    rectangle_union_area(&[old, current])
}

fn rectangle_union_area(rects: &[super::dom_rect_read_only::RectRecord]) -> f64 {
    let mut xs = rects
        .iter()
        .flat_map(|rect| [rect.x, rect.x + rect.width])
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|left, right| left.total_cmp(right).is_eq());
    let mut area = 0.0;
    for window in xs.windows(2) {
        let left = window[0];
        let right = window[1];
        if right <= left {
            continue;
        }
        let mut intervals = rects
            .iter()
            .filter(|rect| rect.width > 0.0 && rect.height > 0.0)
            .filter(|rect| rect.x < right && rect.x + rect.width > left)
            .map(|rect| (rect.y, rect.y + rect.height))
            .collect::<Vec<_>>();
        intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut covered = 0.0;
        let mut active: Option<(f64, f64)> = None;
        for (top, bottom) in intervals {
            active = match active {
                None => Some((top, bottom)),
                Some((start, end)) if top > end => {
                    covered += (end - start).max(0.0);
                    Some((top, bottom))
                }
                Some((start, end)) => Some((start, end.max(bottom))),
            };
        }
        if let Some((start, end)) = active {
            covered += (end - start).max(0.0);
        }
        area += (right - left) * covered;
    }
    area
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RenderingPerformanceState>() {
        store.realms.remove(&realm_id);
    }
}
