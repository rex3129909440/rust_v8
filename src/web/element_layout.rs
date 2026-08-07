#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LayoutBox {
    pub(crate) rendered: bool,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) content_width: f64,
    pub(crate) content_height: f64,
    pub(crate) padding_left: f64,
    pub(crate) padding_right: f64,
    pub(crate) padding_top: f64,
    pub(crate) padding_bottom: f64,
    pub(crate) border_left: f64,
    pub(crate) border_right: f64,
    pub(crate) border_top: f64,
    pub(crate) border_bottom: f64,
}

impl LayoutBox {
    pub(crate) fn client_width(self) -> f64 {
        self.content_width + self.padding_left + self.padding_right
    }

    pub(crate) fn client_height(self) -> f64 {
        self.content_height + self.padding_top + self.padding_bottom
    }

    pub(crate) fn border_width(self) -> f64 {
        self.client_width() + self.border_left + self.border_right
    }

    pub(crate) fn border_height(self) -> f64 {
        self.client_height() + self.border_top + self.border_bottom
    }

    pub(crate) fn rect(self) -> super::dom_rect_read_only::RectRecord {
        super::dom_rect_read_only::RectRecord {
            x: self.x,
            y: self.y,
            width: self.border_width(),
            height: self.border_height(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScrollMetrics {
    pub(crate) client_width: f64,
    pub(crate) client_height: f64,
    pub(crate) scroll_width: f64,
    pub(crate) scroll_height: f64,
}

pub(crate) fn compute(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> LayoutBox {
    if super::element::record(scope, element).is_none()
        || !super::node::is_connected(scope, element)
        || current_iframe_is_display_none(scope)
        || hidden_by_display(scope, element)
    {
        return LayoutBox::default();
    }

    let horizontal_padding = side_lengths(scope, element, "padding", 0.0);
    let vertical_padding = horizontal_padding;
    let padding_left =
        property_length(scope, element, "padding-left").unwrap_or(horizontal_padding.3);
    let padding_right =
        property_length(scope, element, "padding-right").unwrap_or(horizontal_padding.1);
    let padding_top = property_length(scope, element, "padding-top").unwrap_or(vertical_padding.0);
    let padding_bottom =
        property_length(scope, element, "padding-bottom").unwrap_or(vertical_padding.2);

    let border_shorthand = property(scope, element, "border");
    let border_fallback = border_shorthand_width(&border_shorthand);
    let border_widths = if border_shorthand.is_empty() {
        side_lengths(scope, element, "border-width", border_fallback)
    } else {
        (
            border_fallback,
            border_fallback,
            border_fallback,
            border_fallback,
        )
    };
    let border_left = border_side_width(scope, element, "left").unwrap_or(border_widths.3);
    let border_right = border_side_width(scope, element, "right").unwrap_or(border_widths.1);
    let border_top = border_side_width(scope, element, "top").unwrap_or(border_widths.0);
    let border_bottom = border_side_width(scope, element, "bottom").unwrap_or(border_widths.2);

    let specified_width = property_length(scope, element, "width");
    let specified_height = property_length(scope, element, "height");
    let border_box_sizing =
        property(scope, element, "box-sizing").eq_ignore_ascii_case("border-box");
    let horizontal_edges = padding_left + padding_right + border_left + border_right;
    let vertical_edges = padding_top + padding_bottom + border_top + border_bottom;
    let content_width = specified_width
        .map(|width| {
            if border_box_sizing {
                (width - horizontal_edges).max(0.0)
            } else {
                width.max(0.0)
            }
        })
        .unwrap_or_else(|| default_content_width(scope, element, horizontal_edges));
    let content_height = specified_height
        .map(|height| {
            if border_box_sizing {
                (height - vertical_edges).max(0.0)
            } else {
                height.max(0.0)
            }
        })
        .unwrap_or_else(|| default_content_height(scope, element));

    let position = property(scope, element, "position");
    let fixed = position.eq_ignore_ascii_case("fixed");
    let positioned = fixed || position.eq_ignore_ascii_case("absolute");
    let mut x = property_length(scope, element, "left").unwrap_or(0.0);
    let mut y = property_length(scope, element, "top").unwrap_or(0.0);
    if !positioned {
        let margin = side_lengths(scope, element, "margin", 0.0);
        x = margin.3;
        y = margin.0;
        if let Some(parent) = nearest_element_parent(scope, element) {
            let parent_box = compute(scope, parent);
            x += parent_box.x + parent_box.border_left + parent_box.padding_left;
            y += parent_box.y + parent_box.border_top + parent_box.padding_top;
            if let Some(parent_record) = super::element::record(scope, parent) {
                x -= parent_record.scroll_left;
                y -= parent_record.scroll_top;
            }
        }
    } else if !fixed {
        if let Some(parent) = positioned_parent(scope, element) {
            let parent_box = compute(scope, parent);
            x += parent_box.x + parent_box.border_left;
            y += parent_box.y + parent_box.border_top;
            if let Some(parent_record) = super::element::record(scope, parent) {
                x -= parent_record.scroll_left;
                y -= parent_record.scroll_top;
            }
        }
        x -= super::window_view_state::scroll_x(scope);
        y -= super::window_view_state::scroll_y(scope);
    }

    LayoutBox {
        rendered: true,
        x,
        y,
        content_width,
        content_height,
        padding_left,
        padding_right,
        padding_top,
        padding_bottom,
        border_left,
        border_right,
        border_top,
        border_bottom,
    }
}

pub(crate) fn scroll_metrics(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> ScrollMetrics {
    let layout = compute(scope, element);
    if !layout.rendered {
        return ScrollMetrics::default();
    }
    let mut base_client_width = layout.client_width();
    let mut base_client_height = layout.client_height();
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("HTML")
        && super::window_view_state::inner_width(scope) == 0.0
        && super::window_view_state::inner_height(scope) == 0.0
    {
        return ScrollMetrics {
            client_width: base_client_width,
            client_height: base_client_height,
            scroll_width: base_client_width,
            scroll_height: base_client_height,
        };
    }
    let scroll_left = super::element::record(scope, element)
        .map(|record| record.scroll_left)
        .unwrap_or(0.0);
    let scroll_top = super::element::record(scope, element)
        .map(|record| record.scroll_top)
        .unwrap_or(0.0);
    let content_origin_x = layout.x + layout.border_left;
    let content_origin_y = layout.y + layout.border_top;
    let mut extent_width = base_client_width;
    let mut extent_height = base_client_height;
    for child in super::node::children(scope, element) {
        if super::element::record(scope, child).is_none() {
            continue;
        }
        let child_layout = compute(scope, child);
        if !child_layout.rendered {
            continue;
        }
        extent_width = extent_width
            .max(child_layout.x - content_origin_x + scroll_left + child_layout.border_width());
        extent_height = extent_height
            .max(child_layout.y - content_origin_y + scroll_top + child_layout.border_height());
    }

    if tag.eq_ignore_ascii_case("BODY") {
        if property_length(scope, element, "width").is_none() {
            base_client_width = base_client_width.max(extent_width);
        }
        if property_length(scope, element, "height").is_none() {
            base_client_height = base_client_height.max(extent_height);
        }
    }

    let overflow = property(scope, element, "overflow");
    let overflow_x = {
        let value = property(scope, element, "overflow-x");
        if value.is_empty() {
            overflow.clone()
        } else {
            value
        }
    };
    let overflow_y = {
        let value = property(scope, element, "overflow-y");
        if value.is_empty() { overflow } else { value }
    };
    let horizontal_mode = scrollbar_mode(&overflow_x);
    let vertical_mode = scrollbar_mode(&overflow_y);
    let scrollbar_width = 15.0;
    let mut horizontal = horizontal_mode == ScrollbarMode::Always;
    let mut vertical = vertical_mode == ScrollbarMode::Always;
    for _ in 0..2 {
        let available_width =
            (base_client_width - if vertical { scrollbar_width } else { 0.0 }).max(0.0);
        let available_height =
            (base_client_height - if horizontal { scrollbar_width } else { 0.0 }).max(0.0);
        horizontal |= horizontal_mode == ScrollbarMode::Automatic && extent_width > available_width;
        vertical |= vertical_mode == ScrollbarMode::Automatic && extent_height > available_height;
    }
    let client_width = (base_client_width - if vertical { scrollbar_width } else { 0.0 }).max(0.0);
    let client_height =
        (base_client_height - if horizontal { scrollbar_width } else { 0.0 }).max(0.0);
    ScrollMetrics {
        client_width,
        client_height,
        scroll_width: extent_width.max(client_width),
        scroll_height: extent_height.max(client_height),
    }
}

pub(crate) fn offset_parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let layout = compute(scope, element);
    if !layout.rendered || property(scope, element, "position").eq_ignore_ascii_case("fixed") {
        return None;
    }
    let mut parent = super::node::parent(scope, element);
    while let Some(candidate) = parent {
        if let Some(record) = super::element::record(scope, candidate) {
            if record.tag_name.eq_ignore_ascii_case("BODY")
                || !property(scope, candidate, "position").is_empty()
                    && !property(scope, candidate, "position").eq_ignore_ascii_case("static")
            {
                return Some(candidate);
            }
        }
        parent = super::node::parent(scope, candidate);
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollbarMode {
    Never,
    Automatic,
    Always,
}

fn scrollbar_mode(value: &str) -> ScrollbarMode {
    if value.eq_ignore_ascii_case("scroll") {
        ScrollbarMode::Always
    } else if value.eq_ignore_ascii_case("auto") {
        ScrollbarMode::Automatic
    } else {
        ScrollbarMode::Never
    }
}

pub(crate) fn rounded(value: f64) -> i32 {
    value.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
}

fn hidden_by_display(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    let mut current = Some(element);
    while let Some(candidate) = current {
        if super::element::record(scope, candidate).is_some()
            && property(scope, candidate, "display").eq_ignore_ascii_case("none")
        {
            return true;
        }
        current = layout_parent(scope, candidate);
    }
    false
}

fn current_iframe_is_display_none(scope: &v8::PinScope<'_, '_>) -> bool {
    super::html_i_frame_element::current_frame_element_for_layout(scope)
        .is_some_and(|frame| hidden_by_display(scope, frame))
}

fn layout_parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::node::parent(scope, node).or_else(|| super::shadow_root::host(scope, node))
}

fn nearest_element_parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut parent = layout_parent(scope, element);
    while let Some(candidate) = parent {
        if super::element::record(scope, candidate).is_some() {
            return Some(candidate);
        }
        parent = layout_parent(scope, candidate);
    }
    None
}

fn positioned_parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut parent = layout_parent(scope, element);
    while let Some(candidate) = parent {
        if super::element::record(scope, candidate).is_some() {
            let position = property(scope, candidate, "position");
            if !position.is_empty() && !position.eq_ignore_ascii_case("static") {
                return Some(candidate);
            }
        }
        parent = layout_parent(scope, candidate);
    }
    None
}

fn default_content_width(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    edges: f64,
) -> f64 {
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("IMG") {
        return super::html_image_element::layout_dimensions(scope, element)
            .map(|dimensions| f64::from(dimensions.0))
            .unwrap_or(0.0);
    }
    if tag.eq_ignore_ascii_case("HTML") {
        super::window_view_state::inner_width(scope)
    } else if tag.eq_ignore_ascii_case("BODY") {
        (super::window_view_state::inner_width(scope) - 16.0).max(0.0)
    } else if let Some(parent) = nearest_element_parent(scope, element) {
        (compute(scope, parent).content_width - edges).max(0.0)
    } else {
        0.0
    }
}

fn default_content_height(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> f64 {
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("HTML") {
        super::window_view_state::inner_height(scope)
    } else if tag.eq_ignore_ascii_case("IMG") {
        super::html_image_element::layout_dimensions(scope, element)
            .map(|dimensions| f64::from(dimensions.1))
            .unwrap_or(0.0)
    } else {
        0.0
    }
}

fn property(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    super::get_computed_style_global::computed_property_value(scope, element, name)
}

fn property_length(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    resolve_length(scope, element, name, &property(scope, element, name))
}

fn resolve_length(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    property_name: &str,
    value: &str,
) -> Option<f64> {
    let (font_size, root_font_size) = font_sizes(scope, element);
    if property_name == "font-size" {
        return Some(font_size);
    }
    if property_name == "line-height" {
        return super::css_calculation::resolve_line_height(
            value,
            super::css_calculation::EvaluationContext {
                viewport_width: super::window_view_state::inner_width(scope),
                viewport_height: super::window_view_state::inner_height(scope),
                percentage_basis: Some(font_size),
                font_size,
                root_font_size,
                intrinsic_size: None,
            },
        );
    }
    if let Some(length) = parse_length(value) {
        return Some(super::css_calculation::layout_unit(length));
    }
    let viewport_width = super::window_view_state::inner_width(scope);
    let viewport_height = super::window_view_state::inner_height(scope);
    let vertical = matches!(
        property_name,
        "height" | "min-height" | "max-height" | "top" | "bottom"
    );
    let percentage_basis = if value.contains('%') {
        Some(
            if let Some(parent) = nearest_element_parent(scope, element) {
                let parent = compute(scope, parent);
                if vertical {
                    parent.content_height
                } else {
                    parent.content_width
                }
            } else if vertical {
                viewport_height
            } else {
                viewport_width
            },
        )
    } else {
        None
    };
    let intrinsic_size = value.to_ascii_lowercase().contains("calc-size(").then(|| {
        if vertical {
            default_content_height(scope, element)
        } else {
            default_content_width(scope, element, 0.0)
        }
    });
    super::css_calculation::resolve_length(
        value,
        super::css_calculation::EvaluationContext {
            viewport_width,
            viewport_height,
            percentage_basis,
            font_size,
            root_font_size,
            intrinsic_size,
        },
    )
}

fn font_sizes(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> (f64, f64) {
    let mut ancestry = vec![element];
    let mut parent = nearest_element_parent(scope, element);
    while let Some(candidate) = parent {
        ancestry.push(candidate);
        parent = nearest_element_parent(scope, candidate);
    }
    ancestry.reverse();

    let viewport_width = super::window_view_state::inner_width(scope);
    let viewport_height = super::window_view_state::inner_height(scope);
    let mut current = 16.0;
    let mut root = 16.0;
    for (index, candidate) in ancestry.into_iter().enumerate() {
        let parent_size = current;
        if let Some(source) = super::get_computed_style_global::cascaded_property_source(
            scope,
            candidate,
            "font-size",
        ) {
            current = font_size_keyword(&source, parent_size).unwrap_or_else(|| {
                super::css_calculation::resolve_length(
                    &source,
                    super::css_calculation::EvaluationContext {
                        viewport_width,
                        viewport_height,
                        percentage_basis: Some(parent_size),
                        font_size: parent_size,
                        root_font_size: root,
                        intrinsic_size: None,
                    },
                )
                .unwrap_or(parent_size)
            });
        }
        if index == 0 {
            root = current;
        }
    }
    (current, root)
}

fn font_size_keyword(value: &str, parent: f64) -> Option<f64> {
    Some(match value.trim().to_ascii_lowercase().as_str() {
        "xx-small" => 9.0,
        "x-small" => 10.0,
        "small" => 13.0,
        "medium" => 16.0,
        "large" => 18.0,
        "x-large" => 24.0,
        "xx-large" => 32.0,
        "xxx-large" => 48.0,
        "smaller" => parent * 0.8,
        "larger" => parent * 1.2,
        _ => return None,
    })
}

pub(crate) fn resolve_css_length(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    property_name: &str,
    value: &str,
) -> Option<f64> {
    resolve_length(scope, element, property_name, value)
}

fn parse_length(value: &str) -> Option<f64> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    let number = value.strip_suffix("px")?.trim().parse::<f64>().ok()?;
    number.is_finite().then_some(number)
}

fn side_lengths(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
    fallback: f64,
) -> (f64, f64, f64, f64) {
    let value = property(scope, element, name);
    let values = value
        .split_whitespace()
        .filter_map(|value| resolve_length(scope, element, name, value))
        .collect::<Vec<_>>();
    match values.as_slice() {
        [all] => (*all, *all, *all, *all),
        [vertical, horizontal] => (*vertical, *horizontal, *vertical, *horizontal),
        [top, horizontal, bottom] => (*top, *horizontal, *bottom, *horizontal),
        [top, right, bottom, left, ..] => (*top, *right, *bottom, *left),
        _ => (fallback, fallback, fallback, fallback),
    }
}

fn border_shorthand_width(border: &str) -> f64 {
    if border.is_empty() || border.split_whitespace().any(|part| part == "none") {
        return 0.0;
    }
    border
        .split_whitespace()
        .find_map(parse_length)
        .unwrap_or(3.0)
}

fn border_side_width(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    side: &str,
) -> Option<f64> {
    let style = property(scope, element, &format!("border-{side}-style"));
    if style.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    property_length(scope, element, &format!("border-{side}-width"))
}
