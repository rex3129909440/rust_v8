use unicode_bidi::{BidiInfo, Level};
use unicode_linebreak::linebreaks;
use unicode_segmentation::UnicodeSegmentation;
use unicode_vo::Orientation;

#[derive(Clone, Copy)]
pub(crate) struct TextFragment {
    pub node_identity: i32,
    pub start_utf16: u32,
    pub end_utf16: u32,
    pub line: usize,
    pub rect: super::dom_rect_read_only::RectRecord,
    pub paint_left_overflow: f64,
    pub paint_right_overflow: f64,
    pub ruby_group: Option<i32>,
    pub ruby_annotation: bool,
    pub inline_element_rect: Option<super::dom_rect_read_only::RectRecord>,
    /// Preserved newlines are exposed by Blink as their own zero-width
    /// DOMRect at the end of the preceding line.  They must not be folded
    /// into an adjacent glyph run.
    pub separate_rect: bool,
}

#[derive(Clone, Default)]
pub(crate) struct InlineTextLayout {
    pub fragments: Vec<TextFragment>,
    pub line_rects: Vec<super::dom_rect_read_only::RectRecord>,
    pub paint_rects: Vec<super::dom_rect_read_only::RectRecord>,
    pub content_height: f64,
}

#[derive(Clone)]
struct RawGlyph {
    node_identity: i32,
    start_utf16: u32,
    end_utf16: u32,
    rendered: String,
    font: String,
    implicit_default_font: bool,
    spacing: f64,
    width: f64,
    paint_left_overflow: f64,
    paint_right_overflow: f64,
    line_height: f64,
    ink_height: f64,
    advance: f64,
    layout_advance: f64,
    font_size: f64,
    vertical: bool,
    naturally_upright: bool,
    force_upright: bool,
    explicit_sideways: bool,
    combine_group: Option<i32>,
    combine_leader: bool,
    combine_cross_offset: f64,
    combine_cross_extent: f64,
    ruby_group: Option<i32>,
    ruby_annotation: bool,
    ruby_position: String,
    ruby_inline_offset: f64,
    rtl: bool,
    binary_shaped: bool,
}

enum Token {
    Word(Vec<RawGlyph>),
    Space(Vec<RawGlyph>, bool),
    Break(RawGlyph),
}

#[derive(Clone)]
struct PlacedGlyph {
    glyph: RawGlyph,
    line: usize,
    x: f64,
    bidi_run_start: bool,
}

pub(crate) fn layout_for_element(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    content_width: f64,
    origin_x: f64,
    origin_y: f64,
) -> InlineTextLayout {
    if content_width <= 0.0 || !content_width.is_finite() {
        return InlineTextLayout::default();
    }
    let writing_mode =
        super::get_computed_style_global::computed_property_value(scope, element, "writing-mode")
            .to_ascii_lowercase();
    let vertical = matches!(writing_mode.as_str(), "vertical-rl" | "vertical-lr");
    let inline_extent = if vertical {
        pixel_value(&super::get_computed_style_global::computed_property_value(
            scope, element, "height",
        ))
        .filter(|height| *height > 0.0)
        .unwrap_or(content_width)
    } else {
        content_width
    };
    let mut tokens = Vec::new();
    let mut pending_space = Vec::new();
    let mut word = Vec::new();
    collect_tokens(
        scope,
        element,
        element,
        &mut tokens,
        &mut pending_space,
        &mut word,
    );
    flush_word(&mut tokens, &mut word);
    shape_inline_runs(scope, &mut tokens);
    prepare_ruby_tokens(&mut tokens);
    if vertical {
        prepare_vertical_tokens(&mut tokens);
        tokens = split_at_vertical_line_breaks(tokens);
    }

    let white_space =
        super::get_computed_style_global::computed_property_value(scope, element, "white-space")
            .to_ascii_lowercase();
    let wrapping = !matches!(white_space.as_str(), "nowrap" | "pre");
    let anywhere = matches!(
        super::get_computed_style_global::computed_property_value(scope, element, "overflow-wrap",)
            .to_ascii_lowercase()
            .as_str(),
        "anywhere" | "break-word"
    ) || super::get_computed_style_global::computed_property_value(
        scope,
        element,
        "word-break",
    )
    .eq_ignore_ascii_case("break-all");

    let mut placed = Vec::new();
    let mut line = 0_usize;
    let mut x = 0.0;
    let mut pending: Option<Vec<RawGlyph>> = None;
    for token in tokens {
        match token {
            Token::Break(glyph) => {
                pending = None;
                placed.push(PlacedGlyph {
                    glyph,
                    line,
                    x,
                    bidi_run_start: false,
                });
                line += 1;
                x = 0.0;
            }
            Token::Space(space, preserve) => {
                if preserve {
                    if let Some(collapsed) = pending.take()
                        && x > 0.0
                    {
                        place_glyphs(scope, &mut placed, collapsed, line, &mut x);
                    }
                    for glyph in space {
                        if wrapping && x > 0.0 && x + glyph.width > inline_extent {
                            line += 1;
                            x = 0.0;
                        }
                        place_glyphs(scope, &mut placed, vec![glyph], line, &mut x);
                    }
                } else {
                    pending = Some(space);
                }
            }
            Token::Word(word) => {
                let word_width = glyphs_width(scope, &word);
                let mut space_width = pending
                    .as_ref()
                    .map(|space| glyphs_width(scope, space))
                    .unwrap_or(0.0);
                if let (Some(left), Some(right)) = (
                    placed.last().filter(|placed| placed.line == line),
                    pending.as_ref().and_then(|space| space.first()),
                ) {
                    space_width += pair_adjustment(scope, &left.glyph, right);
                }
                if wrapping && x > 0.0 && x + space_width + word_width > inline_extent {
                    line += 1;
                    x = 0.0;
                    pending = None;
                } else if let Some(space) = pending.take()
                    && x > 0.0
                {
                    place_glyphs(scope, &mut placed, space, line, &mut x);
                }
                if wrapping && anywhere && word_width > inline_extent {
                    for glyph in word {
                        let adjustment = placed
                            .last()
                            .filter(|placed| placed.line == line)
                            .map(|left| pair_adjustment(scope, &left.glyph, &glyph))
                            .unwrap_or(0.0);
                        if x > 0.0 && x + adjustment + glyph.width > inline_extent {
                            line += 1;
                            x = 0.0;
                        }
                        place_glyphs(scope, &mut placed, vec![glyph], line, &mut x);
                    }
                } else {
                    place_glyphs(scope, &mut placed, word, line, &mut x);
                }
            }
        }
    }
    if placed.is_empty() {
        return InlineTextLayout::default();
    }

    apply_horizontal_alignment(scope, element, content_width, &mut placed);

    let line_count = placed.iter().map(|glyph| glyph.line).max().unwrap_or(0) + 1;
    let mut line_heights = vec![0.0_f64; line_count];
    for glyph in &placed {
        let cross_advance = if vertical {
            vertical_cross_advance(&glyph.glyph)
        } else {
            glyph.glyph.line_height
        };
        line_heights[glyph.line] = line_heights[glyph.line].max(cross_advance);
    }
    let fallback_line_height = super::element_layout::line_box_height(scope, element);
    for height in &mut line_heights {
        if *height <= 0.0 {
            *height = fallback_line_height;
        }
    }
    let mut line_offsets = vec![0.0_f64; line_count];
    for index in 1..line_count {
        line_offsets[index] = line_offsets[index - 1] + line_heights[index - 1];
    }
    let mut line_has_naturally_upright = vec![false; line_count];
    for glyph in &placed {
        line_has_naturally_upright[glyph.line] |= glyph.glyph.naturally_upright;
    }

    let mut ruby_metrics = std::collections::HashMap::<i32, (f64, f64, f64, f64, String)>::new();
    for item in &placed {
        let Some(group) = item.glyph.ruby_group else {
            continue;
        };
        let entry = ruby_metrics.entry(group).or_insert((
            0.0,
            0.0,
            0.0,
            0.0,
            item.glyph.ruby_position.clone(),
        ));
        if item.glyph.ruby_annotation {
            entry.2 = entry.2.max(item.glyph.line_height);
            entry.3 += item.glyph.advance;
        } else {
            entry.0 = entry.0.max(item.glyph.font_size);
            entry.1 = entry.1.max(item.glyph.ink_height);
        }
    }
    let mut fragments = Vec::with_capacity(placed.len());
    for placed in placed {
        let line_height = line_heights[placed.line];
        let ink_height = placed.glyph.ink_height.min(line_height).max(0.0);
        let inset = ((line_height - ink_height) / 2.0).floor().max(0.0);
        let (raw_x, raw_y, raw_width, raw_height) = if vertical {
            let cross_advance = line_heights[placed.line];
            let natural_inset = if line_has_naturally_upright[placed.line] {
                (cross_advance - ink_height).max(0.0) / 2.0
            } else if writing_mode == "vertical-lr" {
                vertical_latin_lr_inset(&placed.glyph)
            } else {
                (cross_advance - ink_height).max(0.0) / 2.0
            };
            let mut line_x = if writing_mode == "vertical-rl" {
                origin_x + content_width - line_offsets[placed.line] - natural_inset - ink_height
            } else {
                origin_x + line_offsets[placed.line] + natural_inset
            };
            if placed.glyph.ruby_annotation {
                let (base_font_size, base_ink, annotation_extent, _, position) = placed
                    .glyph
                    .ruby_group
                    .and_then(|group| ruby_metrics.get(&group))
                    .cloned()
                    .unwrap_or((
                        placed.glyph.font_size,
                        ink_height,
                        ink_height,
                        placed.glyph.advance,
                        "over".to_owned(),
                    ));
                let base_leading = (line_height - base_ink).max(0.0) / 2.0;
                let mut base_x = if writing_mode == "vertical-rl" {
                    origin_x + content_width - line_offsets[placed.line] - base_leading - base_ink
                } else {
                    origin_x + line_offsets[placed.line] + base_leading
                };
                let ruby_x = if position.eq_ignore_ascii_case("under") {
                    let inner_leading =
                        (placed.glyph.line_height - placed.glyph.font_size).max(0.0);
                    base_x - annotation_extent + inner_leading + 0.5
                } else {
                    base_x + base_font_size + 0.5
                };
                (
                    ruby_x,
                    origin_y + placed.x + placed.glyph.ruby_inline_offset,
                    annotation_extent,
                    placed.glyph.advance,
                )
            } else if placed.glyph.combine_group.is_some() {
                (
                    line_x + placed.glyph.combine_cross_offset,
                    origin_y + placed.x,
                    placed.glyph.combine_cross_extent,
                    placed.glyph.font_size,
                )
            } else {
                (
                    line_x,
                    origin_y + placed.x,
                    ink_height,
                    placed.glyph.advance,
                )
            }
        } else if let Some(group) = placed.glyph.ruby_group
            && let Some((base_font_size, _, _, annotation_advance, _)) = ruby_metrics.get(&group)
        {
            if placed.glyph.ruby_annotation {
                let annotation_height = placed.glyph.line_height;
                let annotation_cross_inset =
                    ((annotation_height - placed.glyph.ink_height).max(0.0) / 2.0).floor();
                if placed.glyph.vertical {
                    (
                        origin_x + placed.x,
                        origin_y - placed.glyph.font_size * 0.05 + placed.glyph.ruby_inline_offset,
                        *base_font_size,
                        placed.glyph.advance,
                    )
                } else {
                    (
                        origin_x + placed.x + placed.glyph.ruby_inline_offset,
                        origin_y - placed.glyph.font_size * 0.1 + annotation_cross_inset,
                        placed.glyph.advance,
                        placed.glyph.ink_height,
                    )
                }
            } else {
                let vertical_ruby = placed.glyph.vertical;
                (
                    origin_x + placed.x,
                    origin_y + placed.glyph.font_size * if vertical_ruby { 0.45 } else { 0.4 },
                    if vertical_ruby {
                        *base_font_size
                    } else {
                        placed.glyph.width
                    },
                    ink_height,
                )
            }
        } else {
            (
                origin_x + placed.x,
                origin_y + line_offsets[placed.line] + inset,
                placed.glyph.width,
                ink_height,
            )
        };
        let left = quantize(raw_x);
        let top = quantize(raw_y);
        let right = if raw_width == 0.0 {
            left
        } else if placed.glyph.combine_group.is_some() {
            quantize(raw_x + raw_width)
        } else if vertical {
            quantize_end(raw_x + raw_width)
        } else {
            quantize_glyph_end(raw_x + raw_width, &placed.glyph.font)
        };
        let bottom = if raw_height == 0.0 {
            top
        } else if vertical
            && !placed.glyph.naturally_upright
            && !placed.glyph.force_upright
            && placed.glyph.combine_group.is_none()
        {
            quantize_vertical_glyph_end(raw_y + raw_height)
        } else {
            quantize_end(raw_y + raw_height)
        };
        let rect = super::dom_rect_read_only::RectRecord {
            x: left,
            y: top,
            width: (right - left).max(0.0),
            height: (bottom - top).max(0.0),
        };
        let inline_element_rect = placed.glyph.ruby_group.and_then(|group| {
            let (base_font_size, _, annotation_extent, annotation_advance, _) =
                ruby_metrics.get(&group)?;
            if placed.glyph.ruby_annotation {
                if vertical {
                    Some(super::dom_rect_read_only::RectRecord {
                        x: rect.x,
                        y: origin_y + placed.x,
                        width: *annotation_extent,
                        height: *annotation_advance,
                    })
                } else if placed.glyph.vertical {
                    Some(super::dom_rect_read_only::RectRecord {
                        x: origin_x + placed.x,
                        y: origin_y - placed.glyph.font_size * 0.05,
                        width: *base_font_size,
                        height: *annotation_extent,
                    })
                } else {
                    Some(super::dom_rect_read_only::RectRecord {
                        x: origin_x + placed.x,
                        y: origin_y - placed.glyph.font_size * 0.1,
                        width: *annotation_advance,
                        height: *annotation_extent,
                    })
                }
            } else if !vertical && placed.glyph.vertical {
                Some(super::dom_rect_read_only::RectRecord {
                    x: rect.x,
                    y: rect.y,
                    width: rect.height,
                    height: rect.width,
                })
            } else {
                Some(rect)
            }
        });
        fragments.push(TextFragment {
            node_identity: placed.glyph.node_identity,
            start_utf16: placed.glyph.start_utf16,
            end_utf16: placed.glyph.end_utf16,
            line: placed.line,
            rect,
            paint_left_overflow: placed.glyph.paint_left_overflow,
            paint_right_overflow: placed.glyph.paint_right_overflow,
            ruby_group: placed.glyph.ruby_group,
            ruby_annotation: placed.glyph.ruby_annotation,
            inline_element_rect,
            separate_rect: placed.glyph.rendered == "\n" || placed.bidi_run_start,
        });
    }
    align_ruby_lines(
        &mut fragments,
        &ruby_metrics,
        vertical,
        &writing_mode,
        origin_x,
        content_width,
    );
    let line_rects = consolidate(&fragments, |fragment| !fragment.separate_rect);
    let paint_rects = consolidate_paint(&fragments);
    let content_height = if vertical {
        fragments
            .iter()
            .map(|fragment| fragment.rect.y + fragment.rect.height - origin_y)
            .fold(0.0, f64::max)
    } else {
        line_heights.iter().sum()
    };
    InlineTextLayout {
        fragments,
        line_rects,
        paint_rects,
        content_height,
    }
}

fn align_ruby_lines(
    fragments: &mut [TextFragment],
    ruby_metrics: &std::collections::HashMap<i32, (f64, f64, f64, f64, String)>,
    vertical: bool,
    writing_mode: &str,
    origin_x: f64,
    content_width: f64,
) {
    let line_count = fragments
        .iter()
        .map(|fragment| fragment.line)
        .max()
        .unwrap_or(0)
        + 1;
    for line in 0..line_count {
        if vertical {
            let mut shift: f64 = 0.0;
            for fragment in fragments
                .iter()
                .filter(|fragment| fragment.line == line && fragment.ruby_annotation)
            {
                let Some(group) = fragment.ruby_group else {
                    continue;
                };
                let Some((_, _, _, _, position)) = ruby_metrics.get(&group) else {
                    continue;
                };
                if writing_mode == "vertical-rl" && position.eq_ignore_ascii_case("over") {
                    let target_right = origin_x + content_width + 0.5;
                    shift = shift.min(target_right - (fragment.rect.x + fragment.rect.width));
                } else if writing_mode == "vertical-lr" && position.eq_ignore_ascii_case("under") {
                    let target_left = origin_x - 0.5;
                    shift = shift.max(target_left - fragment.rect.x);
                }
            }
            if shift != 0.0 {
                for fragment in fragments
                    .iter_mut()
                    .filter(|fragment| fragment.line == line)
                {
                    fragment.rect.x += shift;
                    if let Some(rect) = fragment.inline_element_rect.as_mut() {
                        rect.x += shift;
                    }
                }
            }
        } else {
            let target_top = fragments
                .iter()
                .filter(|fragment| {
                    fragment.line == line
                        && fragment.ruby_group.is_some()
                        && !fragment.ruby_annotation
                })
                .map(|fragment| fragment.rect.y)
                .reduce(f64::min);
            if let Some(target_top) = target_top {
                for fragment in fragments
                    .iter_mut()
                    .filter(|fragment| fragment.line == line && fragment.ruby_group.is_none())
                {
                    fragment.rect.y = target_top;
                    if let Some(rect) = fragment.inline_element_rect.as_mut() {
                        rect.y = target_top;
                    }
                }
            }
        }
    }
}

fn apply_horizontal_alignment(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    content_width: f64,
    placed: &mut [PlacedGlyph],
) {
    let writing_mode =
        super::get_computed_style_global::computed_property_value(scope, element, "writing-mode");
    if !writing_mode.is_empty() && !writing_mode.eq_ignore_ascii_case("horizontal-tb") {
        return;
    }
    let direction =
        super::get_computed_style_global::computed_property_value(scope, element, "direction");
    let right_to_left = direction.eq_ignore_ascii_case("rtl");
    let unicode_bidi =
        super::get_computed_style_global::computed_property_value(scope, element, "unicode-bidi");
    let line_count = placed.iter().map(|glyph| glyph.line).max().unwrap_or(0) + 1;
    let mut widths = vec![0.0_f64; line_count];
    for glyph in placed.iter() {
        widths[glyph.line] = widths[glyph.line].max(glyph.x + glyph.glyph.width);
    }
    if right_to_left && unicode_bidi.eq_ignore_ascii_case("bidi-override") {
        for glyph in placed.iter_mut() {
            glyph.x = (widths[glyph.line] - glyph.x - glyph.glyph.width).max(0.0);
        }
    } else {
        apply_bidi_reordering(placed, right_to_left);
        widths.fill(0.0);
        for glyph in placed.iter() {
            widths[glyph.line] = widths[glyph.line].max(glyph.x + glyph.glyph.width);
        }
    }
    let text_align =
        super::get_computed_style_global::computed_property_value(scope, element, "text-align")
            .to_ascii_lowercase();
    for glyph in placed.iter_mut() {
        let free = (content_width - widths[glyph.line]).max(0.0);
        let offset = match text_align.as_str() {
            // Blink resolves centered inline offsets in layout units and
            // assigns the odd 1/64 remainder to the inline-end side.
            "center" => (free * 32.0).floor() / 64.0,
            "right" => free,
            "left" => 0.0,
            "end" if !right_to_left => free,
            "end" => 0.0,
            "start" if right_to_left => free,
            "start" => 0.0,
            _ if right_to_left => free,
            _ => 0.0,
        };
        glyph.x += offset;
    }
}

fn apply_bidi_reordering(placed: &mut [PlacedGlyph], right_to_left: bool) {
    let mut start = 0_usize;
    while start < placed.len() {
        let line = placed[start].line;
        let mut end = start + 1;
        while end < placed.len() && placed[end].line == line {
            end += 1;
        }
        let line_glyphs = &placed[start..end];
        let mut text = String::new();
        let mut char_offsets = Vec::with_capacity(line_glyphs.len());
        let mut char_offset = 0_usize;
        for glyph in line_glyphs {
            char_offsets.push(char_offset);
            text.push_str(&glyph.glyph.rendered);
            char_offset += glyph.glyph.rendered.chars().count();
        }
        let base_level = if right_to_left {
            Level::rtl()
        } else {
            Level::ltr()
        };
        let bidi = BidiInfo::new(&text, Some(base_level));
        let Some(paragraph) = bidi.paragraphs.first() else {
            start = end;
            continue;
        };
        let char_levels = bidi.reordered_levels_per_char(paragraph, paragraph.range.clone());
        let glyph_levels = char_offsets
            .iter()
            .map(|offset| char_levels.get(*offset).copied().unwrap_or(base_level))
            .collect::<Vec<_>>();
        let order = BidiInfo::reorder_visual(&glyph_levels);
        if order
            .iter()
            .enumerate()
            .all(|(visual, logical)| visual == *logical)
        {
            start = end;
            continue;
        }
        let mut cursor = 0.0_f64;
        let mut reordered = Vec::with_capacity(line_glyphs.len());
        let mut visual = 0_usize;
        while visual < order.len() {
            let level = glyph_levels[order[visual]];
            let mut run_end = visual + 1;
            while run_end < order.len() && glyph_levels[order[run_end]] == level {
                run_end += 1;
            }
            let (run_left, run_right) = order[visual..run_end].iter().fold(
                (f64::INFINITY, f64::NEG_INFINITY),
                |(left, right), logical| {
                    let glyph = &line_glyphs[*logical];
                    (left.min(glyph.x), right.max(glyph.x + glyph.glyph.width))
                },
            );
            for (run_offset, logical) in order[visual..run_end].iter().enumerate() {
                let mut glyph = line_glyphs[*logical].clone();
                let relative = if level.is_rtl() {
                    run_right - glyph.x - glyph.glyph.width
                } else {
                    glyph.x - run_left
                };
                // Blink stores glyph origins relative to each bidi run as
                // LayoutUnits.  Quantizing only the final, globally aligned
                // DOMRect loses the error distribution inside repeated-glyph
                // runs (for example the middle `2` in an RTL `123` run is
                // otherwise one 1/64px unit too far right).
                glyph.x = cursor + quantize(relative.max(0.0));
                glyph.bidi_run_start = visual > 0 && run_offset == 0;
                reordered.push(glyph);
            }
            cursor += (run_right - run_left).max(0.0);
            visual = run_end;
        }
        placed[start..end].clone_from_slice(&reordered);
        start = end;
    }
}

pub(crate) fn selection_rects(
    layout: &InlineTextLayout,
    node_identity: i32,
    start_utf16: u32,
    end_utf16: u32,
) -> Vec<super::dom_rect_read_only::RectRecord> {
    let selected = layout
        .fragments
        .iter()
        .filter(|fragment| {
            fragment.node_identity == node_identity
                && fragment.start_utf16 < end_utf16
                && fragment.end_utf16 > start_utf16
        })
        .collect::<Vec<_>>();
    if selected.len() > 1
        && selected.iter().all(|fragment| {
            fragment.ruby_annotation
                && fragment.ruby_group == selected[0].ruby_group
                && fragment.inline_element_rect.is_some()
        })
    {
        return vec![selected[0].inline_element_rect.unwrap()];
    }
    consolidate(&layout.fragments, |fragment| {
        fragment.node_identity == node_identity
            && fragment.start_utf16 < end_utf16
            && fragment.end_utf16 > start_utf16
    })
}

pub(crate) fn node_set_rects(
    layout: &InlineTextLayout,
    node_identities: &[i32],
) -> Vec<super::dom_rect_read_only::RectRecord> {
    consolidate(&layout.fragments, |fragment| {
        node_identities.contains(&fragment.node_identity)
    })
}

pub(crate) fn inline_element_rects(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Vec<super::dom_rect_read_only::RectRecord> {
    let mut text_identities = Vec::new();
    collect_text_identities(scope, element, &mut text_identities);
    if text_identities.is_empty() {
        return Vec::new();
    }
    let Some(container) = containing_inline_box(scope, element) else {
        return Vec::new();
    };
    let container_layout = super::element_layout::compute(scope, container);
    if !container_layout.rendered || container_layout.content_width <= 0.0 {
        return Vec::new();
    }
    let scroll = super::element::record(scope, container)
        .map(|record| (record.scroll_left, record.scroll_top))
        .unwrap_or_default();
    let layout = layout_for_element(
        scope,
        container,
        container_layout.content_width,
        container_layout.x + container_layout.border_left + container_layout.padding_left
            - scroll.0,
        container_layout.y + container_layout.border_top + container_layout.padding_top - scroll.1,
    );
    let tag = super::element::record(scope, element)
        .map(|record| record.tag_name)
        .unwrap_or_default();
    let ruby_annotation = tag.eq_ignore_ascii_case("RT");
    let ruby_base = tag.eq_ignore_ascii_case("RUBY");
    if ruby_annotation || ruby_base {
        let mut output = Vec::new();
        for fragment in layout.fragments.iter().filter(|fragment| {
            text_identities.contains(&fragment.node_identity)
                && fragment.ruby_group.is_some()
                && fragment.ruby_annotation == ruby_annotation
        }) {
            let Some(rect) = fragment.inline_element_rect else {
                continue;
            };
            if !output
                .iter()
                .any(|candidate: &super::dom_rect_read_only::RectRecord| {
                    candidate.x == rect.x
                        && candidate.y == rect.y
                        && candidate.width == rect.width
                        && candidate.height == rect.height
                })
            {
                output.push(rect);
            }
        }
        if !output.is_empty() {
            return output;
        }
    }
    node_set_rects(&layout, &text_identities)
}

/// Whether an element exposes line-box fragments for its own CSS box.
///
/// `is_block_level()` is intentionally not used here: atomic inline-level
/// boxes such as `inline-block`, replaced controls, inline flex/grid and
/// inline tables participate in an inline formatting context, but their
/// client rect is still the element border box. Only non-atomic inline/ruby
/// boxes expose the text-fragment geometry assembled by this module.
pub(crate) fn uses_inline_fragment_geometry(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> bool {
    matches!(
        super::get_computed_style_global::computed_property_value(scope, element, "display")
            .to_ascii_lowercase()
            .as_str(),
        "inline"
            | "ruby"
            | "ruby-text"
            | "ruby-base"
            | "ruby-base-container"
            | "ruby-text-container"
    )
}

fn collect_text_identities(
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
            collect_text_identities(scope, child, output);
        }
    }
}

pub(crate) fn containing_inline_box<'s>(
    scope: &v8::PinScope<'s, '_>,
    text: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut current = super::node::parent(scope, text);
    let mut nearest = None;
    while let Some(candidate) = current {
        if super::element::record(scope, candidate).is_some() {
            nearest = Some(candidate);
            if super::element_layout::is_block_level(scope, candidate) {
                return Some(candidate);
            }
        }
        current = super::node::parent(scope, candidate);
    }
    nearest
}

fn collect_tokens(
    scope: &v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
    node: v8::Local<'_, v8::Object>,
    tokens: &mut Vec<Token>,
    pending_space: &mut Vec<RawGlyph>,
    word: &mut Vec<RawGlyph>,
) {
    for child in super::node::children(scope, node) {
        let Some(node_record) = super::node::record(scope, child) else {
            continue;
        };
        if node_record.node_type == super::node::TEXT_NODE {
            let style_element = super::node::parent(scope, child)
                .filter(|parent| super::element::record(scope, *parent).is_some())
                .unwrap_or(root);
            append_text(
                scope,
                child,
                style_element,
                node_record.node_value.as_deref().unwrap_or(""),
                tokens,
                pending_space,
                word,
            );
            continue;
        }
        if super::element::record(scope, child).is_none() {
            continue;
        }
        let display =
            super::get_computed_style_global::computed_property_value(scope, child, "display");
        if display.eq_ignore_ascii_case("none") {
            continue;
        }
        if super::element_layout::is_block_level(scope, child) {
            flush_word(tokens, word);
            pending_space.clear();
            continue;
        }
        collect_tokens(scope, root, child, tokens, pending_space, word);
    }
}

fn append_text(
    scope: &v8::PinScope<'_, '_>,
    text_node: v8::Local<'_, v8::Object>,
    style_element: v8::Local<'_, v8::Object>,
    text: &str,
    tokens: &mut Vec<Token>,
    pending_space: &mut Vec<RawGlyph>,
    word: &mut Vec<RawGlyph>,
) {
    let white_space = super::get_computed_style_global::computed_property_value(
        scope,
        style_element,
        "white-space",
    )
    .to_ascii_lowercase();
    let preserve_spaces = matches!(white_space.as_str(), "pre" | "pre-wrap" | "break-spaces");
    let preserve_newlines = preserve_spaces || white_space == "pre-line";
    let font =
        super::get_computed_style_global::computed_property_value(scope, style_element, "font");
    let implicit_default_font =
        super::element_layout::uses_implicit_default_font(scope, style_element);
    let line_height = super::element_layout::line_box_height(scope, style_element);
    let fallback_ink_height = text_ink_height(scope, style_element, line_height);
    let letter_spacing = pixel_value(&super::get_computed_style_global::computed_property_value(
        scope,
        style_element,
        "letter-spacing",
    ))
    .unwrap_or(0.0);
    let word_spacing = pixel_value(&super::get_computed_style_global::computed_property_value(
        scope,
        style_element,
        "word-spacing",
    ))
    .unwrap_or(0.0);
    let transform = super::get_computed_style_global::computed_property_value(
        scope,
        style_element,
        "text-transform",
    )
    .to_ascii_lowercase();
    let mut utf16_offset = 0_u32;
    for grapheme in text.graphemes(true) {
        let start = utf16_offset;
        utf16_offset += grapheme.encode_utf16().count() as u32;
        if matches!(grapheme, "\n" | "\r" | "\r\n") && preserve_newlines {
            flush_word(tokens, word);
            pending_space.clear();
            let mut marker = glyph(
                scope,
                text_node,
                start,
                utf16_offset,
                "\n".to_owned(),
                &font,
                implicit_default_font,
                0.0,
                line_height,
                fallback_ink_height,
            );
            marker.width = 0.0;
            marker.paint_left_overflow = 0.0;
            marker.paint_right_overflow = 0.0;
            tokens.push(Token::Break(marker));
            continue;
        }
        if grapheme.chars().all(char::is_whitespace) {
            flush_word(tokens, word);
            let rendered = if grapheme == "\t" && preserve_spaces {
                "        ".to_owned()
            } else {
                " ".to_owned()
            };
            let glyph = glyph(
                scope,
                text_node,
                start,
                utf16_offset,
                rendered,
                &font,
                implicit_default_font,
                letter_spacing + word_spacing,
                line_height,
                fallback_ink_height,
            );
            if preserve_spaces {
                flush_space(tokens, pending_space);
                tokens.push(Token::Space(vec![glyph], true));
            } else if pending_space.is_empty() {
                pending_space.push(glyph);
            } else if pending_space[0].node_identity == text_node.get_identity_hash().get() {
                pending_space[0].end_utf16 = utf16_offset;
            }
            continue;
        }
        flush_space(tokens, pending_space);
        let rendered = match transform.as_str() {
            "uppercase" => grapheme.chars().flat_map(char::to_uppercase).collect(),
            "lowercase" => grapheme.chars().flat_map(char::to_lowercase).collect(),
            _ => grapheme.to_owned(),
        };
        word.push(glyph(
            scope,
            text_node,
            start,
            utf16_offset,
            rendered,
            &font,
            implicit_default_font,
            letter_spacing,
            line_height,
            fallback_ink_height,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn glyph(
    scope: &v8::PinScope<'_, '_>,
    text_node: v8::Local<'_, v8::Object>,
    start_utf16: u32,
    end_utf16: u32,
    rendered: String,
    font: &str,
    implicit_default_font: bool,
    spacing: f64,
    line_height: f64,
    ink_height: f64,
) -> RawGlyph {
    let direction = super::node::parent(scope, text_node)
        .filter(|parent| super::element::record(scope, *parent).is_some())
        .map(|parent| {
            super::get_computed_style_global::computed_property_value(scope, parent, "direction")
        })
        .unwrap_or_else(|| "ltr".to_owned());
    let rtl = direction.eq_ignore_ascii_case("rtl");
    let shaped = super::offscreen_canvas_rendering_context_2d::shaped_font_metrics(
        scope, &rendered, font, rtl,
    );
    let width = shaped.map(|metrics| metrics.advance).unwrap_or_else(|| {
        super::offscreen_canvas_rendering_context_2d::measured_inline_text_width_for_font(
            scope,
            &rendered,
            font,
            implicit_default_font,
        )
    }) + spacing;
    let font_size = super::offscreen_canvas_rendering_context_2d::canvas_font_size(font);
    let writing_mode = super::node::parent(scope, text_node)
        .filter(|parent| super::element::record(scope, *parent).is_some())
        .map(|parent| {
            super::get_computed_style_global::computed_property_value(scope, parent, "writing-mode")
        })
        .unwrap_or_default()
        .to_ascii_lowercase();
    let vertical = matches!(writing_mode.as_str(), "vertical-rl" | "vertical-lr");
    let text_orientation = super::node::parent(scope, text_node)
        .filter(|parent| super::element::record(scope, *parent).is_some())
        .map(|parent| {
            super::get_computed_style_global::computed_property_value(
                scope,
                parent,
                "text-orientation",
            )
        })
        .unwrap_or_else(|| "mixed".to_owned())
        .to_ascii_lowercase();
    let naturally_upright = rendered.chars().all(|character| {
        !matches!(
            unicode_vo::char_orientation(character),
            Orientation::Rotated
        )
    });
    let force_upright = vertical && text_orientation == "upright";
    let explicit_sideways = vertical && text_orientation == "sideways";
    let combine_group = if vertical {
        super::node::parent(scope, text_node)
            .filter(|parent| super::element::record(scope, *parent).is_some())
            .filter(|parent| {
                super::get_computed_style_global::computed_property_value(
                    scope,
                    *parent,
                    "text-combine-upright",
                )
                .eq_ignore_ascii_case("all")
            })
            .map(|parent| parent.get_identity_hash().get())
    } else {
        None
    };
    let ink_height = shaped
        .map(|metrics| metrics.actual_ascent + metrics.actual_descent)
        .filter(|height| *height > 0.0)
        .unwrap_or(ink_height);
    let (ruby_group, ruby_annotation, ruby_position) = ruby_context(scope, text_node);
    let advance = if vertical && force_upright && !naturally_upright {
        ink_height
    } else {
        width.max(0.0)
    };
    let (paint_left_overflow, paint_right_overflow) = shaped
        .map(|metrics| {
            (
                metrics.actual_left.max(0.0),
                (metrics.actual_right - metrics.advance).max(0.0),
            )
        })
        .unwrap_or_else(|| {
            rendered
                .chars()
                .next()
                .map(|character| {
                    super::font_metric_tables::paint_overflow(
                        font,
                        character,
                        super::offscreen_canvas_rendering_context_2d::canvas_font_size(font),
                        implicit_default_font,
                        width.max(0.0),
                    )
                })
                .unwrap_or_default()
        });
    RawGlyph {
        node_identity: text_node.get_identity_hash().get(),
        start_utf16,
        end_utf16,
        rendered,
        font: font.to_owned(),
        implicit_default_font,
        spacing,
        width: width.max(0.0),
        paint_left_overflow,
        paint_right_overflow,
        line_height,
        ink_height,
        advance,
        layout_advance: advance,
        font_size,
        vertical,
        naturally_upright,
        force_upright,
        explicit_sideways,
        combine_group,
        combine_leader: false,
        combine_cross_offset: 0.0,
        combine_cross_extent: 0.0,
        ruby_group,
        ruby_annotation,
        ruby_position,
        ruby_inline_offset: 0.0,
        rtl,
        binary_shaped: shaped.is_some(),
    }
}

fn shape_inline_runs(scope: &v8::PinScope<'_, '_>, tokens: &mut [Token]) {
    for token in tokens {
        let glyphs = match token {
            Token::Word(glyphs) | Token::Space(glyphs, _) => glyphs,
            Token::Break(_) => continue,
        };
        let Some(first) = glyphs.first() else {
            continue;
        };
        if glyphs.iter().any(|glyph| {
            glyph.font != first.font
                || glyph.implicit_default_font != first.implicit_default_font
                || glyph.rtl != first.rtl
        }) {
            continue;
        }
        let graphemes = glyphs
            .iter()
            .map(|glyph| glyph.rendered.as_str())
            .collect::<Vec<_>>();
        let Some(metrics) = crate::font_shaping::grapheme_metrics(
            scope,
            &graphemes,
            &first.font,
            if first.rtl {
                rustybuzz::Direction::RightToLeft
            } else {
                rustybuzz::Direction::LeftToRight
            },
        ) else {
            continue;
        };
        for (glyph, metrics) in glyphs.iter_mut().zip(metrics) {
            glyph.width = (metrics.advance + glyph.spacing).max(0.0);
            if !glyph.vertical {
                glyph.advance = glyph.width;
                glyph.layout_advance = glyph.width;
            }
            glyph.binary_shaped = true;
        }
    }
}

fn ruby_context(
    scope: &v8::PinScope<'_, '_>,
    text_node: v8::Local<'_, v8::Object>,
) -> (Option<i32>, bool, String) {
    let mut current = super::node::parent(scope, text_node);
    let mut annotation = false;
    while let Some(element) = current {
        let Some(record) = super::element::record(scope, element) else {
            current = super::node::parent(scope, element);
            continue;
        };
        if record.tag_name.eq_ignore_ascii_case("RT") {
            annotation = true;
        }
        if record.tag_name.eq_ignore_ascii_case("RUBY") {
            let position = super::get_computed_style_global::computed_property_value(
                scope,
                element,
                "ruby-position",
            );
            return (
                Some(element.get_identity_hash().get()),
                annotation,
                position,
            );
        }
        if super::element_layout::is_block_level(scope, element) {
            break;
        }
        current = super::node::parent(scope, element);
    }
    (None, false, String::new())
}

fn prepare_vertical_tokens(tokens: &mut [Token]) {
    let mut group_widths = std::collections::HashMap::<i32, (f64, f64)>::new();
    for token in tokens.iter() {
        let glyphs = match token {
            Token::Word(glyphs) | Token::Space(glyphs, _) => glyphs.as_slice(),
            Token::Break(glyph) => std::slice::from_ref(glyph),
        };
        for glyph in glyphs {
            if let Some(group) = glyph.combine_group {
                let entry = group_widths.entry(group).or_insert((0.0, glyph.ink_height));
                entry.0 += glyph.width;
                entry.1 = entry.1.min(glyph.ink_height);
            }
        }
    }
    let mut group_offsets = std::collections::HashMap::<i32, f64>::new();
    for token in tokens {
        let glyphs = match token {
            Token::Word(glyphs) | Token::Space(glyphs, _) => glyphs,
            Token::Break(glyph) => std::slice::from_mut(glyph),
        };
        for glyph in glyphs {
            if let Some(group) = glyph.combine_group {
                let (source_width, target_width) = group_widths[&group];
                let offset = group_offsets.entry(group).or_default();
                let scale = if source_width > target_width && source_width > 0.0 {
                    target_width / source_width
                } else {
                    1.0
                };
                glyph.combine_leader = *offset == 0.0;
                let raw_start = *offset * scale;
                let raw_end = (*offset + glyph.width) * scale;
                let start = if *offset == 0.0 {
                    0.0
                } else {
                    (raw_start * 64.0 - 1e-9).floor() / 64.0
                };
                let end = (raw_end * 64.0).round() / 64.0;
                glyph.combine_cross_offset = start;
                glyph.combine_cross_extent = (end - start).max(0.0);
                glyph.advance = glyph.font_size;
                glyph.layout_advance = if glyph.combine_leader {
                    glyph.font_size
                } else {
                    0.0
                };
                *offset += glyph.width;
            }
        }
    }
}

fn prepare_ruby_tokens(tokens: &mut [Token]) {
    let mut annotation_offsets = std::collections::HashMap::<i32, f64>::new();
    for token in tokens {
        let glyphs = match token {
            Token::Word(glyphs) | Token::Space(glyphs, _) => glyphs,
            Token::Break(glyph) => std::slice::from_mut(glyph),
        };
        for glyph in glyphs {
            let Some(group) = glyph.ruby_group else {
                continue;
            };
            if glyph.ruby_annotation {
                let offset = annotation_offsets.entry(group).or_default();
                glyph.ruby_inline_offset = *offset;
                glyph.layout_advance = 0.0;
                *offset += glyph.advance;
            }
        }
    }
}

fn split_at_vertical_line_breaks(tokens: Vec<Token>) -> Vec<Token> {
    let mut output = Vec::new();
    for token in tokens {
        let Token::Word(glyphs) = token else {
            output.push(token);
            continue;
        };
        if glyphs.len() < 2 || glyphs.iter().any(|glyph| glyph.combine_group.is_some()) {
            output.push(Token::Word(glyphs));
            continue;
        }
        let text = glyphs
            .iter()
            .map(|glyph| glyph.rendered.as_str())
            .collect::<String>();
        let breaks = linebreaks(&text)
            .map(|(offset, _)| offset)
            .collect::<std::collections::HashSet<_>>();
        let mut chunk = Vec::new();
        let mut byte_offset = 0_usize;
        let glyph_count = glyphs.len();
        for (index, glyph) in glyphs.into_iter().enumerate() {
            byte_offset += glyph.rendered.len();
            chunk.push(glyph);
            if index + 1 < glyph_count && breaks.contains(&byte_offset) {
                output.push(Token::Word(std::mem::take(&mut chunk)));
            }
        }
        if !chunk.is_empty() {
            output.push(Token::Word(chunk));
        }
    }
    output
}

fn vertical_cross_advance(glyph: &RawGlyph) -> f64 {
    if glyph.explicit_sideways {
        glyph.ink_height + 2.0 * (glyph.font_size * 0.125).round()
    } else if glyph.naturally_upright || glyph.combine_group.is_some() {
        (glyph.font_size * 1.3).round().max(glyph.line_height)
    } else {
        glyph.ink_height
    }
}

fn vertical_latin_lr_inset(glyph: &RawGlyph) -> f64 {
    let family = glyph.font.to_ascii_lowercase();
    if family.contains("segoe ui") {
        return 0.0;
    }
    if family.contains("times new roman") {
        if glyph.font_size >= 64.0 {
            2.0
        } else if glyph.font_size >= 12.0 {
            1.0
        } else {
            0.0
        }
    } else if family.contains("arial") {
        if glyph.font_size >= 16.0 { 1.0 } else { 0.0 }
    } else {
        0.0
    }
}

fn place_glyphs(
    scope: &v8::PinScope<'_, '_>,
    placed: &mut Vec<PlacedGlyph>,
    glyphs: Vec<RawGlyph>,
    line: usize,
    x: &mut f64,
) {
    for glyph in glyphs {
        let adjustment = (!glyph.vertical)
            .then(|| {
                placed
                    .last()
                    .filter(|placed| placed.line == line)
                    .map(|left| pair_adjustment(scope, &left.glyph, &glyph))
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0);
        if adjustment != 0.0 {
            *x += adjustment;
            if let Some(previous) = placed.last_mut() {
                previous.glyph.width = (previous.glyph.width + adjustment).max(0.0);
                previous.glyph.paint_right_overflow =
                    (previous.glyph.paint_right_overflow - adjustment).max(0.0);
            }
        }
        let glyph_x = glyph
            .combine_group
            .and_then(|group| {
                placed
                    .iter()
                    .rev()
                    .find(|placed| placed.glyph.combine_group == Some(group))
                    .map(|placed| placed.x)
            })
            .or_else(|| {
                glyph.ruby_annotation.then(|| {
                    glyph
                        .ruby_group
                        .and_then(|group| {
                            placed
                                .iter()
                                .rev()
                                .find(|placed| {
                                    placed.glyph.ruby_group == Some(group)
                                        && !placed.glyph.ruby_annotation
                                })
                                .map(|placed| placed.x)
                        })
                        .unwrap_or(*x)
                })
            })
            .unwrap_or(*x);
        placed.push(PlacedGlyph {
            glyph: glyph.clone(),
            line,
            x: glyph_x,
            bidi_run_start: false,
        });
        if !glyph.ruby_annotation {
            *x += glyph.layout_advance;
        }
    }
}

fn glyphs_width(scope: &v8::PinScope<'_, '_>, glyphs: &[RawGlyph]) -> f64 {
    let Some(first) = glyphs.first() else {
        return 0.0;
    };
    if first.vertical {
        return glyphs.iter().map(|glyph| glyph.layout_advance).sum();
    }
    if glyphs.iter().all(|glyph| {
        glyph.font == first.font && glyph.implicit_default_font == first.implicit_default_font
    }) {
        let rendered = glyphs
            .iter()
            .map(|glyph| glyph.rendered.as_str())
            .collect::<String>();
        return super::offscreen_canvas_rendering_context_2d::measured_inline_text_width_for_font(
            scope,
            &rendered,
            &first.font,
            first.implicit_default_font,
        ) + glyphs.iter().map(|glyph| glyph.spacing).sum::<f64>();
    }
    glyphs.iter().map(|glyph| glyph.width).sum()
}

fn pair_adjustment(scope: &v8::PinScope<'_, '_>, left: &RawGlyph, right: &RawGlyph) -> f64 {
    if left.binary_shaped
        || right.binary_shaped
        || left.font != right.font
        || left.implicit_default_font != right.implicit_default_font
    {
        return 0.0;
    }
    // The captured ASCII kerning tables do not describe fallback-font
    // shaping.  Treating a mixed-script pair as though the entire pair used
    // the generic fallback path corrupts the preceding ASCII advance (for
    // example, the A in A+emoji).  Blink shapes the fallback run separately.
    if !left
        .rendered
        .chars()
        .all(|character| (' '..='~').contains(&character))
        || !right
            .rendered
            .chars()
            .all(|character| (' '..='~').contains(&character))
    {
        return 0.0;
    }
    let pair = format!("{}{}", left.rendered, right.rendered);
    let pair_width =
        super::offscreen_canvas_rendering_context_2d::measured_inline_text_width_for_font(
            scope,
            &pair,
            &left.font,
            left.implicit_default_font,
        );
    let left_width =
        super::offscreen_canvas_rendering_context_2d::measured_inline_text_width_for_font(
            scope,
            &left.rendered,
            &left.font,
            left.implicit_default_font,
        );
    let right_width =
        super::offscreen_canvas_rendering_context_2d::measured_inline_text_width_for_font(
            scope,
            &right.rendered,
            &right.font,
            right.implicit_default_font,
        );
    let dom_space_adjustment = if !left.implicit_default_font
        && left.rendered.chars().count() == 1
        && right.rendered.chars().count() == 1
        && (left.rendered == " " || right.rendered == " ")
        && let (Some(left_character), Some(right_character)) =
            (left.rendered.chars().next(), right.rendered.chars().next())
    {
        let font_size = super::offscreen_canvas_rendering_context_2d::canvas_font_size(&left.font);
        super::font_metric_tables::dom_whitespace_kerning_100(
            &left.font,
            left_character,
            right_character,
        ) * font_size
            / 100.0
    } else {
        0.0
    };
    pair_width - left_width - right_width + dom_space_adjustment
}

fn flush_word(tokens: &mut Vec<Token>, word: &mut Vec<RawGlyph>) {
    if !word.is_empty() {
        tokens.push(Token::Word(std::mem::take(word)));
    }
}

fn flush_space(tokens: &mut Vec<Token>, pending_space: &mut Vec<RawGlyph>) {
    if !pending_space.is_empty() {
        tokens.push(Token::Space(std::mem::take(pending_space), false));
    }
}

fn consolidate(
    fragments: &[TextFragment],
    include: impl Fn(&TextFragment) -> bool,
) -> Vec<super::dom_rect_read_only::RectRecord> {
    let mut output: Vec<super::dom_rect_read_only::RectRecord> = Vec::new();
    let mut output_lines = Vec::new();
    for fragment in fragments.iter().filter(|fragment| include(fragment)) {
        if let Some(last) = output.last_mut()
            && output_lines.last() == Some(&fragment.line)
            && !fragment.separate_rect
            // Kerning may make adjacent glyph boxes overlap. Blink still
            // exposes one line fragment, so merge both touching and
            // overlapping glyph geometry.
            && fragment.rect.x <= last.x + last.width + 1.0 / 64.0
        {
            let left = last.x.min(fragment.rect.x);
            let right = (last.x + last.width).max(fragment.rect.x + fragment.rect.width);
            let bottom = (last.y + last.height).max(fragment.rect.y + fragment.rect.height);
            last.y = last.y.min(fragment.rect.y);
            last.x = left;
            last.width = (right - left).max(0.0);
            last.height = (bottom - last.y).max(0.0);
        } else {
            output.push(fragment.rect);
            output_lines.push(fragment.line);
        }
    }
    output
}

fn consolidate_paint(fragments: &[TextFragment]) -> Vec<super::dom_rect_read_only::RectRecord> {
    let mut output: Vec<super::dom_rect_read_only::RectRecord> = Vec::new();
    let mut output_lines = Vec::new();
    for fragment in fragments {
        if fragment.separate_rect {
            continue;
        }
        let left = fragment.rect.x - fragment.paint_left_overflow;
        let right = fragment.rect.x + fragment.rect.width + fragment.paint_right_overflow;
        if let Some(last) = output.last_mut()
            && output_lines.last() == Some(&fragment.line)
        {
            let last_right = last.x + last.width;
            last.x = last.x.min(left);
            last.width = (last_right.max(right) - last.x).max(0.0);
        } else {
            output.push(super::dom_rect_read_only::RectRecord {
                x: left,
                y: fragment.rect.y,
                width: (right - left).max(0.0),
                height: fragment.rect.height,
            });
            output_lines.push(fragment.line);
        }
    }
    output
}

fn text_ink_height(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    line_height: f64,
) -> f64 {
    if super::element_layout::uses_implicit_default_font(scope, element) {
        return line_height;
    }
    let font_size = pixel_value(&super::get_computed_style_global::computed_property_value(
        scope,
        element,
        "font-size",
    ))
    .unwrap_or(16.0);
    let family =
        super::get_computed_style_global::computed_property_value(scope, element, "font-family")
            .to_ascii_lowercase();
    let candidate = if family.contains("segoe ui") {
        line_height
    } else if family.contains("arial") && font_size < 14.0 {
        line_height
    } else if font_size >= 32.0 {
        line_height - 2.0
    } else {
        line_height - 1.0
    };
    candidate.max(1.0)
}

fn pixel_value(value: &str) -> Option<f64> {
    value
        .trim()
        .strip_suffix("px")
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite())
}

fn quantize(value: f64) -> f64 {
    super::css_calculation::layout_unit(value)
}

fn quantize_end(value: f64) -> f64 {
    const MAX: f64 = (i32::MAX as f64) / 64.0;
    if value.is_nan() {
        0.0
    } else {
        (value.clamp(-MAX, MAX) * 64.0).round() / 64.0
    }
}

fn quantize_glyph_end(value: f64, font: &str) -> f64 {
    let size = super::offscreen_canvas_rendering_context_2d::canvas_font_size(font);
    if (size * 64.0 - (size * 64.0).round()).abs() > 1e-6 {
        const MAX: f64 = (i32::MAX as f64) / 64.0;
        (value.clamp(-MAX, MAX) * 64.0).ceil() / 64.0
    } else {
        quantize_end(value)
    }
}

fn quantize_vertical_glyph_end(value: f64) -> f64 {
    const MAX: f64 = (i32::MAX as f64) / 64.0;
    (value.clamp(-MAX, MAX) * 64.0).ceil() / 64.0
}
