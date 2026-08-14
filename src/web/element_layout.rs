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

    let position = property(scope, element, "position");
    let fixed = position.eq_ignore_ascii_case("fixed");
    let positioned = fixed || position.eq_ignore_ascii_case("absolute");
    let positioned_parent_box = if positioned && !fixed {
        positioned_parent(scope, element).map(|parent| (parent, compute(scope, parent)))
    } else {
        None
    };
    let horizontal_percentage_basis = positioned_parent_box
        .map(|(_, parent)| parent.content_width + parent.padding_left + parent.padding_right);
    let vertical_percentage_basis = positioned_parent_box
        .map(|(_, parent)| parent.content_height + parent.padding_top + parent.padding_bottom);
    let specified_width =
        percentage_aware_property_length(scope, element, "width", horizontal_percentage_basis);
    let specified_height =
        percentage_aware_property_length(scope, element, "height", vertical_percentage_basis);
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
        .unwrap_or_else(|| {
            grid_item_content_width(scope, element, horizontal_edges)
                .unwrap_or_else(|| default_content_width(scope, element, horizontal_edges))
        });
    let content_height = specified_height
        .map(|height| {
            if border_box_sizing {
                (height - vertical_edges).max(0.0)
            } else {
                height.max(0.0)
            }
        })
        .unwrap_or_else(|| {
            grid_container_content_height(scope, element)
                .unwrap_or_else(|| default_content_height(scope, element, content_width))
        });

    let mut x =
        percentage_aware_property_length(scope, element, "left", horizontal_percentage_basis)
            .unwrap_or(0.0);
    let mut y = percentage_aware_property_length(scope, element, "top", vertical_percentage_basis)
        .unwrap_or(0.0);
    if !positioned {
        let margin = side_lengths(scope, element, "margin", 0.0);
        let margin_left = property_length(scope, element, "margin-left").unwrap_or(margin.3);
        let margin_top = property_length(scope, element, "margin-top").unwrap_or(margin.0);
        x = margin_left;
        y = margin_top;
        if let Some(parent) = nearest_element_parent(scope, element) {
            let parent_box = compute(scope, parent);
            x += parent_box.x + parent_box.border_left + parent_box.padding_left;
            y += parent_box.y + parent_box.border_top + parent_box.padding_top;
            let parent_display = property(scope, parent, "display").to_ascii_lowercase();
            if parent_display == "flex" || parent_display == "inline-flex" {
                let (flow_x, flow_y) = flex_item_offset(scope, parent, element, parent_box);
                x = parent_box.x + parent_box.border_left + parent_box.padding_left + flow_x;
                y = parent_box.y + parent_box.border_top + parent_box.padding_top + flow_y;
            } else if parent_display == "grid" || parent_display == "inline-grid" {
                let (flow_x, flow_y) = grid_item_offset(scope, parent, element);
                x = parent_box.x + parent_box.border_left + parent_box.padding_left + flow_x;
                y = parent_box.y + parent_box.border_top + parent_box.padding_top + flow_y;
            } else if is_block_level(scope, element) {
                y = parent_box.y
                    + parent_box.border_top
                    + parent_box.padding_top
                    + normal_flow_block_offset(scope, parent, element, margin_top);
            } else {
                let (flow_x, flow_y) = inline_flow_offset(scope, parent, element, parent_box);
                x = parent_box.x + parent_box.border_left + parent_box.padding_left + flow_x;
                y = parent_box.y + parent_box.border_top + parent_box.padding_top + flow_y;
            }
            if let Some(parent_record) = super::element::record(scope, parent) {
                x -= parent_record.scroll_left;
                y -= parent_record.scroll_top;
            }
        }
    } else if !fixed {
        if let Some((parent, parent_box)) = positioned_parent_box {
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

pub(crate) fn bounding_rect(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> super::dom_rect_read_only::RectRecord {
    let mut layout = compute(scope, element);
    let body_with_auto_height = super::element::record(scope, element)
        .is_some_and(|record| record.tag_name.eq_ignore_ascii_case("BODY"))
        && super::get_computed_style_global::cascaded_property_source(scope, element, "height")
            .is_none();
    if body_with_auto_height {
        layout.content_height = auto_flow_content_height(scope, element, layout.content_width);
    }
    layout.rect()
}

pub(crate) fn is_block_level(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> bool {
    let display = property(scope, element, "display").to_ascii_lowercase();
    if !display.is_empty() {
        return matches!(
            display.as_str(),
            "block" | "flow-root" | "list-item" | "table" | "flex" | "grid"
        );
    }
    super::element::record(scope, element).is_some_and(|record| {
        matches!(
            record.tag_name.as_str(),
            "ADDRESS"
                | "ARTICLE"
                | "ASIDE"
                | "BLOCKQUOTE"
                | "BODY"
                | "DD"
                | "DIV"
                | "DL"
                | "DT"
                | "FIELDSET"
                | "FIGCAPTION"
                | "FIGURE"
                | "FOOTER"
                | "FORM"
                | "H1"
                | "H2"
                | "H3"
                | "H4"
                | "H5"
                | "H6"
                | "HEADER"
                | "HGROUP"
                | "HR"
                | "HTML"
                | "LEGEND"
                | "LI"
                | "MAIN"
                | "MENU"
                | "NAV"
                | "OL"
                | "P"
                | "PRE"
                | "SEARCH"
                | "SECTION"
                | "SUMMARY"
                | "TABLE"
                | "UL"
        )
    })
}

pub(crate) fn uses_implicit_default_font(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> bool {
    let mut current = Some(element);
    while let Some(candidate) = current {
        if super::get_computed_style_global::own_specified_property_source(
            scope,
            candidate,
            "font-family",
        )
        .is_some()
        {
            return false;
        }
        current = nearest_element_parent(scope, candidate);
    }
    true
}

fn normal_flow_block_offset(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    element: v8::Local<'_, v8::Object>,
    current_margin_top: f64,
) -> f64 {
    let mut offset = 0.0;
    let mut previous_margin_bottom: f64 = 0.0;
    let parent_box = compute(scope, parent);
    let mut inline_x = 0.0;
    let mut inline_height: f64 = 0.0;
    for sibling in super::node::children(scope, parent) {
        if sibling == element {
            offset += inline_height;
            return offset + previous_margin_bottom.max(current_margin_top);
        }
        let Some(node_record) = super::node::record(scope, sibling) else {
            continue;
        };
        if node_record.node_type == super::node::TEXT_NODE {
            if node_record
                .node_value
                .as_deref()
                .is_some_and(|text| !text.trim().is_empty())
            {
                inline_height = inline_height.max(line_box_height(scope, parent));
            }
            continue;
        }
        if super::element::record(scope, sibling).is_none() || hidden_by_display(scope, sibling) {
            continue;
        }
        let position = property(scope, sibling, "position");
        if matches!(position.to_ascii_lowercase().as_str(), "absolute" | "fixed") {
            continue;
        }
        if !is_block_level(scope, sibling) {
            let intrinsic = intrinsic_replaced_dimensions(scope, sibling);
            let metrics = flow_box_metrics(
                scope,
                sibling,
                parent_box.content_width,
                parent_box.content_height,
                intrinsic.0,
                intrinsic.1,
            );
            if inline_x > 0.0 && inline_x + metrics.outer_width() > parent_box.content_width {
                offset += inline_height;
                inline_x = 0.0;
                inline_height = 0.0;
            }
            inline_x += metrics.outer_width();
            inline_height = inline_height.max(inline_line_outer_height(scope, sibling, metrics));
            continue;
        }
        if inline_height > 0.0 {
            offset += inline_height;
            inline_x = 0.0;
            inline_height = 0.0;
            previous_margin_bottom = 0.0;
        }
        let intrinsic = intrinsic_replaced_dimensions(scope, sibling);
        let metrics = flow_box_metrics(
            scope,
            sibling,
            parent_box.content_width,
            parent_box.content_height,
            parent_box.content_width,
            shallow_auto_content_height(scope, sibling, parent_box.content_width).max(intrinsic.1),
        );
        offset += previous_margin_bottom.max(metrics.margin_top);
        offset += metrics.border_height;
        previous_margin_bottom = metrics.margin_bottom;
    }
    offset + inline_height + previous_margin_bottom.max(current_margin_top)
}

#[derive(Clone, Copy, Default)]
struct FlowBoxMetrics {
    border_width: f64,
    border_height: f64,
    margin_top: f64,
    margin_right: f64,
    margin_bottom: f64,
    margin_left: f64,
}

impl FlowBoxMetrics {
    fn outer_width(self) -> f64 {
        self.margin_left + self.border_width + self.margin_right
    }

    fn outer_height(self) -> f64 {
        self.margin_top + self.border_height + self.margin_bottom
    }
}

fn flow_box_metrics(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    horizontal_basis: f64,
    vertical_basis: f64,
    fallback_width: f64,
    fallback_height: f64,
) -> FlowBoxMetrics {
    let padding = side_lengths(scope, element, "padding", 0.0);
    let padding_left =
        percentage_aware_property_length(scope, element, "padding-left", Some(horizontal_basis))
            .unwrap_or(padding.3);
    let padding_right =
        percentage_aware_property_length(scope, element, "padding-right", Some(horizontal_basis))
            .unwrap_or(padding.1);
    let padding_top =
        percentage_aware_property_length(scope, element, "padding-top", Some(horizontal_basis))
            .unwrap_or(padding.0);
    let padding_bottom =
        percentage_aware_property_length(scope, element, "padding-bottom", Some(horizontal_basis))
            .unwrap_or(padding.2);
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
    let horizontal_edges = padding_left + padding_right + border_left + border_right;
    let vertical_edges = padding_top + padding_bottom + border_top + border_bottom;
    let border_box_sizing =
        property(scope, element, "box-sizing").eq_ignore_ascii_case("border-box");
    let width = percentage_aware_property_length(scope, element, "width", Some(horizontal_basis))
        .unwrap_or(fallback_width)
        .max(0.0);
    let height = percentage_aware_property_length(scope, element, "height", Some(vertical_basis))
        .unwrap_or(fallback_height)
        .max(0.0);
    let border_width = if border_box_sizing {
        width
    } else {
        width + horizontal_edges
    };
    let border_height = if border_box_sizing {
        height
    } else {
        height + vertical_edges
    };
    let margins = side_lengths(scope, element, "margin", 0.0);
    FlowBoxMetrics {
        border_width,
        border_height,
        margin_top: percentage_aware_property_length(
            scope,
            element,
            "margin-top",
            Some(horizontal_basis),
        )
        .unwrap_or(margins.0),
        margin_right: percentage_aware_property_length(
            scope,
            element,
            "margin-right",
            Some(horizontal_basis),
        )
        .unwrap_or(margins.1),
        margin_bottom: percentage_aware_property_length(
            scope,
            element,
            "margin-bottom",
            Some(horizontal_basis),
        )
        .unwrap_or(margins.2),
        margin_left: percentage_aware_property_length(
            scope,
            element,
            "margin-left",
            Some(horizontal_basis),
        )
        .unwrap_or(margins.3),
    }
}

fn intrinsic_replaced_dimensions(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> (f64, f64) {
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("IMG") {
        return super::html_image_element::layout_dimensions(scope, element)
            .map(|(width, height)| (f64::from(width), f64::from(height)))
            .unwrap_or_default();
    }
    if tag.eq_ignore_ascii_case("BUTTON") {
        let padding = side_lengths(scope, element, "padding", 0.0);
        let border = border_shorthand_width(&property(scope, element, "border"));
        let horizontal_edges = padding.1 + padding.3 + border * 2.0;
        let vertical_edges = padding.0 + padding.2 + border * 2.0;
        let text_width = button_content_width(scope, element);
        return (
            text_width + horizontal_edges,
            line_box_height(scope, element) + vertical_edges,
        );
    }
    (0.0, 0.0)
}

fn button_content_width(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> f64 {
    let text = super::node::text_content(scope, element);
    let font = property(scope, element, "font");
    let measured = super::offscreen_canvas_rendering_context_2d::measured_text_width_for_font(
        scope, &text, &font,
    );
    // Blink rounds the shrink-to-fit control content width outward to its
    // 1/64 CSS-pixel layout unit before adding padding and borders.
    (measured * 64.0).ceil() / 64.0
}

fn inline_line_outer_height(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    metrics: FlowBoxMetrics,
) -> f64 {
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("BUTTON") {
        // Blink's default Windows button sits two CSS pixels below the top of
        // the surrounding line box, so the following block begins after a
        // 23px line even though the button border box is 21px high.
        metrics.outer_height() + 2.0
    } else if tag.eq_ignore_ascii_case("IMG") {
        // An inline image's baseline is its bottom margin edge.  Blink keeps
        // the parent font strut below that baseline, so the line containing a
        // 180px image is 184px high with the default 16px Times metrics.
        // This descent does not change the image border box itself.
        let (font_size, _) = font_sizes(scope, element);
        let line_height = standard_font_line_height(scope, element, font_size);
        let descent = (font_size * 3.0 / 16.0 + (line_height - font_size) / 2.0)
            .round()
            .max(0.0);
        (metrics.outer_height() + descent).max(line_height)
    } else {
        metrics.outer_height().max(line_box_height(scope, element))
    }
}

fn inline_top_alignment_offset(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> f64 {
    super::element::record(scope, element)
        .is_some_and(|record| record.tag_name.eq_ignore_ascii_case("BUTTON"))
        .then_some(2.0)
        .unwrap_or(0.0)
}

fn in_flow_element_children<'s>(
    scope: &v8::PinScope<'s, '_>,
    parent: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::node::children(scope, parent)
        .into_iter()
        .filter(|child| {
            super::element::record(scope, *child).is_some()
                && !hidden_by_display(scope, *child)
                && !matches!(
                    property(scope, *child, "position")
                        .to_ascii_lowercase()
                        .as_str(),
                    "absolute" | "fixed"
                )
        })
        .collect()
}

fn inline_flow_offset(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    element: v8::Local<'_, v8::Object>,
    parent_box: LayoutBox,
) -> (f64, f64) {
    let current_intrinsic = intrinsic_replaced_dimensions(scope, element);
    let current = flow_box_metrics(
        scope,
        element,
        parent_box.content_width,
        parent_box.content_height,
        current_intrinsic.0,
        current_intrinsic.1,
    );
    let mut line_x = 0.0;
    let mut line_y = 0.0;
    let mut line_height: f64 = 0.0;
    for sibling in in_flow_element_children(scope, parent) {
        if sibling == element {
            if line_x > 0.0 && line_x + current.outer_width() > parent_box.content_width {
                line_x = 0.0;
                line_y += line_height;
            }
            return (
                line_x + current.margin_left,
                line_y + current.margin_top + inline_top_alignment_offset(scope, element),
            );
        }
        if is_block_level(scope, sibling) {
            line_x = 0.0;
            line_y += line_height;
            let metrics = flow_box_metrics(
                scope,
                sibling,
                parent_box.content_width,
                parent_box.content_height,
                0.0,
                0.0,
            );
            line_y += metrics.outer_height();
            line_height = 0.0;
            continue;
        }
        let intrinsic = intrinsic_replaced_dimensions(scope, sibling);
        let metrics = flow_box_metrics(
            scope,
            sibling,
            parent_box.content_width,
            parent_box.content_height,
            intrinsic.0,
            intrinsic.1,
        );
        if line_x > 0.0 && line_x + metrics.outer_width() > parent_box.content_width {
            line_x = 0.0;
            line_y += line_height;
            line_height = 0.0;
        }
        line_x += metrics.outer_width();
        line_height = line_height.max(inline_line_outer_height(scope, sibling, metrics));
    }
    (
        current.margin_left,
        current.margin_top + inline_top_alignment_offset(scope, element),
    )
}

fn gap_lengths(
    scope: &v8::PinScope<'_, '_>,
    container: v8::Local<'_, v8::Object>,
    horizontal_basis: f64,
) -> (f64, f64) {
    let gap = super::get_computed_style_global::cascaded_property_source(scope, container, "gap")
        .unwrap_or_else(|| property(scope, container, "gap"));
    let mut values = gap.split_whitespace();
    let first = values
        .next()
        .and_then(|value| {
            resolve_length_with_percentage_basis(
                scope,
                container,
                "row-gap",
                value,
                Some(horizontal_basis),
            )
        })
        .unwrap_or(0.0);
    let second = values
        .next()
        .and_then(|value| {
            resolve_length_with_percentage_basis(
                scope,
                container,
                "column-gap",
                value,
                Some(horizontal_basis),
            )
        })
        .unwrap_or(first);
    let row = percentage_aware_property_length(scope, container, "row-gap", Some(horizontal_basis))
        .unwrap_or(first);
    let column =
        percentage_aware_property_length(scope, container, "column-gap", Some(horizontal_basis))
            .unwrap_or(second);
    (row, column)
}

fn flex_item_offset(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    element: v8::Local<'_, v8::Object>,
    parent_box: LayoutBox,
) -> (f64, f64) {
    let children = in_flow_element_children(scope, parent);
    let items = children
        .iter()
        .map(|child| {
            let intrinsic = intrinsic_replaced_dimensions(scope, *child);
            flow_box_metrics(
                scope,
                *child,
                parent_box.content_width,
                parent_box.content_height,
                intrinsic.0,
                intrinsic.1,
            )
        })
        .collect::<Vec<_>>();
    let Some(index) = children.iter().position(|child| *child == element) else {
        return (0.0, 0.0);
    };
    let (row_gap, column_gap) = gap_lengths(scope, parent, parent_box.content_width);
    let column = property(scope, parent, "flex-direction")
        .to_ascii_lowercase()
        .starts_with("column");
    let reverse = property(scope, parent, "flex-direction")
        .to_ascii_lowercase()
        .ends_with("reverse");
    let visual_index = if reverse {
        items.len().saturating_sub(index + 1)
    } else {
        index
    };
    let ordered = if reverse {
        items.iter().rev().copied().collect::<Vec<_>>()
    } else {
        items.clone()
    };
    let gap = if column { row_gap } else { column_gap };
    let main_size = if column {
        parent_box.content_height
    } else {
        parent_box.content_width
    };
    let occupied = ordered
        .iter()
        .map(|item| {
            if column {
                item.outer_height()
            } else {
                item.outer_width()
            }
        })
        .sum::<f64>()
        + gap * ordered.len().saturating_sub(1) as f64;
    let free = (main_size - occupied).max(0.0);
    let justify = property(scope, parent, "justify-content").to_ascii_lowercase();
    let (start, extra_gap) = match justify.as_str() {
        "center" => (free / 2.0, 0.0),
        "flex-end" | "end" | "right" => (free, 0.0),
        "space-between" if ordered.len() > 1 => (0.0, free / (ordered.len() - 1) as f64),
        "space-around" if !ordered.is_empty() => {
            let between = free / ordered.len() as f64;
            (between / 2.0, between)
        }
        "space-evenly" if !ordered.is_empty() => {
            let between = free / (ordered.len() + 1) as f64;
            (between, between)
        }
        _ => (0.0, 0.0),
    };
    let preceding = ordered
        .iter()
        .take(visual_index)
        .map(|item| {
            if column {
                item.outer_height()
            } else {
                item.outer_width()
            }
        })
        .sum::<f64>();
    let main = start + preceding + (gap + extra_gap) * visual_index as f64;
    let item = items[index];
    let align = {
        let own = property(scope, element, "align-self").to_ascii_lowercase();
        if own.is_empty() || own == "auto" {
            property(scope, parent, "align-items").to_ascii_lowercase()
        } else {
            own
        }
    };
    let cross_size = if column {
        parent_box.content_width
    } else {
        parent_box.content_height
    };
    let item_cross = if column {
        item.outer_width()
    } else {
        item.outer_height()
    };
    let cross = match align.as_str() {
        "center" => (cross_size - item_cross).max(0.0) / 2.0,
        "flex-end" | "end" => (cross_size - item_cross).max(0.0),
        _ => 0.0,
    };
    if column {
        (cross + item.margin_left, main + item.margin_top)
    } else {
        (main + item.margin_left, cross + item.margin_top)
    }
}

fn grid_track_sizes(
    scope: &v8::PinScope<'_, '_>,
    container: v8::Local<'_, v8::Object>,
    horizontal_basis: f64,
) -> Vec<f64> {
    let source = super::get_computed_style_global::cascaded_property_source(
        scope,
        container,
        "grid-template-columns",
    )
    .unwrap_or_else(|| property(scope, container, "grid-template-columns"));
    let tracks = source
        .split_whitespace()
        .filter_map(|value| {
            resolve_length_with_percentage_basis(
                scope,
                container,
                "width",
                value,
                Some(horizontal_basis),
            )
        })
        .collect::<Vec<_>>();
    if tracks.is_empty() {
        vec![horizontal_basis.max(0.0)]
    } else {
        tracks
    }
}

fn grid_item_content_width(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    edges: f64,
) -> Option<f64> {
    let parent = nearest_element_parent(scope, element)?;
    let display = property(scope, parent, "display").to_ascii_lowercase();
    if display != "grid" && display != "inline-grid" {
        return None;
    }
    let parent_box = compute(scope, parent);
    let children = in_flow_element_children(scope, parent);
    let index = children.iter().position(|child| *child == element)?;
    let tracks = grid_track_sizes(scope, parent, parent_box.content_width);
    Some((tracks[index % tracks.len()] - edges).max(0.0))
}

fn grid_row_heights(
    scope: &v8::PinScope<'_, '_>,
    container: v8::Local<'_, v8::Object>,
    horizontal_basis: f64,
    vertical_basis: f64,
    columns: usize,
) -> Vec<f64> {
    let mut rows = Vec::<f64>::new();
    for (index, child) in in_flow_element_children(scope, container)
        .into_iter()
        .enumerate()
    {
        let intrinsic = intrinsic_replaced_dimensions(scope, child);
        let metrics = flow_box_metrics(
            scope,
            child,
            horizontal_basis,
            vertical_basis,
            intrinsic.0,
            intrinsic.1,
        );
        let row = index / columns.max(1);
        if rows.len() <= row {
            rows.push(metrics.outer_height());
        } else {
            rows[row] = rows[row].max(metrics.outer_height());
        }
    }
    rows
}

fn grid_container_content_height(
    scope: &v8::PinScope<'_, '_>,
    container: v8::Local<'_, v8::Object>,
) -> Option<f64> {
    let display = property(scope, container, "display").to_ascii_lowercase();
    if display != "grid" && display != "inline-grid" {
        return None;
    }
    let width = property_length(scope, container, "width")
        .unwrap_or_else(|| default_content_width(scope, container, 0.0));
    let tracks = grid_track_sizes(scope, container, width);
    let rows = grid_row_heights(scope, container, width, 0.0, tracks.len());
    let (row_gap, _) = gap_lengths(scope, container, width);
    Some(rows.iter().sum::<f64>() + row_gap * rows.len().saturating_sub(1) as f64)
}

fn grid_item_offset(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    element: v8::Local<'_, v8::Object>,
) -> (f64, f64) {
    let parent_box = compute(scope, parent);
    let children = in_flow_element_children(scope, parent);
    let Some(index) = children.iter().position(|child| *child == element) else {
        return (0.0, 0.0);
    };
    let tracks = grid_track_sizes(scope, parent, parent_box.content_width);
    let columns = tracks.len().max(1);
    let row = index / columns;
    let column = index % columns;
    let rows = grid_row_heights(
        scope,
        parent,
        parent_box.content_width,
        parent_box.content_height,
        columns,
    );
    let (row_gap, column_gap) = gap_lengths(scope, parent, parent_box.content_width);
    let intrinsic = intrinsic_replaced_dimensions(scope, element);
    let current = flow_box_metrics(
        scope,
        element,
        parent_box.content_width,
        parent_box.content_height,
        intrinsic.0,
        intrinsic.1,
    );
    (
        tracks.iter().take(column).sum::<f64>() + column_gap * column as f64 + current.margin_left,
        rows.iter().take(row).sum::<f64>() + row_gap * row as f64 + current.margin_top,
    )
}

fn auto_flow_content_height(
    scope: &v8::PinScope<'_, '_>,
    container: v8::Local<'_, v8::Object>,
    container_width: f64,
) -> f64 {
    let display = property(scope, container, "display").to_ascii_lowercase();
    if display == "grid" || display == "inline-grid" {
        return grid_container_content_height(scope, container).unwrap_or(0.0);
    }
    let children = in_flow_element_children(scope, container);
    if display == "flex" || display == "inline-flex" {
        let column = property(scope, container, "flex-direction")
            .to_ascii_lowercase()
            .starts_with("column");
        let (row_gap, _) = gap_lengths(scope, container, container_width);
        let heights = children
            .iter()
            .map(|child| {
                let intrinsic = intrinsic_replaced_dimensions(scope, *child);
                flow_box_metrics(
                    scope,
                    *child,
                    container_width,
                    0.0,
                    intrinsic.0,
                    shallow_auto_content_height(scope, *child, container_width).max(intrinsic.1),
                )
                .outer_height()
            })
            .collect::<Vec<_>>();
        return if column {
            heights.iter().sum::<f64>() + row_gap * heights.len().saturating_sub(1) as f64
        } else {
            heights.into_iter().fold(0.0, f64::max)
        };
    }

    let mut height = 0.0;
    let mut previous_margin_bottom: f64 = 0.0;
    let mut inline_height =
        super::inline_text_layout::layout_for_element(scope, container, container_width, 0.0, 0.0)
            .content_height;
    for child in children {
        let intrinsic = intrinsic_replaced_dimensions(scope, child);
        let fallback_height =
            shallow_auto_content_height(scope, child, container_width).max(intrinsic.1);
        let metrics = flow_box_metrics(
            scope,
            child,
            container_width,
            0.0,
            intrinsic.0,
            fallback_height,
        );
        if is_block_level(scope, child) {
            if inline_height > 0.0 {
                height += inline_height;
                inline_height = 0.0;
            }
            height += previous_margin_bottom.max(metrics.margin_top);
            height += metrics.border_height;
            previous_margin_bottom = metrics.margin_bottom;
        } else {
            inline_height = inline_height.max(metrics.outer_height());
        }
    }
    height + inline_height + previous_margin_bottom
}

fn shallow_auto_content_height(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    content_width: f64,
) -> f64 {
    let display = property(scope, element, "display").to_ascii_lowercase();
    if display == "grid" || display == "inline-grid" {
        return grid_container_content_height(scope, element).unwrap_or(0.0);
    }
    super::inline_text_layout::layout_for_element(scope, element, content_width, 0.0, 0.0)
        .content_height
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
    let mut has_inline_line_box = false;
    let mut has_inline_text = false;
    for child in super::node::children(scope, element) {
        let Some(child_record) = super::node::record(scope, child) else {
            continue;
        };
        if child_record.node_type == super::node::TEXT_NODE {
            let has_text = child_record
                .node_value
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            has_inline_line_box |= has_text;
            has_inline_text |= has_text;
            continue;
        }
        if super::element::record(scope, child).is_none() {
            continue;
        }
        let child_layout = compute(scope, child);
        if !child_layout.rendered {
            continue;
        }
        let participates_inline = participates_in_inline_line_box(scope, child);
        has_inline_line_box |= participates_inline;
        has_inline_text |=
            participates_inline && !super::node::text_content(scope, child).trim().is_empty();
        extent_width = extent_width
            .max(child_layout.x - content_origin_x + scroll_left + child_layout.border_width());
        extent_height = extent_height
            .max(child_layout.y - content_origin_y + scroll_top + child_layout.border_height());
    }

    if tag.eq_ignore_ascii_case("BODY") {
        if has_inline_line_box {
            let line_height = if has_inline_text {
                line_box_height(scope, element)
            } else {
                inline_strut_height(scope, element)
            };
            extent_height = extent_height.max(line_height);
        }
        if property_length(scope, element, "width").is_none() {
            base_client_width = base_client_width.max(extent_width);
        }
        if property_length(scope, element, "height").is_none() {
            base_client_height = base_client_height.max(extent_height);
        }
    }

    let overflow = property(scope, element, "overflow");
    let overflow_x = {
        // The computed initial `overflow-x: visible` is not a cascaded
        // longhand and therefore cannot override `overflow: auto`.
        super::get_computed_style_global::cascaded_property_source(scope, element, "overflow-x")
            .unwrap_or_else(|| overflow.clone())
    };
    let overflow_y = {
        super::get_computed_style_global::cascaded_property_source(scope, element, "overflow-y")
            .unwrap_or(overflow)
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

fn participates_in_inline_line_box(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> bool {
    let display = property(scope, element, "display");
    if display.eq_ignore_ascii_case("none") {
        return false;
    }
    if !display.is_empty() {
        return display
            .split_whitespace()
            .next()
            .is_some_and(|outer| outer.eq_ignore_ascii_case("inline"));
    }
    super::element::record(scope, element).is_some_and(|record| {
        matches!(
            record.tag_name.to_ascii_uppercase().as_str(),
            "AUDIO"
                | "BUTTON"
                | "CANVAS"
                | "EMBED"
                | "IFRAME"
                | "IMG"
                | "INPUT"
                | "OBJECT"
                | "SELECT"
                | "TEXTAREA"
                | "VIDEO"
        )
    })
}

pub(crate) fn line_box_height(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> f64 {
    property_length(scope, element, "line-height").unwrap_or_else(|| {
        let (font_size, _) = font_sizes(scope, element);
        if uses_implicit_default_font(scope, element) {
            return super::font_metric_tables::implicit_default_line_height(font_size);
        }
        standard_font_line_height(scope, element, font_size)
    })
}

fn inline_strut_height(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> f64 {
    property_length(scope, element, "line-height").unwrap_or_else(|| {
        let (font_size, _) = font_sizes(scope, element);
        standard_font_line_height(scope, element, font_size)
    })
}

fn standard_font_line_height(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    font_size: f64,
) -> f64 {
    let family = property(scope, element, "font-family").to_ascii_lowercase();
    // Normal line height is derived from the active font's ascent, descent
    // and line gap. A replaced inline with no text uses this CSS strut even
    // when the document's implicit text run takes the captured default-font
    // shaping path.
    let scale = if family.contains("segoe ui") {
        1.3125
    } else if family.contains("times new roman") {
        55.0 / 48.0
    } else {
        1.125
    };
    super::css_calculation::layout_unit((font_size * scale).round().max(1.0))
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
    } else if tag.eq_ignore_ascii_case("BUTTON") {
        button_content_width(scope, element)
    } else if let Some(parent) = nearest_element_parent(scope, element) {
        (compute(scope, parent).content_width - edges).max(0.0)
    } else {
        0.0
    }
}

fn calc_size_auto_content_width(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    explicit_containing_width: Option<f64>,
) -> f64 {
    if let Some(width) = explicit_containing_width {
        return width.max(0.0);
    }
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("IMG") {
        return super::html_image_element::layout_dimensions(scope, element)
            .map(|dimensions| f64::from(dimensions.0))
            .unwrap_or(0.0);
    }
    if tag.eq_ignore_ascii_case("HTML") {
        return super::window_view_state::inner_width(scope);
    }
    if tag.eq_ignore_ascii_case("BODY") {
        return (super::window_view_state::inner_width(scope) - 16.0).max(0.0);
    }
    if tag.eq_ignore_ascii_case("BUTTON") {
        return button_content_width(scope, element);
    }
    nearest_element_parent(scope, element)
        .map(|parent| shallow_content_width_without_descendants(scope, parent))
        .unwrap_or_else(|| super::window_view_state::inner_width(scope))
        .max(0.0)
}

fn shallow_content_width_without_descendants(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> f64 {
    let auto_width = calc_size_auto_content_width(scope, element, None);
    let Some(source) =
        super::get_computed_style_global::cascaded_property_source(scope, element, "width")
    else {
        return auto_width;
    };
    let (font_size, root_font_size) = font_sizes(scope, element);
    let declared = super::css_calculation::resolve_length(
        &source,
        super::css_calculation::EvaluationContext {
            viewport_width: super::window_view_state::inner_width(scope),
            viewport_height: super::window_view_state::inner_height(scope),
            percentage_basis: Some(auto_width),
            font_size,
            root_font_size,
            intrinsic_size: Some(auto_width),
        },
    )
    .unwrap_or(auto_width)
    .max(0.0);
    if !property(scope, element, "box-sizing").eq_ignore_ascii_case("border-box") {
        return declared;
    }
    let padding = side_lengths(scope, element, "padding", 0.0);
    let border = border_shorthand_width(&property(scope, element, "border"));
    let border_widths = if border == 0.0 {
        side_lengths(scope, element, "border-width", 0.0)
    } else {
        (border, border, border, border)
    };
    (declared - padding.1 - padding.3 - border_widths.1 - border_widths.3).max(0.0)
}

fn default_content_height(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    content_width: f64,
) -> f64 {
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    if tag.eq_ignore_ascii_case("HTML") {
        super::window_view_state::inner_height(scope)
    } else if tag.eq_ignore_ascii_case("IMG") {
        super::html_image_element::layout_dimensions(scope, element)
            .map(|dimensions| f64::from(dimensions.1))
            .unwrap_or(0.0)
    } else if tag.eq_ignore_ascii_case("BUTTON") {
        line_box_height(scope, element)
    } else {
        auto_flow_content_height(scope, element, content_width)
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
    if let Some(source) =
        super::get_computed_style_global::cascaded_property_source(scope, element, name)
        && let Some(length) = resolve_length(scope, element, name, &source)
    {
        return Some(length);
    }
    // A computed initial longhand (for example `margin-left: 0px`) must not
    // mask an actually cascaded shorthand (`margin: 8px`). Callers resolve
    // the shorthand separately and use this function only as its longhand
    // override, so absence from the cascade is deliberately `None` here.
    None
}

fn percentage_aware_property_length(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
    percentage_basis: Option<f64>,
) -> Option<f64> {
    if let Some(source) =
        super::get_computed_style_global::cascaded_property_source(scope, element, name)
        && let Some(length) =
            resolve_length_with_percentage_basis(scope, element, name, &source, percentage_basis)
    {
        return Some(length);
    }
    // See `property_length`: using the fully computed baseline here would
    // incorrectly outrank a cascaded shorthand and would also turn an absent
    // auto size into a concrete zero length.
    None
}

fn resolve_length(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    property_name: &str,
    value: &str,
) -> Option<f64> {
    resolve_length_with_percentage_basis(scope, element, property_name, value, None)
}

fn resolve_length_with_percentage_basis(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    property_name: &str,
    value: &str,
    explicit_percentage_basis: Option<f64>,
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
        explicit_percentage_basis.or_else(|| {
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
        })
    } else {
        None
    };
    let intrinsic_size = value.to_ascii_lowercase().contains("calc-size(").then(|| {
        if vertical {
            default_content_height(scope, element, default_content_width(scope, element, 0.0))
        } else {
            calc_size_auto_content_width(scope, element, explicit_percentage_basis)
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
            current = font_size_keyword(&source, parent_size)
                .or_else(|| parse_length(&source))
                .unwrap_or_else(|| {
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
    let style_name = format!("border-{side}-style");
    if let Some(style) =
        super::get_computed_style_global::cascaded_property_source(scope, element, &style_name)
        && style.eq_ignore_ascii_case("none")
    {
        return Some(0.0);
    }
    let width_name = format!("border-{side}-width");
    super::get_computed_style_global::cascaded_property_source(scope, element, &width_name)
        .and_then(|value| resolve_length(scope, element, &width_name, &value))
}
