use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
enum PaintStyle {
    Color(String),
    Object(v8::Global<v8::Object>),
}

#[derive(Clone)]
struct CanvasState {
    lang: String,
    font: String,
    text_align: String,
    text_baseline: String,
    direction: String,
    font_kerning: String,
    font_stretch: String,
    font_variant_caps: String,
    letter_spacing: String,
    text_rendering: String,
    word_spacing: String,
    global_composite_operation: String,
    filter: String,
    image_smoothing_quality: String,
    stroke_style: PaintStyle,
    fill_style: PaintStyle,
    shadow_color: String,
    line_cap: String,
    line_join: String,
    global_alpha: f64,
    image_smoothing_enabled: bool,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    line_width: f64,
    miter_limit: f64,
    line_dash_offset: f64,
    line_dash: Vec<f64>,
    transform: [f64; 6],
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            lang: "inherit".to_owned(),
            font: "10px sans-serif".to_owned(),
            text_align: "start".to_owned(),
            text_baseline: "alphabetic".to_owned(),
            direction: "ltr".to_owned(),
            font_kerning: "auto".to_owned(),
            font_stretch: "normal".to_owned(),
            font_variant_caps: "normal".to_owned(),
            letter_spacing: "0px".to_owned(),
            text_rendering: "auto".to_owned(),
            word_spacing: "0px".to_owned(),
            global_composite_operation: "source-over".to_owned(),
            filter: "none".to_owned(),
            image_smoothing_quality: "low".to_owned(),
            stroke_style: PaintStyle::Color("#000000".to_owned()),
            fill_style: PaintStyle::Color("#000000".to_owned()),
            shadow_color: "rgba(0, 0, 0, 0)".to_owned(),
            line_cap: "butt".to_owned(),
            line_join: "miter".to_owned(),
            global_alpha: 1.0,
            image_smoothing_enabled: true,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            line_width: 1.0,
            miter_limit: 10.0,
            line_dash_offset: 0.0,
            line_dash: Vec::new(),
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

#[derive(Clone)]
enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    Rect(f64, f64, f64, f64),
    RoundRect(f64, f64, f64, f64),
    Arc(f64, f64, f64, f64, f64, bool),
    ArcTo(f64, f64, f64, f64, f64),
    Bezier(f64, f64, f64, f64, f64, f64),
    Quadratic(f64, f64, f64, f64),
    Ellipse(f64, f64, f64, f64, f64, f64, f64, bool),
    Close,
}

#[derive(Clone)]
struct ContextRecord {
    canvas: v8::Global<v8::Object>,
    state: CanvasState,
    stack: Vec<CanvasState>,
    path: Vec<PathCommand>,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    alpha: bool,
    color_space: String,
    color_type: String,
    desynchronized: bool,
    will_read_frequently: bool,
}

#[derive(Default)]
pub(crate) struct OffscreenCanvasRenderingContext2DStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, ContextRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OffscreenCanvasRenderingContext2DStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "OffscreenCanvasRenderingContext2D",
        constructor.into(),
    )
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<OffscreenCanvasRenderingContext2DStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "OffscreenCanvasRenderingContext2D",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "canvas", get_canvas)?;
    crate::webidl::define_accessor(scope, prototype, "lang", get_lang, set_lang)?;
    crate::webidl::define_accessor(scope, prototype, "font", get_font, set_font)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "textAlign",
        get_text_align,
        set_text_align,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "textBaseline",
        get_text_baseline,
        set_text_baseline,
    )?;
    crate::webidl::define_accessor(scope, prototype, "direction", get_direction, set_direction)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "fontKerning",
        get_font_kerning,
        set_font_kerning,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "fontStretch",
        get_font_stretch,
        set_font_stretch,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "fontVariantCaps",
        get_font_variant_caps,
        set_font_variant_caps,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "letterSpacing",
        get_letter_spacing,
        set_letter_spacing,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "textRendering",
        get_text_rendering,
        set_text_rendering,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "wordSpacing",
        get_word_spacing,
        set_word_spacing,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "globalCompositeOperation",
        get_global_composite_operation,
        set_global_composite_operation,
    )?;
    crate::webidl::define_accessor(scope, prototype, "filter", get_filter, set_filter)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "imageSmoothingQuality",
        get_image_smoothing_quality,
        set_image_smoothing_quality,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "strokeStyle",
        get_stroke_style,
        set_stroke_style,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "fillStyle",
        get_fill_style,
        set_fill_style,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "shadowColor",
        get_shadow_color,
        set_shadow_color,
    )?;
    crate::webidl::define_accessor(scope, prototype, "lineCap", get_line_cap, set_line_cap)?;
    crate::webidl::define_accessor(scope, prototype, "lineJoin", get_line_join, set_line_join)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "globalAlpha",
        get_global_alpha,
        set_global_alpha,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "imageSmoothingEnabled",
        get_image_smoothing_enabled,
        set_image_smoothing_enabled,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "shadowOffsetX",
        get_shadow_offset_x,
        set_shadow_offset_x,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "shadowOffsetY",
        get_shadow_offset_y,
        set_shadow_offset_y,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "shadowBlur",
        get_shadow_blur,
        set_shadow_blur,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "lineWidth",
        get_line_width,
        set_line_width,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "miterLimit",
        get_miter_limit,
        set_miter_limit,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "lineDashOffset",
        get_line_dash_offset,
        set_line_dash_offset,
    )?;
    crate::webidl::define_method(scope, prototype, "clip", 0, clip)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createConicGradient",
        3,
        create_conic_gradient,
    )?;
    crate::webidl::define_method(scope, prototype, "createImageData", 1, create_image_data)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createLinearGradient",
        4,
        create_linear_gradient,
    )?;
    crate::webidl::define_method(scope, prototype, "createPattern", 2, create_pattern)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createRadialGradient",
        6,
        create_radial_gradient,
    )?;
    crate::webidl::define_method(scope, prototype, "drawImage", 3, draw_image)?;
    crate::webidl::define_method(scope, prototype, "fill", 0, fill)?;
    crate::webidl::define_method(scope, prototype, "fillText", 3, fill_text)?;
    crate::webidl::define_method(scope, prototype, "getImageData", 4, get_image_data)?;
    crate::webidl::define_method(scope, prototype, "getLineDash", 0, get_line_dash)?;
    crate::webidl::define_method(scope, prototype, "getTransform", 0, get_transform)?;
    crate::webidl::define_method(scope, prototype, "isContextLost", 0, is_context_lost)?;
    crate::webidl::define_method(scope, prototype, "isPointInPath", 2, is_point_in_path)?;
    crate::webidl::define_method(scope, prototype, "isPointInStroke", 2, is_point_in_stroke)?;
    crate::webidl::define_method(scope, prototype, "measureText", 1, measure_text)?;
    crate::webidl::define_method(scope, prototype, "reset", 0, reset)?;
    crate::webidl::define_method(scope, prototype, "roundRect", 4, round_rect)?;
    crate::webidl::define_method(scope, prototype, "setLineDash", 1, set_line_dash)?;
    crate::webidl::define_method(scope, prototype, "strokeText", 3, stroke_text)?;
    crate::webidl::define_method(scope, prototype, "arc", 5, arc)?;
    crate::webidl::define_method(scope, prototype, "arcTo", 5, arc_to)?;
    crate::webidl::define_method(scope, prototype, "beginPath", 0, begin_path)?;
    crate::webidl::define_method(scope, prototype, "bezierCurveTo", 6, bezier_curve_to)?;
    crate::webidl::define_method(scope, prototype, "clearRect", 4, clear_rect)?;
    crate::webidl::define_method(scope, prototype, "closePath", 0, close_path)?;
    crate::webidl::define_method(scope, prototype, "ellipse", 7, ellipse)?;
    crate::webidl::define_method(scope, prototype, "fillRect", 4, fill_rect)?;
    crate::webidl::define_method(scope, prototype, "lineTo", 2, line_to)?;
    crate::webidl::define_method(scope, prototype, "moveTo", 2, move_to)?;
    crate::webidl::define_method(scope, prototype, "putImageData", 3, put_image_data)?;
    crate::webidl::define_method(scope, prototype, "quadraticCurveTo", 4, quadratic_curve_to)?;
    crate::webidl::define_method(scope, prototype, "rect", 4, rect)?;
    crate::webidl::define_method(scope, prototype, "resetTransform", 0, reset_transform)?;
    crate::webidl::define_method(scope, prototype, "restore", 0, restore)?;
    crate::webidl::define_method(scope, prototype, "rotate", 1, rotate)?;
    crate::webidl::define_method(scope, prototype, "save", 0, save)?;
    crate::webidl::define_method(scope, prototype, "scale", 2, scale)?;
    crate::webidl::define_method(scope, prototype, "setTransform", 0, set_transform)?;
    crate::webidl::define_method(scope, prototype, "stroke", 0, stroke)?;
    crate::webidl::define_method(scope, prototype, "strokeRect", 4, stroke_rect)?;
    crate::webidl::define_method(scope, prototype, "transform", 6, transform)?;
    crate::webidl::define_method(scope, prototype, "translate", 2, translate)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getContextAttributes",
        0,
        get_context_attributes,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<OffscreenCanvasRenderingContext2DStore>()
        .ok_or_else(|| "OffscreenCanvasRenderingContext2D state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'OffscreenCanvasRenderingContext2D': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    canvas: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let (width, height) = super::offscreen_canvas::dimensions(scope, canvas)
        .ok_or_else(|| "The canvas is not an OffscreenCanvas".to_owned())?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let context = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, context, prototype.into()) != Some(true) {
        return Err("cannot create OffscreenCanvasRenderingContext2D".to_owned());
    }
    let alpha = options
        .map(|value| boolean_option(scope, value, "alpha", true))
        .unwrap_or(true);
    let desynchronized = options
        .map(|value| boolean_option(scope, value, "desynchronized", false))
        .unwrap_or(false);
    let will_read_frequently = options
        .map(|value| boolean_option(scope, value, "willReadFrequently", false))
        .unwrap_or(false);
    let color_space = options
        .map(|value| string_option(scope, value, "colorSpace", "srgb"))
        .unwrap_or_else(|| "srgb".to_owned());
    let color_type = options
        .map(|value| string_option(scope, value, "colorType", "unorm8"))
        .unwrap_or_else(|| "unorm8".to_owned());
    let canvas = v8::Global::new(scope, canvas);
    scope
        .get_slot_mut::<OffscreenCanvasRenderingContext2DStore>()
        .ok_or_else(|| "OffscreenCanvasRenderingContext2D state was not prepared".to_owned())?
        .records
        .insert(
            context.get_identity_hash().get(),
            ContextRecord {
                canvas,
                state: CanvasState::default(),
                stack: Vec::new(),
                path: Vec::new(),
                width,
                height,
                pixels: vec![0_u8; width as usize * height as usize * 4],
                alpha,
                color_space,
                color_type,
                desynchronized,
                will_read_frequently,
            },
        );
    Ok(context)
}

fn boolean_option(
    scope: &v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Object>,
    name: &str,
    default: bool,
) -> bool {
    let Some(key) = v8::String::new(scope, name) else {
        return default;
    };
    options
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| value.boolean_value(scope))
        .unwrap_or(default)
}
fn string_option(
    scope: &v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Object>,
    name: &str,
    default: &str,
) -> String {
    let Some(key) = v8::String::new(scope, name) else {
        return default.to_owned();
    };
    options
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_else(|| default.to_owned())
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ContextRecord> {
    scope
        .get_slot::<OffscreenCanvasRenderingContext2DStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn require_context(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    if record(scope, object).is_some() {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn pixel_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    record(scope, object).map(|record| (record.width, record.height, record.pixels))
}

pub(crate) fn take_pixel_snapshot_and_reset(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = scope
        .get_slot_mut::<OffscreenCanvasRenderingContext2DStore>()?
        .records
        .get_mut(&object.get_identity_hash().get())?;
    let pixels = std::mem::replace(
        &mut record.pixels,
        vec![0; record.width as usize * record.height as usize * 4],
    );
    Some((record.width, record.height, pixels))
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut ContextRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<OffscreenCanvasRenderingContext2DStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn reset_for_resize(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) {
    let canvas = record(scope, context).map(|record| record.canvas);
    let Some(canvas) = canvas else {
        return;
    };
    let canvas = v8::Local::new(scope, &canvas);
    let Some((width, height)) = super::offscreen_canvas::dimensions(scope, canvas) else {
        return;
    };
    update(scope, context, |record| {
        record.state = CanvasState::default();
        record.stack.clear();
        record.path.clear();
        record.width = width;
        record.height = height;
        record.pixels = vec![0_u8; width as usize * height as usize * 4];
    });
}

pub(crate) fn snapshot_pixels(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    record(scope, context).map(|record| (record.width, record.height, record.pixels))
}

fn get_canvas(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.canvas).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&CanvasState) -> &str,
) {
    if let Some(v) = record(scope, a.this()) {
        if let Some(s) = v8::String::new(scope, select(&v.state)) {
            r.set(s.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    select: fn(&mut CanvasState) -> &mut String,
    valid: fn(&str) -> bool,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if valid(&value) {
        update(scope, a.this(), |record| *select(&mut record.state) = value)
    }
}
fn any(_: &str) -> bool {
    true
}
fn get_lang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.lang)
}
fn set_lang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |v| &mut v.lang, any)
}
fn get_font(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.font)
}
fn set_font(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_context(s, a.this()) {
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    let fingerprint = crate::fingerprint::edge(s);
    let context = CanvasFontParseContext {
        platform: &fingerprint.navigator.platform,
        viewport_width: fingerprint.screen.viewport_width,
        viewport_height: fingerprint.screen.viewport_height,
    };
    if let Some(value) = parse_canvas_font(&value, context) {
        update(s, a.this(), |record| {
            record.state.font = value.serialized;
            record.state.font_stretch = value.stretch;
            record.state.font_variant_caps = value.variant_caps;
        });
    }
}

#[derive(Clone, Copy)]
struct CanvasFontParseContext<'a> {
    platform: &'a str,
    viewport_width: f64,
    viewport_height: f64,
}

struct ParsedCanvasFont {
    serialized: String,
    stretch: String,
    variant_caps: String,
}

/// Parse and serialize the CSS `font` shorthand used by Canvas. Invalid
/// assignments are ignored by the platform setter, so this deliberately
/// returns `None` instead of retaining an unparsed source string.
fn canonical_canvas_font(value: &str, context: CanvasFontParseContext<'_>) -> Option<String> {
    parse_canvas_font(value, context).map(|font| font.serialized)
}

fn parse_canvas_font(value: &str, context: CanvasFontParseContext<'_>) -> Option<ParsedCanvasFont> {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();
    if value.is_empty()
        || value.contains('\0')
        || lower.contains("var(")
        || lower.contains("env(")
        || lower.contains("!important")
        || matches!(
            lower.as_str(),
            "inherit" | "initial" | "unset" | "revert" | "revert-layer"
        )
    {
        return None;
    }
    if let Some(system) = canvas_system_font(&lower, context.platform) {
        return Some(ParsedCanvasFont {
            serialized: system.to_owned(),
            stretch: "normal".to_owned(),
            variant_caps: "normal".to_owned(),
        });
    }

    let tokens = canvas_font_tokens(value)?;
    let size_index = tokens
        .iter()
        .position(|token| canvas_font_size_value(token, context).is_some())?;
    let size = canvas_font_size_value(&tokens[size_index], context)?;
    if !size.is_finite() || size < 0.0 {
        return None;
    }

    let mut style = None;
    let mut variant = None;
    let mut weight = None;
    let mut stretch = None;
    let mut position = 0;
    while position < size_index {
        let token = tokens[position].to_ascii_lowercase();
        match token.as_str() {
            "normal" => {}
            "italic" if style.is_none() => style = Some("italic"),
            "oblique" if style.is_none() => {
                // Edge 150 serializes an angle-less oblique style as italic.
                // Its Canvas shorthand consumes an optional angle but does not
                // retain that style in the exposed serialization.
                if tokens
                    .get(position + 1)
                    .is_some_and(|next| is_css_angle(next))
                {
                    position += 1;
                    style = Some("normal");
                } else {
                    style = Some("italic");
                }
            }
            "small-caps" if variant.is_none() => variant = Some("small-caps"),
            "bold" | "bolder" if weight.is_none() => weight = Some("bold".to_owned()),
            "lighter" if weight.is_none() => weight = Some("100".to_owned()),
            value
                if weight.is_none()
                    && value.parse::<f64>().is_ok_and(|number| number.is_finite()) =>
            {
                let number = value.parse::<f64>().ok()?;
                if !number.is_finite() || !(1.0..=1000.0).contains(&number) {
                    return None;
                }
                weight = match number {
                    400.0 => Some("normal".to_owned()),
                    700.0 => Some("bold".to_owned()),
                    _ => Some(format_css_number(number)),
                };
            }
            value if stretch.is_none() && is_font_stretch_keyword(value) => {
                stretch = Some(value.to_owned())
            }
            _ => return None,
        }
        position += 1;
    }

    position = size_index + 1;
    if tokens.get(position).is_some_and(|token| token == "/") {
        let line_height = tokens.get(position + 1)?;
        if !valid_canvas_line_height(line_height, size, context) {
            return None;
        }
        position += 2;
    }
    if position >= tokens.len() || tokens[position] == "/" {
        return None;
    }
    let family = canonical_font_family_list(&tokens[position..].join(" "))?;

    let mut output = Vec::new();
    if let Some(style) = style.filter(|value| *value != "normal") {
        output.push(style.to_owned());
    }
    if let Some(variant) = variant {
        output.push(variant.to_owned());
    }
    if let Some(weight) = weight.filter(|value| value != "normal") {
        output.push(weight);
    }
    // Canvas retains font-stretch for face selection but omits it from the
    // `font` getter serialization in Edge 150. Consuming it here still makes
    // the rest of the shorthand parse correctly.
    output.push(format!("{}px", format_css_number(size)));
    output.push(family);
    Some(ParsedCanvasFont {
        serialized: output.join(" "),
        stretch: stretch.unwrap_or_else(|| "normal".to_owned()),
        variant_caps: variant.unwrap_or("normal").to_owned(),
    })
}

fn canvas_system_font(value: &str, platform: &str) -> Option<&'static str> {
    if platform.to_ascii_lowercase().starts_with("win") {
        match value {
            "caption" | "icon" | "message-box" => Some("16px Arial"),
            "menu" | "small-caption" | "status-bar" => Some("12px \"Microsoft YaHei UI\""),
            _ => None,
        }
    } else if platform.eq_ignore_ascii_case("MacIntel") {
        match value {
            "caption" | "icon" | "menu" | "message-box" | "small-caption" | "status-bar" => {
                Some("13px system-ui")
            }
            _ => None,
        }
    } else {
        match value {
            "caption" | "icon" | "menu" | "message-box" | "small-caption" | "status-bar" => {
                Some("16px sans-serif")
            }
            _ => None,
        }
    }
}

fn canvas_font_tokens(value: &str) -> Option<Vec<String>> {
    let mut output = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut depth = 0_u32;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            token.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            token.push(character);
            escaped = true;
            continue;
        }
        if let Some(delimiter) = quote {
            token.push(character);
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            token.push(character);
            continue;
        }
        match character {
            '(' => {
                depth += 1;
                token.push(character);
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                token.push(character);
            }
            '/' if depth == 0 => {
                if !token.is_empty() {
                    output.push(std::mem::take(&mut token));
                }
                output.push("/".to_owned());
            }
            character if character.is_whitespace() && depth == 0 => {
                if !token.is_empty() {
                    output.push(std::mem::take(&mut token));
                }
            }
            _ => token.push(character),
        }
    }
    if escaped || depth != 0 {
        return None;
    }
    if !token.is_empty() {
        output.push(token);
    }
    (!output.is_empty()).then_some(output)
}

fn canvas_font_size_value(token: &str, context: CanvasFontParseContext<'_>) -> Option<f64> {
    let lower = token.to_ascii_lowercase();
    let named = match lower.as_str() {
        "xx-small" => Some(9.0),
        "x-small" => Some(10.0),
        "small" => Some(13.0),
        "medium" => Some(16.0),
        "large" => Some(18.0),
        "x-large" => Some(24.0),
        "xx-large" => Some(32.0),
        "xxx-large" => Some(48.0),
        "smaller" => Some(8.0),
        "larger" => Some(12.0),
        "math" => Some(16.0),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    let resolved = crate::web::css_calculation::resolve_length(
        &lower,
        crate::web::css_calculation::EvaluationContext {
            viewport_width: context.viewport_width,
            viewport_height: context.viewport_height,
            percentage_basis: Some(10.0),
            font_size: 10.0,
            root_font_size: 16.0,
            intrinsic_size: None,
        },
    )?;
    if resolved < 0.0 {
        return None;
    }
    // Absolute units are serialized before LayoutUnit quantization in the
    // Canvas font getter (16pt -> 21.3333px rather than 21.3281px).
    crate::web::css_calculation::computed_absolute_length(&lower)
        .and_then(|value| value.strip_suffix("px")?.parse::<f64>().ok())
        .or(Some(resolved))
}

fn valid_canvas_line_height(value: &str, size: f64, context: CanvasFontParseContext<'_>) -> bool {
    if value.eq_ignore_ascii_case("normal") {
        return true;
    }
    if let Ok(number) = value.parse::<f64>() {
        return number.is_finite() && number >= 0.0;
    }
    crate::web::css_calculation::resolve_line_height(
        &value.to_ascii_lowercase(),
        crate::web::css_calculation::EvaluationContext {
            viewport_width: context.viewport_width,
            viewport_height: context.viewport_height,
            percentage_basis: Some(size),
            font_size: size,
            root_font_size: 16.0,
            intrinsic_size: None,
        },
    )
    .is_some_and(|value| value >= 0.0)
}

fn is_css_angle(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    ["deg", "grad", "rad", "turn"]
        .iter()
        .find_map(|unit| lower.strip_suffix(unit))
        .and_then(|value| value.parse::<f64>().ok())
        .is_some_and(f64::is_finite)
}

fn is_font_stretch_keyword(value: &str) -> bool {
    matches!(
        value,
        "ultra-condensed"
            | "extra-condensed"
            | "condensed"
            | "semi-condensed"
            | "semi-expanded"
            | "expanded"
            | "extra-expanded"
            | "ultra-expanded"
    )
}

fn canonical_font_family_list(value: &str) -> Option<String> {
    let families = split_canvas_font_families(value)?;
    let mut output = Vec::with_capacity(families.len());
    for family in families {
        let family = canonical_font_family(&family)?;
        output.push(family);
    }
    (!output.is_empty()).then(|| output.join(", "))
}

fn split_canvas_font_families(value: &str) -> Option<Vec<String>> {
    let mut output = Vec::new();
    let mut part = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            part.push('\\');
            part.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if let Some(delimiter) = quote {
            part.push(character);
            if character == delimiter {
                quote = None;
            }
        } else if matches!(character, '\'' | '"') {
            quote = Some(character);
            part.push(character);
        } else if character == ',' {
            let family = part.trim();
            if family.is_empty() {
                return None;
            }
            output.push(family.to_owned());
            part.clear();
        } else {
            part.push(character);
        }
    }
    if escaped || part.trim().is_empty() {
        return None;
    }
    output.push(part.trim().to_owned());
    Some(output)
}

fn canonical_font_family(value: &str) -> Option<String> {
    let value = value.trim();
    let quoted = value.starts_with(['\'', '"']);
    if !quoted && !valid_unquoted_family(value) {
        return None;
    }
    let raw = if quoted {
        let delimiter = value.chars().next()?;
        value
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter))
            // Blink accepts an unterminated opening quote in this Canvas
            // setter and serializes the remaining identifier normally.
            .unwrap_or_else(|| &value[delimiter.len_utf8()..])
    } else {
        value
    };
    let family = unescape_css_identifier(raw.trim())?;
    if family.is_empty()
        || family.chars().any(|character| character.is_control())
        || matches!(
            family.to_ascii_lowercase().as_str(),
            "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "default"
        )
    {
        return None;
    }
    let generic = matches!(
        family.to_ascii_lowercase().as_str(),
        "serif"
            | "sans-serif"
            | "cursive"
            | "fantasy"
            | "monospace"
            | "system-ui"
            | "ui-serif"
            | "ui-sans-serif"
            | "ui-monospace"
            | "ui-rounded"
            | "math"
            | "fangsong"
    );
    let simple = family
        .chars()
        .all(|character| character.is_alphanumeric() || matches!(character, '-' | '_'));
    if generic || (simple && !quoted) {
        Some(family)
    } else if simple && quoted && !family.contains(char::is_whitespace) {
        Some(family)
    } else {
        Some(format!(
            "\"{}\"",
            family.replace('\\', "\\\\").replace('"', "\\\"")
        ))
    }
}

fn valid_unquoted_family(value: &str) -> bool {
    let mut escaped = false;
    let mut at_identifier_start = true;
    for character in value.chars() {
        if escaped {
            escaped = false;
            at_identifier_start = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
        } else if character.is_whitespace() {
            at_identifier_start = true;
        } else if at_identifier_start {
            if character.is_ascii_digit() || matches!(character, '.' | '+' | '/' | '!' | '(' | ')')
            {
                return false;
            }
            at_identifier_start = false;
        } else if matches!(character, '/' | '!' | '(' | ')') {
            return false;
        }
    }
    !escaped && !at_identifier_start
}

fn unescape_css_identifier(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        let first = characters.next()?;
        if first.is_ascii_hexdigit() {
            let mut digits = String::from(first);
            while digits.len() < 6
                && characters
                    .peek()
                    .is_some_and(|character| character.is_ascii_hexdigit())
            {
                digits.push(characters.next()?);
            }
            if characters
                .peek()
                .is_some_and(|character| character.is_whitespace())
            {
                characters.next();
            }
            let scalar = u32::from_str_radix(&digits, 16).ok()?;
            output.push(char::from_u32(scalar).unwrap_or('\u{FFFD}'));
        } else if first != '\n' && first != '\r' {
            output.push(first);
        }
    }
    Some(output)
}

fn format_css_number(value: f64) -> String {
    let mut output = format!("{value:.4}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    if output == "-0" {
        "0".to_owned()
    } else {
        output
    }
}

#[cfg(test)]
mod canvas_font_parser_tests {
    use super::{CanvasFontParseContext, canonical_canvas_font};

    fn parse(value: &str) -> Option<String> {
        canonical_canvas_font(
            value,
            CanvasFontParseContext {
                platform: "Win32",
                viewport_width: 1536.0,
                viewport_height: 864.0,
            },
        )
    }

    #[test]
    fn matches_edge_150_canvas_font_serialization_matrix() {
        let accepted = [
            ("caption", "16px Arial"),
            ("menu", "12px \"Microsoft YaHei UI\""),
            ("italic 16px serif", "italic 16px serif"),
            ("oblique 16px serif", "italic 16px serif"),
            ("oblique 10deg 16px serif", "16px serif"),
            ("small-caps 16px serif", "small-caps 16px serif"),
            ("700 16px serif", "bold 16px serif"),
            ("1000 16px serif", "1000 16px serif"),
            ("condensed 16px serif", "16px serif"),
            (
                "normal normal normal normal 16px/normal serif",
                "16px serif",
            ),
            ("16px/20px Arial", "16px Arial"),
            ("16px/1.5 Arial", "16px Arial"),
            ("16pt Arial", "21.3333px Arial"),
            ("1em Arial", "10px Arial"),
            ("100% Arial", "10px Arial"),
            ("medium Arial", "16px Arial"),
            ("16px Arial, serif", "16px Arial, serif"),
            (r"16px A\ B", "16px \"A B\""),
            ("16px \"A B\", serif", "16px \"A B\", serif"),
            ("16PX Arial", "16px Arial"),
            ("calc(16px) Arial", "16px Arial"),
        ];
        for (source, expected) in accepted {
            assert_eq!(parse(source).as_deref(), Some(expected), "{source}");
        }
        for invalid in [
            "",
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "foo",
            "16px",
            "1001 16px serif",
            "75% 16px serif",
            "-1px Arial",
            "16px var(--x)",
            "16px serif !important",
        ] {
            assert_eq!(parse(invalid), None, "{invalid}");
        }
    }
}
fn get_text_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.text_align)
}
fn set_text_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.text_align,
        |v| matches!(v, "start" | "end" | "left" | "right" | "center"),
    )
}
fn get_text_baseline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.text_baseline)
}
fn set_text_baseline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.text_baseline,
        |v| {
            matches!(
                v,
                "top" | "hanging" | "middle" | "alphabetic" | "ideographic" | "bottom"
            )
        },
    )
}
fn get_direction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.direction)
}
fn set_direction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.direction,
        |v| matches!(v, "ltr" | "rtl" | "inherit"),
    )
}
fn get_font_kerning(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.font_kerning)
}
fn set_font_kerning(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.font_kerning,
        |v| matches!(v, "auto" | "normal" | "none"),
    )
}
fn get_font_stretch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.font_stretch)
}
fn set_font_stretch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.font_stretch,
        |v| {
            matches!(
                v,
                "ultra-condensed"
                    | "extra-condensed"
                    | "condensed"
                    | "semi-condensed"
                    | "normal"
                    | "semi-expanded"
                    | "expanded"
                    | "extra-expanded"
                    | "ultra-expanded"
            )
        },
    )
}
fn get_font_variant_caps(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.font_variant_caps)
}
fn set_font_variant_caps(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.font_variant_caps,
        |v| {
            matches!(
                v,
                "normal"
                    | "small-caps"
                    | "all-small-caps"
                    | "petite-caps"
                    | "all-petite-caps"
                    | "unicase"
                    | "titling-caps"
            )
        },
    )
}
fn get_letter_spacing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.letter_spacing)
}
fn set_letter_spacing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |v| &mut v.letter_spacing, any)
}
fn get_text_rendering(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.text_rendering)
}
fn set_text_rendering(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |v| &mut v.text_rendering, any)
}
fn get_word_spacing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.word_spacing)
}
fn set_word_spacing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |v| &mut v.word_spacing, any)
}
fn get_global_composite_operation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.global_composite_operation)
}
fn set_global_composite_operation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.global_composite_operation,
        |v| {
            matches!(
                v,
                "source-over"
                    | "source-in"
                    | "source-out"
                    | "source-atop"
                    | "destination-over"
                    | "destination-in"
                    | "destination-out"
                    | "destination-atop"
                    | "lighter"
                    | "copy"
                    | "xor"
                    | "multiply"
                    | "screen"
                    | "overlay"
                    | "darken"
                    | "lighten"
                    | "color-dodge"
                    | "color-burn"
                    | "hard-light"
                    | "soft-light"
                    | "difference"
                    | "exclusion"
                    | "hue"
                    | "saturation"
                    | "color"
                    | "luminosity"
            )
        },
    )
}
fn get_filter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.filter)
}
fn set_filter(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |v| &mut v.filter, any)
}
fn get_image_smoothing_quality(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.image_smoothing_quality)
}
fn set_image_smoothing_quality(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.image_smoothing_quality,
        |v| matches!(v, "low" | "medium" | "high"),
    )
}
fn get_shadow_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.shadow_color)
}
fn set_shadow_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |v| &mut v.shadow_color, |v| !v.trim().is_empty())
}
fn get_line_cap(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.line_cap)
}
fn set_line_cap(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.line_cap,
        |v| matches!(v, "butt" | "round" | "square"),
    )
}
fn get_line_join(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |v| &v.line_join)
}
fn set_line_join(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(
        s,
        a,
        |v| &mut v.line_join,
        |v| matches!(v, "round" | "bevel" | "miter"),
    )
}

fn get_style(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&CanvasState) -> PaintStyle,
) {
    if let Some(v) = record(scope, a.this()) {
        match select(&v.state) {
            PaintStyle::Color(c) => {
                if let Some(s) = v8::String::new(scope, &c) {
                    r.set(s.into())
                }
            }
            PaintStyle::Object(o) => r.set(v8::Local::new(scope, &o).into()),
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    select: fn(&mut CanvasState) -> &mut PaintStyle,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let value = a.get(0);
    let style = if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        if super::canvas_gradient::is_gradient(scope, object)
            || super::canvas_pattern::is_pattern(scope, object)
        {
            Some(PaintStyle::Object(v8::Global::new(scope, object)))
        } else {
            None
        }
    } else {
        let color = crate::webidl::value_to_string(scope, value);
        (!color.trim().is_empty()).then_some(PaintStyle::Color(normalize_color(&color)))
    };
    if let Some(style) = style {
        update(scope, a.this(), |record| *select(&mut record.state) = style)
    }
}
fn normalize_color(value: &str) -> String {
    match value.to_ascii_lowercase().as_str() {
        "black" => "#000000",
        "white" => "#ffffff",
        "red" => "#ff0000",
        "blue" => "#0000ff",
        "green" => "#008000",
        _ => value,
    }
    .to_owned()
}
fn get_stroke_style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_style(s, a, r, |v| v.stroke_style.clone())
}
fn set_stroke_style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_style(s, a, |v| &mut v.stroke_style)
}
fn get_fill_style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_style(s, a, r, |v| v.fill_style.clone())
}
fn set_fill_style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_style(s, a, |v| &mut v.fill_style)
}

fn get_number(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&CanvasState) -> f64,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, select(&v.state)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_number(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    select: fn(&mut CanvasState) -> &mut f64,
    valid: fn(f64) -> bool,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let value = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    if valid(value) {
        update(scope, a.this(), |record| *select(&mut record.state) = value)
    }
}
fn finite(v: f64) -> bool {
    v.is_finite()
}
fn positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}
fn non_negative(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}
fn unit(v: f64) -> bool {
    v.is_finite() && (0.0..=1.0).contains(&v)
}
fn get_global_alpha(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_number(s, a, r, |v| v.global_alpha)
}
fn set_global_alpha(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.global_alpha, unit)
}
fn get_shadow_offset_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_number(s, a, r, |v| v.shadow_offset_x)
}
fn set_shadow_offset_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.shadow_offset_x, finite)
}
fn get_shadow_offset_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_number(s, a, r, |v| v.shadow_offset_y)
}
fn set_shadow_offset_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.shadow_offset_y, finite)
}
fn get_shadow_blur(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_number(s, a, r, |v| v.shadow_blur)
}
fn set_shadow_blur(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.shadow_blur, non_negative)
}
fn get_line_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_number(s, a, r, |v| v.line_width)
}
fn set_line_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.line_width, positive)
}
fn get_miter_limit(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_number(s, a, r, |v| v.miter_limit)
}
fn set_miter_limit(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.miter_limit, positive)
}
fn get_line_dash_offset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_number(s, a, r, |v| v.line_dash_offset)
}
fn set_line_dash_offset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.line_dash_offset, finite)
}
fn get_image_smoothing_enabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, v.state.image_smoothing_enabled).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_image_smoothing_enabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    update(scope, a.this(), |record| {
        record.state.image_smoothing_enabled = value
    })
}

fn values(
    scope: &v8::PinScope<'_, '_>,
    a: &v8::FunctionCallbackArguments<'_>,
    count: usize,
) -> Vec<f64> {
    (0..count)
        .map(|index| a.get(index as i32).number_value(scope).unwrap_or(f64::NAN))
        .collect()
}
fn push_path(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    command: PathCommand,
) {
    update(scope, object, |record| record.path.push(command))
}
fn begin_path(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |record| record.path.clear())
}
fn close_path(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    push_path(scope, a.this(), PathCommand::Close)
}
fn move_to(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 2);
    push_path(scope, a.this(), PathCommand::MoveTo(v[0], v[1]))
}
fn line_to(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 2);
    push_path(scope, a.this(), PathCommand::LineTo(v[0], v[1]))
}
fn rect(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 4);
    push_path(scope, a.this(), PathCommand::Rect(v[0], v[1], v[2], v[3]))
}
fn round_rect(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 4);
    push_path(
        scope,
        a.this(),
        PathCommand::RoundRect(v[0], v[1], v[2], v[3]),
    )
}
fn arc(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 5);
    if v[2] < 0.0 {
        throw_index_size(scope, "The radius is negative");
        return;
    }
    push_path(
        scope,
        a.this(),
        PathCommand::Arc(v[0], v[1], v[2], v[3], v[4], a.get(5).boolean_value(scope)),
    )
}
fn arc_to(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 5);
    if v[4] < 0.0 {
        throw_index_size(scope, "The radius is negative");
        return;
    }
    push_path(
        scope,
        a.this(),
        PathCommand::ArcTo(v[0], v[1], v[2], v[3], v[4]),
    )
}
fn bezier_curve_to(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 6);
    push_path(
        scope,
        a.this(),
        PathCommand::Bezier(v[0], v[1], v[2], v[3], v[4], v[5]),
    )
}
fn quadratic_curve_to(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 4);
    push_path(
        scope,
        a.this(),
        PathCommand::Quadratic(v[0], v[1], v[2], v[3]),
    )
}
fn ellipse(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 7);
    if v[2] < 0.0 || v[3] < 0.0 {
        throw_index_size(scope, "An ellipse radius is negative");
        return;
    }
    push_path(
        scope,
        a.this(),
        PathCommand::Ellipse(
            v[0],
            v[1],
            v[2],
            v[3],
            v[4],
            v[5],
            v[6],
            a.get(7).boolean_value(scope),
        ),
    )
}
fn throw_index_size(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "IndexSizeError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn create_linear_gradient(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let v = values(scope, &a, 4);
    if let Ok(g) = super::canvas_gradient::create(
        scope,
        super::canvas_gradient::CanvasGradientKind::Linear([v[0], v[1], v[2], v[3]]),
    ) {
        r.set(g.into())
    }
}
fn create_radial_gradient(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let v = values(scope, &a, 6);
    if v[2] < 0.0 || v[5] < 0.0 {
        throw_index_size(scope, "A radial gradient radius is negative");
        return;
    }
    if let Ok(g) = super::canvas_gradient::create(
        scope,
        super::canvas_gradient::CanvasGradientKind::Radial([v[0], v[1], v[2], v[3], v[4], v[5]]),
    ) {
        r.set(g.into())
    }
}
fn create_conic_gradient(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let v = values(scope, &a, 3);
    if let Ok(g) = super::canvas_gradient::create(
        scope,
        super::canvas_gradient::CanvasGradientKind::Conic([v[0], v[1], v[2]]),
    ) {
        r.set(g.into())
    }
}
fn create_pattern(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let Ok(source) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "The image source is invalid");
        return;
    };
    let repetition = if a.get(1).is_null_or_undefined() {
        "repeat".to_owned()
    } else {
        crate::webidl::value_to_string(scope, a.get(1))
    };
    match super::canvas_pattern::create(scope, source, &repetition) {
        Ok(pattern) => r.set(pattern.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_image_data(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let (width, height) = if a.get(1).is_undefined() {
        let Ok(source) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
            crate::webidl::throw_type_error(scope, "ImageData or dimensions are required");
            return;
        };
        let Some((w, h, _)) = super::image_data::snapshot(scope, source) else {
            crate::webidl::throw_type_error(scope, "The source is not ImageData");
            return;
        };
        (w, h)
    } else {
        (
            a.get(0).number_value(scope).unwrap_or(0.0).abs() as u32,
            a.get(1).number_value(scope).unwrap_or(0.0).abs() as u32,
        )
    };
    if width == 0 || height == 0 {
        throw_index_size(scope, "ImageData dimensions must be non-zero");
        return;
    }
    if let Ok(data) = super::image_data::create(
        scope,
        width,
        height,
        vec![0_u8; width as usize * height as usize * 4],
        "srgb",
    ) {
        r.set(data.into())
    }
}

fn parse_color(style: &PaintStyle, alpha: f64) -> [u8; 4] {
    let PaintStyle::Color(color) = style else {
        return [0, 0, 0, (alpha * 255.0) as u8];
    };
    let lower = color.trim().to_ascii_lowercase();
    let (rgb, style_alpha) = parse_canvas_color(&lower).unwrap_or(([0, 0, 0], 1.0));
    [
        rgb[0],
        rgb[1],
        rgb[2],
        (style_alpha * alpha.clamp(0.0, 1.0) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8,
    ]
}

fn parse_canvas_color(value: &str) -> Option<([u8; 3], f64)> {
    if value == "transparent" {
        return Some(([0, 0, 0], 0.0));
    }
    if let Some(hex) = value.strip_prefix('#') {
        let expanded;
        let hex = match hex.len() {
            3 | 4 => {
                expanded = hex
                    .chars()
                    .flat_map(|character| [character, character])
                    .collect::<String>();
                expanded.as_str()
            }
            6 | 8 => hex,
            _ => return None,
        };
        let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let alpha = if hex.len() == 8 {
            f64::from(u8::from_str_radix(&hex[6..8], 16).ok()?) / 255.0
        } else {
            1.0
        };
        return Some(([red, green, blue], alpha));
    }
    for (prefix, has_alpha) in [("rgba(", true), ("rgb(", false)] {
        let Some(body) = value
            .strip_prefix(prefix)
            .and_then(|body| body.strip_suffix(')'))
        else {
            continue;
        };
        let parts = body.split(',').map(str::trim).collect::<Vec<_>>();
        if parts.len() != if has_alpha { 4 } else { 3 } {
            return None;
        }
        let channel = |part: &str| -> Option<u8> {
            if let Some(percent) = part.strip_suffix('%') {
                return Some(
                    (percent.parse::<f64>().ok()? * 2.55)
                        .round()
                        .clamp(0.0, 255.0) as u8,
                );
            }
            Some(part.parse::<f64>().ok()?.round().clamp(0.0, 255.0) as u8)
        };
        let alpha = if has_alpha {
            let part = parts[3];
            if let Some(percent) = part.strip_suffix('%') {
                percent.parse::<f64>().ok()? / 100.0
            } else {
                part.parse::<f64>().ok()?
            }
        } else {
            1.0
        };
        return Some((
            [channel(parts[0])?, channel(parts[1])?, channel(parts[2])?],
            alpha.clamp(0.0, 1.0),
        ));
    }
    let rgb = match value {
        "black" => [0, 0, 0],
        "white" => [255, 255, 255],
        "red" => [255, 0, 0],
        "green" => [0, 128, 0],
        "lime" => [0, 255, 0],
        "blue" => [0, 0, 255],
        "yellow" => [255, 255, 0],
        "cyan" | "aqua" => [0, 255, 255],
        "magenta" | "fuchsia" => [255, 0, 255],
        "gray" | "grey" => [128, 128, 128],
        _ => return None,
    };
    Some((rgb, 1.0))
}

fn composite_pixel(destination: &mut [u8], source: [u8; 4], operation: &str) {
    match operation {
        "copy" => destination.copy_from_slice(&source),
        "lighter" => {
            let source_alpha = f64::from(source[3]) / 255.0;
            for channel in 0..3 {
                destination[channel] = (f64::from(destination[channel])
                    + f64::from(source[channel]) * source_alpha)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
            destination[3] = (u16::from(destination[3]) + u16::from(source[3])).min(255) as u8;
        }
        _ => blend_source_over(destination, &source, 1.0),
    }
}

fn paint_rect(record: &mut ContextRecord, x: f64, y: f64, w: f64, h: f64, color: [u8; 4]) {
    let x0 = x.floor().max(0.0) as u32;
    let y0 = y.floor().max(0.0) as u32;
    let x1 = (x + w).ceil().max(0.0).min(record.width as f64) as u32;
    let y1 = (y + h).ceil().max(0.0).min(record.height as f64) as u32;
    let operation = record.state.global_composite_operation.clone();
    for py in y0.min(record.height)..y1 {
        for px in x0.min(record.width)..x1 {
            let i = (py as usize * record.width as usize + px as usize) * 4;
            if i + 3 < record.pixels.len() {
                composite_pixel(&mut record.pixels[i..i + 4], color, &operation)
            }
        }
    }
}

fn clear_pixels(record: &mut ContextRecord, x: f64, y: f64, w: f64, h: f64) {
    let x0 = x.floor().max(0.0) as u32;
    let y0 = y.floor().max(0.0) as u32;
    let x1 = (x + w).ceil().max(0.0).min(record.width as f64) as u32;
    let y1 = (y + h).ceil().max(0.0).min(record.height as f64) as u32;
    for py in y0.min(record.height)..y1 {
        for px in x0.min(record.width)..x1 {
            let index = (py as usize * record.width as usize + px as usize) * 4;
            record.pixels[index..index + 4].fill(0);
        }
    }
}
fn fill_rect(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 4);
    update(scope, a.this(), |record| {
        let color = parse_color(&record.state.fill_style, record.state.global_alpha);
        paint_rect(record, v[0], v[1], v[2], v[3], color)
    })
}
fn clear_rect(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 4);
    update(scope, a.this(), |record| {
        clear_pixels(record, v[0], v[1], v[2], v[3])
    })
}
fn stroke_rect(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 4);
    update(scope, a.this(), |record| {
        let c = parse_color(&record.state.stroke_style, record.state.global_alpha);
        let line_width = record.state.line_width;
        let cap = record.state.line_cap.clone();
        let mut painted = HashSet::new();
        paint_line(
            record,
            (v[0], v[1]),
            (v[0] + v[2], v[1]),
            line_width,
            &cap,
            c,
            &mut painted,
        );
        paint_line(
            record,
            (v[0] + v[2], v[1]),
            (v[0] + v[2], v[1] + v[3]),
            line_width,
            &cap,
            c,
            &mut painted,
        );
        paint_line(
            record,
            (v[0] + v[2], v[1] + v[3]),
            (v[0], v[1] + v[3]),
            line_width,
            &cap,
            c,
            &mut painted,
        );
        paint_line(
            record,
            (v[0], v[1] + v[3]),
            (v[0], v[1]),
            line_width,
            &cap,
            c,
            &mut painted,
        )
    })
}

#[derive(Clone)]
struct RasterSubpath {
    points: Vec<(f64, f64)>,
    closed: bool,
}

fn transformed(transform: [f64; 6], point: (f64, f64)) -> (f64, f64) {
    transform_point(transform, point.0, point.1)
}

fn push_curve_point(path: &mut Vec<(f64, f64)>, transform: [f64; 6], point: (f64, f64)) {
    path.push(transformed(transform, point));
}

fn raster_subpaths(path: &[PathCommand], transform: [f64; 6]) -> Vec<RasterSubpath> {
    let mut output = Vec::new();
    let mut points: Vec<(f64, f64)> = Vec::new();
    let mut current = (0.0, 0.0);
    let mut start = current;
    let flush = |output: &mut Vec<RasterSubpath>, points: &mut Vec<(f64, f64)>, closed| {
        if points.len() > 1 {
            output.push(RasterSubpath {
                points: std::mem::take(points),
                closed,
            });
        } else {
            points.clear();
        }
    };
    for command in path {
        match *command {
            PathCommand::MoveTo(x, y) => {
                flush(&mut output, &mut points, false);
                current = (x, y);
                start = current;
                push_curve_point(&mut points, transform, current);
            }
            PathCommand::LineTo(x, y) => {
                if points.is_empty() {
                    start = current;
                    push_curve_point(&mut points, transform, current);
                }
                current = (x, y);
                push_curve_point(&mut points, transform, current);
            }
            PathCommand::Rect(x, y, width, height)
            | PathCommand::RoundRect(x, y, width, height) => {
                flush(&mut output, &mut points, false);
                output.push(RasterSubpath {
                    points: vec![
                        transformed(transform, (x, y)),
                        transformed(transform, (x + width, y)),
                        transformed(transform, (x + width, y + height)),
                        transformed(transform, (x, y + height)),
                    ],
                    closed: true,
                });
                current = (x, y);
                start = current;
            }
            PathCommand::Quadratic(cx, cy, x, y) => {
                if points.is_empty() {
                    start = current;
                    push_curve_point(&mut points, transform, current);
                }
                let from = current;
                for step in 1..=24 {
                    let t = f64::from(step) / 24.0;
                    let inverse = 1.0 - t;
                    push_curve_point(
                        &mut points,
                        transform,
                        (
                            inverse * inverse * from.0 + 2.0 * inverse * t * cx + t * t * x,
                            inverse * inverse * from.1 + 2.0 * inverse * t * cy + t * t * y,
                        ),
                    );
                }
                current = (x, y);
            }
            PathCommand::Bezier(c1x, c1y, c2x, c2y, x, y) => {
                if points.is_empty() {
                    start = current;
                    push_curve_point(&mut points, transform, current);
                }
                let from = current;
                for step in 1..=32 {
                    let t = f64::from(step) / 32.0;
                    let inverse = 1.0 - t;
                    push_curve_point(
                        &mut points,
                        transform,
                        (
                            inverse.powi(3) * from.0
                                + 3.0 * inverse * inverse * t * c1x
                                + 3.0 * inverse * t * t * c2x
                                + t.powi(3) * x,
                            inverse.powi(3) * from.1
                                + 3.0 * inverse * inverse * t * c1y
                                + 3.0 * inverse * t * t * c2y
                                + t.powi(3) * y,
                        ),
                    );
                }
                current = (x, y);
            }
            PathCommand::Arc(cx, cy, radius, start_angle, end_angle, anticlockwise) => {
                let sweep = angle_sweep(start_angle, end_angle, anticlockwise);
                for step in 0..=32 {
                    let angle = start_angle + sweep * f64::from(step) / 32.0;
                    let point = (cx + radius * angle.cos(), cy + radius * angle.sin());
                    if points.is_empty() {
                        start = point;
                    }
                    push_curve_point(&mut points, transform, point);
                    current = point;
                }
            }
            PathCommand::Ellipse(
                cx,
                cy,
                radius_x,
                radius_y,
                rotation,
                start_angle,
                end_angle,
                anticlockwise,
            ) => {
                let sweep = angle_sweep(start_angle, end_angle, anticlockwise);
                let (cos_rotation, sin_rotation) = (rotation.cos(), rotation.sin());
                for step in 0..=32 {
                    let angle = start_angle + sweep * f64::from(step) / 32.0;
                    let local_x = radius_x * angle.cos();
                    let local_y = radius_y * angle.sin();
                    let point = (
                        cx + local_x * cos_rotation - local_y * sin_rotation,
                        cy + local_x * sin_rotation + local_y * cos_rotation,
                    );
                    if points.is_empty() {
                        start = point;
                    }
                    push_curve_point(&mut points, transform, point);
                    current = point;
                }
            }
            PathCommand::ArcTo(x1, y1, x2, y2, _) => {
                if points.is_empty() {
                    start = current;
                    push_curve_point(&mut points, transform, current);
                }
                current = (x1, y1);
                push_curve_point(&mut points, transform, current);
                current = (x2, y2);
                push_curve_point(&mut points, transform, current);
            }
            PathCommand::Close => {
                if !points.is_empty() {
                    current = start;
                    flush(&mut output, &mut points, true);
                }
            }
        }
    }
    flush(&mut output, &mut points, false);
    output
}

fn angle_sweep(start: f64, end: f64, anticlockwise: bool) -> f64 {
    let tau = std::f64::consts::TAU;
    let mut sweep = end - start;
    if anticlockwise {
        while sweep > 0.0 {
            sweep -= tau;
        }
        sweep.max(-tau)
    } else {
        while sweep < 0.0 {
            sweep += tau;
        }
        sweep.min(tau)
    }
}

fn paint_line(
    record: &mut ContextRecord,
    from: (f64, f64),
    to: (f64, f64),
    line_width: f64,
    line_cap: &str,
    color: [u8; 4],
    painted: &mut HashSet<usize>,
) {
    let radius = (line_width.abs() / 2.0).max(0.5);
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let length_squared = dx * dx + dy * dy;
    let length = length_squared.sqrt();
    let extension = if line_cap == "square" { radius } else { 0.0 };
    let minimum_x = (from.0.min(to.0) - radius - extension).floor().max(0.0) as u32;
    let maximum_x = (from.0.max(to.0) + radius + extension)
        .ceil()
        .min(record.width as f64) as u32;
    let minimum_y = (from.1.min(to.1) - radius - extension).floor().max(0.0) as u32;
    let maximum_y = (from.1.max(to.1) + radius + extension)
        .ceil()
        .min(record.height as f64) as u32;
    let operation = record.state.global_composite_operation.clone();
    for y in minimum_y..maximum_y {
        for x in minimum_x..maximum_x {
            let point = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let projection = if length_squared > 0.0 {
                ((point.0 - from.0) * dx + (point.1 - from.1) * dy) / length_squared
            } else {
                0.0
            };
            let minimum_projection = if line_cap == "square" && length > 0.0 {
                -radius / length
            } else {
                0.0
            };
            let maximum_projection = if line_cap == "square" && length > 0.0 {
                1.0 + radius / length
            } else {
                1.0
            };
            let closest_projection = projection.clamp(0.0, 1.0);
            let closest = (
                from.0 + closest_projection * dx,
                from.1 + closest_projection * dy,
            );
            let distance_squared = (point.0 - closest.0).powi(2) + (point.1 - closest.1).powi(2);
            let inside = if line_cap == "round" {
                distance_squared <= radius * radius
            } else if projection >= minimum_projection && projection <= maximum_projection {
                let line_projection = projection.clamp(minimum_projection, maximum_projection);
                let line_point = (from.0 + line_projection * dx, from.1 + line_projection * dy);
                (point.0 - line_point.0).powi(2) + (point.1 - line_point.1).powi(2)
                    <= radius * radius
            } else {
                false
            };
            if inside {
                let index = (y as usize * record.width as usize + x as usize) * 4;
                if painted.insert(index) {
                    composite_pixel(&mut record.pixels[index..index + 4], color, &operation);
                }
            }
        }
    }
}

fn point_in_polygon(point: (f64, f64), polygon: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let mut previous = polygon.len() - 1;
    for current in 0..polygon.len() {
        let a = polygon[current];
        let b = polygon[previous];
        if (a.1 > point.1) != (b.1 > point.1)
            && point.0 < (b.0 - a.0) * (point.1 - a.1) / (b.1 - a.1) + a.0
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn paint_fill(record: &mut ContextRecord, polygon: &[(f64, f64)], color: [u8; 4]) {
    if polygon.len() < 3 {
        return;
    }
    let minimum_x = polygon
        .iter()
        .map(|point| point.0)
        .fold(f64::INFINITY, f64::min)
        .floor()
        .max(0.0) as u32;
    let maximum_x = polygon
        .iter()
        .map(|point| point.0)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        .min(record.width as f64) as u32;
    let minimum_y = polygon
        .iter()
        .map(|point| point.1)
        .fold(f64::INFINITY, f64::min)
        .floor()
        .max(0.0) as u32;
    let maximum_y = polygon
        .iter()
        .map(|point| point.1)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        .min(record.height as f64) as u32;
    let operation = record.state.global_composite_operation.clone();
    for y in minimum_y..maximum_y {
        for x in minimum_x..maximum_x {
            if point_in_polygon((f64::from(x) + 0.5, f64::from(y) + 0.5), polygon) {
                let index = (y as usize * record.width as usize + x as usize) * 4;
                composite_pixel(&mut record.pixels[index..index + 4], color, &operation);
            }
        }
    }
}
fn fill(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |record| {
        let color = parse_color(&record.state.fill_style, record.state.global_alpha);
        let paths = raster_subpaths(&record.path, record.state.transform);
        for path in paths {
            paint_fill(record, &path.points, color);
        }
    })
}
fn stroke(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |record| {
        let color = parse_color(&record.state.stroke_style, record.state.global_alpha);
        let line_width = record.state.line_width;
        let line_cap = record.state.line_cap.clone();
        let paths = raster_subpaths(&record.path, record.state.transform);
        let mut painted = HashSet::new();
        for path in paths {
            for segment in path.points.windows(2) {
                paint_line(
                    record,
                    segment[0],
                    segment[1],
                    line_width,
                    &line_cap,
                    color,
                    &mut painted,
                );
            }
            if path.closed && path.points.len() > 2 {
                paint_line(
                    record,
                    *path.points.last().unwrap_or(&(0.0, 0.0)),
                    path.points[0],
                    line_width,
                    &line_cap,
                    color,
                    &mut painted,
                );
            }
        }
    })
}
fn clip(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn fill_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, a.get(0));
    let x = a.get(1).number_value(scope).unwrap_or(0.0);
    let y = a.get(2).number_value(scope).unwrap_or(0.0);
    let maximum_width = a
        .get(3)
        .number_value(scope)
        .filter(|value| value.is_finite() && *value > 0.0);
    let profile_scale = crate::fingerprint::edge(scope)
        .rendering
        .canvas
        .text_width_scale;
    update(scope, a.this(), |record| {
        let color = parse_color(&record.state.fill_style, record.state.global_alpha);
        let polygon = text_ink_polygon(record, &text, x, y, maximum_width, profile_scale);
        paint_fill(record, &polygon, color)
    })
}
fn stroke_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, a.get(0));
    let x = a.get(1).number_value(scope).unwrap_or(0.0);
    let y = a.get(2).number_value(scope).unwrap_or(0.0);
    let maximum_width = a
        .get(3)
        .number_value(scope)
        .filter(|value| value.is_finite() && *value > 0.0);
    let profile_scale = crate::fingerprint::edge(scope)
        .rendering
        .canvas
        .text_width_scale;
    update(scope, a.this(), |record| {
        let color = parse_color(&record.state.stroke_style, record.state.global_alpha);
        let polygon = text_ink_polygon(record, &text, x, y, maximum_width, profile_scale);
        let mut painted = HashSet::new();
        for index in 0..polygon.len() {
            paint_line(
                record,
                polygon[index],
                polygon[(index + 1) % polygon.len()],
                record.state.line_width,
                &record.state.line_cap.clone(),
                color,
                &mut painted,
            );
        }
    })
}

fn text_ink_polygon(
    record: &ContextRecord,
    text: &str,
    x: f64,
    y: f64,
    maximum_width: Option<f64>,
    profile_scale: f64,
) -> Vec<(f64, f64)> {
    let font_size = canvas_font_size(&record.state.font);
    let mut width = text.chars().count() as f64 * font_size * 0.6 * profile_scale;
    if let Some(maximum_width) = maximum_width {
        width = width.min(maximum_width);
    }
    let aligned_x = match record.state.text_align.as_str() {
        "center" => x - width / 2.0,
        "right" => x - width,
        "end" if record.state.direction != "rtl" => x - width,
        "start" if record.state.direction == "rtl" => x - width,
        _ => x,
    };
    let ascent = font_size * 0.8;
    let descent = font_size * 0.2;
    let top = match record.state.text_baseline.as_str() {
        "top" => y,
        "hanging" => y - font_size * 0.2,
        "middle" => y - font_size / 2.0,
        "bottom" => y - font_size,
        "ideographic" => y - font_size * 0.9,
        _ => y - ascent,
    };
    let bottom = if record.state.text_baseline == "alphabetic" {
        y + descent
    } else {
        top + font_size
    };
    [
        (aligned_x, top),
        (aligned_x + width, top),
        (aligned_x + width, bottom),
        (aligned_x, bottom),
    ]
    .into_iter()
    .map(|point| transformed(record.state.transform, point))
    .collect()
}

fn get_image_data(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let v = values(scope, &a, 4);
    let width = v[2].abs() as u32;
    let height = v[3].abs() as u32;
    if width == 0 || height == 0 {
        throw_index_size(scope, "The source dimensions are zero");
        return;
    }
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut bytes = vec![0_u8; width as usize * height as usize * 4];
    let sx = v[0] as i32;
    let sy = v[1] as i32;
    for y in 0..height {
        for x in 0..width {
            let source_x = sx + x as i32;
            let source_y = sy + y as i32;
            if source_x >= 0
                && source_y >= 0
                && (source_x as u32) < record.width
                && (source_y as u32) < record.height
            {
                let si = (source_y as usize * record.width as usize + source_x as usize) * 4;
                let di = (y as usize * width as usize + x as usize) * 4;
                bytes[di..di + 4].copy_from_slice(&record.pixels[si..si + 4])
            }
        }
    }
    if let Ok(data) = super::image_data::create(scope, width, height, bytes, &record.color_space) {
        r.set(data.into())
    }
}
fn put_image_data(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let Ok(data) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "The source must be ImageData");
        return;
    };
    let Some((width, height, bytes)) = super::image_data::snapshot(scope, data) else {
        crate::webidl::throw_type_error(scope, "The source must be ImageData");
        return;
    };
    let dx = a.get(1).number_value(scope).unwrap_or(0.0) as i32;
    let dy = a.get(2).number_value(scope).unwrap_or(0.0) as i32;
    update(scope, a.this(), |record| {
        for y in 0..height {
            for x in 0..width {
                let tx = dx + x as i32;
                let ty = dy + y as i32;
                if tx >= 0 && ty >= 0 && (tx as u32) < record.width && (ty as u32) < record.height {
                    let si = (y as usize * width as usize + x as usize) * 4;
                    let di = (ty as usize * record.width as usize + tx as usize) * 4;
                    record.pixels[di..di + 4].copy_from_slice(&bytes[si..si + 4])
                }
            }
        }
    })
}
fn draw_image(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    if a.length() < 3 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'drawImage': 3 arguments required",
        );
        return;
    }
    let Ok(source) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "The image source is invalid");
        return;
    };
    let (source_width, source_height, pixels) =
        match super::create_image_bitmap_global::source_pixels(scope, source) {
            Ok(snapshot) => snapshot,
            Err(super::create_image_bitmap_global::CanvasImageSourceError::InvalidType) => {
                crate::webidl::throw_type_error(scope, "The image source is invalid");
                return;
            }
            Err(
                super::create_image_bitmap_global::CanvasImageSourceError::Unusable
                | super::create_image_bitmap_global::CanvasImageSourceError::Decode,
            ) => {
                let exception = super::dom_exception::create(
                    scope,
                    "The image source is not usable.".to_owned(),
                    "InvalidStateError".to_owned(),
                )
                .map(Into::into)
                .unwrap_or_else(|_| v8::undefined(scope).into());
                scope.throw_exception(exception);
                return;
            }
        };
    let values = if a.length() >= 9 {
        [
            a.get(1).number_value(scope).unwrap_or(f64::NAN),
            a.get(2).number_value(scope).unwrap_or(f64::NAN),
            a.get(3).number_value(scope).unwrap_or(f64::NAN),
            a.get(4).number_value(scope).unwrap_or(f64::NAN),
            a.get(5).number_value(scope).unwrap_or(f64::NAN),
            a.get(6).number_value(scope).unwrap_or(f64::NAN),
            a.get(7).number_value(scope).unwrap_or(f64::NAN),
            a.get(8).number_value(scope).unwrap_or(f64::NAN),
        ]
    } else if a.length() >= 5 {
        [
            0.0,
            0.0,
            f64::from(source_width),
            f64::from(source_height),
            a.get(1).number_value(scope).unwrap_or(f64::NAN),
            a.get(2).number_value(scope).unwrap_or(f64::NAN),
            a.get(3).number_value(scope).unwrap_or(f64::NAN),
            a.get(4).number_value(scope).unwrap_or(f64::NAN),
        ]
    } else {
        [
            0.0,
            0.0,
            f64::from(source_width),
            f64::from(source_height),
            a.get(1).number_value(scope).unwrap_or(f64::NAN),
            a.get(2).number_value(scope).unwrap_or(f64::NAN),
            f64::from(source_width),
            f64::from(source_height),
        ]
    };
    if values.iter().any(|value| !value.is_finite()) {
        return;
    }
    let [
        mut source_x,
        mut source_y,
        mut source_draw_width,
        mut source_draw_height,
        mut destination_x,
        mut destination_y,
        mut destination_width,
        mut destination_height,
    ] = values;
    normalize_negative_rectangle(
        &mut source_x,
        &mut source_y,
        &mut source_draw_width,
        &mut source_draw_height,
    );
    normalize_negative_rectangle(
        &mut destination_x,
        &mut destination_y,
        &mut destination_width,
        &mut destination_height,
    );
    if source_draw_width == 0.0
        || source_draw_height == 0.0
        || destination_width == 0.0
        || destination_height == 0.0
    {
        return;
    }
    update(scope, a.this(), |record| {
        let output_width = destination_width.ceil().max(0.0) as u32;
        let output_height = destination_height.ceil().max(0.0) as u32;
        if output_width
            .checked_mul(output_height)
            .is_none_or(|pixels| pixels > 16 * 1024 * 1024)
        {
            return;
        }
        for output_y in 0..output_height {
            for output_x in 0..output_width {
                let input_x = (source_x
                    + f64::from(output_x) * source_draw_width / destination_width)
                    .floor();
                let input_y = (source_y
                    + f64::from(output_y) * source_draw_height / destination_height)
                    .floor();
                if input_x < 0.0
                    || input_y < 0.0
                    || input_x >= f64::from(source_width)
                    || input_y >= f64::from(source_height)
                {
                    continue;
                }
                let canvas_point = transform_point(
                    record.state.transform,
                    destination_x + f64::from(output_x),
                    destination_y + f64::from(output_y),
                );
                let target_x = canvas_point.0.floor() as i32;
                let target_y = canvas_point.1.floor() as i32;
                if target_x < 0
                    || target_y < 0
                    || target_x as u32 >= record.width
                    || target_y as u32 >= record.height
                {
                    continue;
                }
                let source_offset =
                    (input_y as usize * source_width as usize + input_x as usize) * 4;
                let destination_offset =
                    (target_y as usize * record.width as usize + target_x as usize) * 4;
                blend_source_over(
                    &mut record.pixels[destination_offset..destination_offset + 4],
                    &pixels[source_offset..source_offset + 4],
                    record.state.global_alpha,
                );
            }
        }
    })
}

fn normalize_negative_rectangle(x: &mut f64, y: &mut f64, width: &mut f64, height: &mut f64) {
    if *width < 0.0 {
        *x += *width;
        *width = -*width;
    }
    if *height < 0.0 {
        *y += *height;
        *height = -*height;
    }
}

fn transform_point(transform: [f64; 6], x: f64, y: f64) -> (f64, f64) {
    (
        transform[0] * x + transform[2] * y + transform[4],
        transform[1] * x + transform[3] * y + transform[5],
    )
}

fn blend_source_over(destination: &mut [u8], source: &[u8], global_alpha: f64) {
    let source_alpha = f64::from(source[3]) / 255.0 * global_alpha.clamp(0.0, 1.0);
    let destination_alpha = f64::from(destination[3]) / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    if output_alpha <= 0.0 {
        destination.fill(0);
        return;
    }
    for channel in 0..3 {
        let source_value = f64::from(source[channel]) / 255.0;
        let destination_value = f64::from(destination[channel]) / 255.0;
        destination[channel] = ((source_value * source_alpha
            + destination_value * destination_alpha * (1.0 - source_alpha))
            / output_alpha
            * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    destination[3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

fn get_line_dash(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        let array = v8::Array::new(scope, record.state.line_dash.len() as i32);
        for (index, value) in record.state.line_dash.iter().enumerate() {
            let _ = array.set_index(scope, index as u32, v8::Number::new(scope, *value).into());
        }
        r.set(array.into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_line_dash(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_context(scope, a.this()) {
        return;
    }
    let Ok(sequence) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "The line dash must be a sequence");
        return;
    };
    let Some(key) = v8::String::new(scope, "length") else {
        return;
    };
    let length = sequence
        .get(scope, key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let mut dash = Vec::with_capacity(length as usize * 2);
    for index in 0..length {
        let value = sequence
            .get_index(scope, index)
            .and_then(|value| value.number_value(scope))
            .unwrap_or(f64::NAN);
        if !value.is_finite() || value < 0.0 {
            throw_index_size(scope, "Line dash values must be finite and non-negative");
            return;
        }
        dash.push(value)
    }
    if dash.len() % 2 == 1 {
        let copy = dash.clone();
        dash.extend(copy)
    }
    update(scope, a.this(), |record| record.state.line_dash = dash)
}
fn get_transform(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Ok(matrix) = super::dom_matrix::create_2d(scope, record.state.transform) {
        r.set(matrix.into())
    }
}
fn multiply(left: [f64; 6], right: [f64; 6]) -> [f64; 6] {
    [
        left[0] * right[0] + left[2] * right[1],
        left[1] * right[0] + left[3] * right[1],
        left[0] * right[2] + left[2] * right[3],
        left[1] * right[2] + left[3] * right[3],
        left[0] * right[4] + left[2] * right[5] + left[4],
        left[1] * right[4] + left[3] * right[5] + left[5],
    ]
}
fn apply_transform(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    matrix: [f64; 6],
) {
    update(scope, object, |record| {
        record.state.transform = multiply(record.state.transform, matrix)
    })
}
fn translate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 2);
    apply_transform(scope, a.this(), [1.0, 0.0, 0.0, 1.0, v[0], v[1]])
}
fn scale(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 2);
    apply_transform(scope, a.this(), [v[0], 0.0, 0.0, v[1], 0.0, 0.0])
}
fn rotate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let angle = a.get(0).number_value(scope).unwrap_or(0.0);
    let (c, s) = (angle.cos(), angle.sin());
    apply_transform(scope, a.this(), [c, s, -s, c, 0.0, 0.0])
}
fn transform(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = values(scope, &a, 6);
    apply_transform(scope, a.this(), [v[0], v[1], v[2], v[3], v[4], v[5]])
}
fn set_transform(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let matrix = if a.length() == 0 {
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    } else if a.length() == 1 {
        let Ok(object) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
            crate::webidl::throw_type_error(scope, "The transform must be a matrix");
            return;
        };
        [
            super::event::number_property(scope, object, "a", 1.0),
            super::event::number_property(scope, object, "b", 0.0),
            super::event::number_property(scope, object, "c", 0.0),
            super::event::number_property(scope, object, "d", 1.0),
            super::event::number_property(scope, object, "e", 0.0),
            super::event::number_property(scope, object, "f", 0.0),
        ]
    } else {
        let v = values(scope, &a, 6);
        [v[0], v[1], v[2], v[3], v[4], v[5]]
    };
    update(scope, a.this(), |record| record.state.transform = matrix)
}
fn reset_transform(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |record| {
        record.state.transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    })
}
fn save(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |record| {
        record.stack.push(record.state.clone())
    })
}
fn restore(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |record| {
        if let Some(state) = record.stack.pop() {
            record.state = state
        }
    })
}
fn reset(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, a.this(), |record| {
        record.state = CanvasState::default();
        record.stack.clear();
        record.path.clear();
        record.pixels.fill(0)
    })
}
fn is_context_lost(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(v8::Boolean::new(scope, false).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn point_in_rect(path: &[PathCommand], x: f64, y: f64) -> bool {
    path.iter().any(|command| match command {
        PathCommand::Rect(rx, ry, w, h) | PathCommand::RoundRect(rx, ry, w, h) => {
            x >= *rx && x <= *rx + *w && y >= *ry && y <= *ry + *h
        }
        _ => false,
    })
}
fn is_point_in_path(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let x = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    let y = a.get(1).number_value(scope).unwrap_or(f64::NAN);
    r.set(v8::Boolean::new(scope, point_in_rect(&record.path, x, y)).into())
}
fn is_point_in_stroke(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    is_point_in_path(scope, a, r)
}
fn measure_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let text = crate::webidl::value_to_string(scope, a.get(0));
    let canvas = &crate::fingerprint::edge(scope).rendering.canvas;
    let font_scale = canvas_font_size(&record.state.font) / 10.0;
    let (_, monospace) = canvas_font_metrics(scope, &record.state.font);
    let shaped = shaped_metrics_for_state(scope, &text, &record.state);
    let width = shaped
        .map(|metrics| metrics.advance)
        .unwrap_or_else(|| measured_text_width_for_state(scope, &text, &record.state));
    let has_ink = !text.is_empty();
    let font_size = canvas_font_size(&record.state.font);
    let native_windows_metrics = uses_native_windows_text_metrics(scope, canvas);
    let unicode_ink = native_windows_metrics
        .then(|| {
            mixed_script_horizontal_ink_bounds(&record.state.font, &text, font_size)
                .or_else(|| unicode_horizontal_ink_bounds(&text, font_size))
        })
        .flatten();
    let native_ink = unicode_ink.or_else(|| {
        native_windows_metrics.then(|| {
            super::font_metric_tables::ascii_text_ink_bounds(
                &record.state.font,
                &text,
                font_size,
                width,
            )
        })?
    });
    let glyph_left = if let Some(metrics) = shaped {
        metrics.actual_left
    } else if let Some((left, _)) = native_ink {
        left
    } else if monospace && text.starts_with('W') {
        0.5 * font_scale
    } else {
        canvas.actual_bounding_box_left * font_scale
    };
    let glyph_right = shaped
        .map(|metrics| metrics.actual_right)
        .or_else(|| native_ink.map(|(_, right)| right))
        .unwrap_or_else(|| {
            width * canvas.actual_bounding_box_right_scale
                - trailing_glyph_inset(&text, monospace) * font_scale
        });
    let (alignment_left, alignment_right) =
        aligned_text_bounds(width, glyph_left, glyph_right, &record.state);
    let native_vertical = native_windows_metrics
        .then(|| windows_vertical_text_metrics(&record.state.font, &text, font_size))
        .flatten();
    let font_ascent = if let Some(metrics) = shaped {
        metrics.font_ascent
    } else if let Some(metrics) = native_vertical {
        metrics.font_ascent
    } else if monospace {
        font_size * 0.85
    } else {
        canvas.font_bounding_box_ascent * font_scale
    };
    let font_descent = if let Some(metrics) = shaped {
        metrics.font_descent
    } else if let Some(metrics) = native_vertical {
        metrics.font_descent
    } else if monospace {
        font_size * 0.15
    } else {
        canvas.font_bounding_box_descent * font_scale
    };
    let actual_ascent = if let Some(metrics) = shaped {
        metrics.actual_ascent
    } else if let Some(metrics) = native_vertical {
        metrics.actual_ascent
    } else if monospace {
        font_size * 0.65
    } else {
        canvas.actual_bounding_box_ascent * font_scale
    };
    let actual_descent = if let Some(metrics) = shaped {
        metrics.actual_descent
    } else if let Some(metrics) = native_vertical {
        metrics.actual_descent
    } else if monospace {
        0.0
    } else {
        canvas.actual_bounding_box_descent * font_scale
    };
    let metrics = super::text_metrics::TextMetricsRecord {
        width,
        actual_bounding_box_left: if has_ink { alignment_left } else { 0.0 },
        actual_bounding_box_right: if has_ink { alignment_right } else { 0.0 },
        font_bounding_box_ascent: font_ascent,
        font_bounding_box_descent: font_descent,
        // Blink preserves the signed zero produced by its baseline-relative
        // ink calculation.  `Object.is(measureText("").actualBoundingBoxAscent,
        // -0)` is true in Edge even though ordinary numeric equality hides it.
        actual_bounding_box_ascent: if has_ink { actual_ascent } else { -0.0 },
        actual_bounding_box_descent: if has_ink { actual_descent } else { 0.0 },
        hanging_baseline: if let Some(metrics) = shaped {
            (metrics.font_ascent * 0.8) as f32 as f64
        } else if let Some(metrics) = native_vertical {
            metrics.hanging_baseline
        } else if monospace {
            font_size * 0.68
        } else {
            canvas.hanging_baseline * font_scale
        },
        alphabetic_baseline: if shaped.is_some() || native_vertical.is_some() {
            -0.0
        } else {
            let baseline = canvas.alphabetic_baseline * font_scale;
            if baseline == 0.0 { -0.0 } else { baseline }
        },
        ideographic_baseline: if let Some(metrics) = shaped {
            -metrics.font_descent
        } else if let Some(metrics) = native_vertical {
            -metrics.font_descent
        } else if monospace {
            -font_size * 0.15
        } else {
            canvas.ideographic_baseline * font_scale
        },
    };
    if let Ok(value) = super::text_metrics::create(scope, metrics) {
        r.set(value.into())
    }
}

#[derive(Clone, Copy)]
struct NativeVerticalTextMetrics {
    font_ascent: f64,
    font_descent: f64,
    actual_ascent: f64,
    actual_descent: f64,
    hanging_baseline: f64,
}

fn uses_native_windows_text_metrics(
    scope: &v8::PinScope<'_, '_>,
    canvas: &crate::fingerprint_surface::CanvasFingerprint,
) -> bool {
    let fingerprint = crate::fingerprint::edge(scope);
    fingerprint
        .navigator
        .platform
        .to_ascii_lowercase()
        .starts_with("win")
        && approximately(canvas.font_bounding_box_ascent, 9.0)
        && approximately(canvas.font_bounding_box_descent, 2.0)
        && approximately(canvas.actual_bounding_box_ascent, 7.0)
        && approximately(canvas.actual_bounding_box_descent, 2.0)
        && approximately(canvas.hanging_baseline, 7.199_999_809_265_137)
        && approximately(canvas.ideographic_baseline, -2.0)
}

fn approximately(left: f64, right: f64) -> bool {
    (left - right).abs() <= f64::EPSILON * 8.0
}

fn windows_vertical_text_metrics(
    font: &str,
    text: &str,
    font_size: f64,
) -> Option<NativeVerticalTextMetrics> {
    let family = font.to_ascii_lowercase();
    let (font_ascent, font_descent, ascent_factor, descent_factor) = if family.contains("segoe ui")
    {
        (
            (font_size * 1.08).round(),
            (font_size * 0.25).round(),
            if text
                .chars()
                .any(|character| matches!(character, 'f' | 'l' | 't'))
            {
                0.75
            } else {
                0.71
            },
            0.23,
        )
    } else if family.contains("times new roman") {
        (
            (font_size * 0.9).round(),
            (font_size * 0.2).round(),
            if text.chars().any(|character| matches!(character, 'f' | 'l')) {
                0.70
            } else {
                0.67
            },
            0.20,
        )
    } else if family.contains("arial") {
        (
            (font_size * 0.9).round(),
            (font_size * 0.2).round(),
            0.73,
            0.20,
        )
    } else {
        return None;
    };
    let (actual_ascent, actual_descent) =
        windows_actual_ink_height(text, font_size, ascent_factor, descent_factor);
    Some(NativeVerticalTextMetrics {
        font_ascent,
        font_descent,
        actual_ascent,
        actual_descent,
        hanging_baseline: (font_ascent * 0.8) as f32 as f64,
    })
}

fn windows_actual_ink_height(
    text: &str,
    font_size: f64,
    ascent_factor: f64,
    descent_factor: f64,
) -> (f64, f64) {
    if text.is_empty() {
        return (-0.0, 0.0);
    }
    let codepoints = text
        .chars()
        .map(|character| character as u32)
        .collect::<Vec<_>>();
    if codepoints
        .iter()
        .any(|codepoint| (0x1F000..=0x1FAFF).contains(codepoint))
    {
        if codepoints.len() == 2
            && codepoints
                .iter()
                .all(|codepoint| (0x1F1E6..=0x1F1FF).contains(codepoint))
        {
            return ((font_size * 0.5625).round(), (font_size * 0.0625).round());
        }
        return ((font_size * 0.875).round(), (font_size * 0.1875).round());
    }
    if text.chars().any(is_full_width_fallback) {
        let descent = if text
            .chars()
            .any(|character| matches!(character as u32, 0x3040..=0x30FF))
        {
            (font_size * 0.0625).round()
        } else {
            (font_size * 0.125).round()
        };
        return ((font_size * 0.8125).round(), descent);
    }
    if text == "\u{0645}" {
        return ((font_size * 0.3125).round(), (font_size * 0.1875).round());
    }
    let ascent = if font_size <= 10.0 {
        7.0
    } else {
        (font_size * ascent_factor).round()
    };
    let descent = if text
        .chars()
        .any(|character| matches!(character, 'g' | 'j' | 'p' | 'q' | 'y'))
    {
        (font_size * descent_factor).round()
    } else {
        0.0
    };
    (ascent, descent)
}

fn unicode_horizontal_ink_bounds(text: &str, font_size: f64) -> Option<(f64, f64)> {
    let mut graphemes = text.graphemes(true);
    let grapheme = graphemes.next()?;
    if graphemes.next().is_some() {
        return None;
    }
    let codepoints = grapheme
        .chars()
        .map(|character| character as u32)
        .collect::<Vec<_>>();
    if codepoints.len() == 2
        && codepoints
            .iter()
            .all(|codepoint| (0x1F1E6..=0x1F1FF).contains(codepoint))
    {
        return Some((0.0, font_size * 1.024_414_062_5));
    }
    if codepoints.contains(&0x200D)
        && codepoints
            .iter()
            .any(|codepoint| (0x1F000..=0x1FAFF).contains(codepoint))
    {
        return Some((font_size * -0.082_519_531_25, font_size * 1.186_035_156_25));
    }
    if codepoints.len() == 1 && is_emoji_presentation(grapheme.chars().next()?) {
        let right_factor = if codepoints[0] == 0x1F680 {
            1.25
        } else {
            1.1875
        };
        return Some((font_size * -0.125, font_size * right_factor));
    }
    if codepoints.len() == 1 && is_full_width_fallback(grapheme.chars().next()?) {
        return Some((font_size * -0.0625, font_size * 0.9375));
    }
    if grapheme == "\u{0645}" {
        return Some((0.0, font_size * 0.375));
    }
    if codepoints
        .iter()
        .any(|codepoint| char::from_u32(*codepoint).is_some_and(is_combining_mark))
    {
        let visible = grapheme
            .chars()
            .filter(|character| !is_combining_mark(*character))
            .collect::<String>();
        if visible.is_ascii() && !visible.is_empty() {
            return Some((0.0, measured_ascii_ink_right(&visible, font_size)));
        }
    }
    None
}

fn mixed_script_horizontal_ink_bounds(
    font: &str,
    text: &str,
    font_size: f64,
) -> Option<(f64, f64)> {
    if text.is_empty()
        || text
            .chars()
            .all(|character| (' '..='~').contains(&character))
        || text.graphemes(true).count() < 2
    {
        return None;
    }
    let mut cursor = 0.0;
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    let mut ascii_run = String::new();
    let flush_ascii =
        |run: &mut String, cursor: &mut f64, minimum: &mut f64, maximum: &mut f64| -> Option<()> {
            if run.is_empty() {
                return Some(());
            }
            let advance = super::font_metric_tables::ascii_advance_width(font, run, font_size)?;
            let (left, right) =
                super::font_metric_tables::ascii_text_ink_bounds(font, run, font_size, advance)?;
            *minimum = minimum.min(*cursor - left);
            *maximum = maximum.max(*cursor + right);
            *cursor += advance;
            run.clear();
            Some(())
        };
    for grapheme in text.graphemes(true) {
        if grapheme
            .chars()
            .all(|character| (' '..='~').contains(&character))
        {
            ascii_run.push_str(grapheme);
            continue;
        }
        flush_ascii(&mut ascii_run, &mut cursor, &mut minimum, &mut maximum)?;
        let (left, right) = unicode_horizontal_ink_bounds(grapheme, font_size)?;
        minimum = minimum.min(cursor - left);
        maximum = maximum.max(cursor + right);
        cursor += grapheme_advance_10(font, grapheme) * font_size / 10.0;
    }
    flush_ascii(&mut ascii_run, &mut cursor, &mut minimum, &mut maximum)?;
    (minimum.is_finite() && maximum.is_finite()).then_some((-minimum, maximum))
}

fn measured_ascii_ink_right(text: &str, font_size: f64) -> f64 {
    super::font_metric_tables::ascii_advance_width("Arial", text, font_size)
        .unwrap_or(0.0)
        .ceil()
}

pub(crate) fn measured_text_width_for_font(
    scope: &v8::PinScope<'_, '_>,
    text: &str,
    font: &str,
) -> f64 {
    let mut state = CanvasState::default();
    state.font = font.to_owned();
    measured_text_width_for_state(scope, text, &state)
}

pub(crate) fn measured_inline_text_width_for_font(
    scope: &v8::PinScope<'_, '_>,
    text: &str,
    font: &str,
    implicit_default: bool,
) -> f64 {
    if implicit_default {
        let font_size = canvas_font_size(font);
        if let Some(width) =
            super::font_metric_tables::implicit_default_advance_width(text, font_size)
        {
            let canvas = &crate::fingerprint::edge(scope).rendering.canvas;
            let (configured_width_scale, _) = canvas_font_metrics(scope, font);
            return width * canvas.text_width_scale * configured_width_scale;
        }
    }
    measured_text_width_for_font(scope, text, font)
}

fn measured_text_width_for_state(
    scope: &v8::PinScope<'_, '_>,
    text: &str,
    state: &CanvasState,
) -> f64 {
    if let Some(metrics) = shaped_metrics_for_state(scope, text, state) {
        let characters = text.graphemes(true).count() as f64;
        let spaces = text
            .chars()
            .filter(|character| character.is_whitespace())
            .count() as f64;
        return metrics.advance
            + characters * canvas_spacing(&state.letter_spacing)
            + spaces * canvas_spacing(&state.word_spacing);
    }
    let canvas = &crate::fingerprint::edge(scope).rendering.canvas;
    let font_size = canvas_font_size(&state.font);
    let font_scale = font_size / 10.0;
    let (configured_width_scale, monospace) = canvas_font_metrics(scope, &state.font);
    if !monospace
        && let Some(width) =
            super::font_metric_tables::ascii_advance_width(&state.font, text, font_size)
    {
        let characters = text.chars().count() as f64;
        let spaces = text
            .chars()
            .filter(|character| character.is_whitespace())
            .count() as f64;
        return width * canvas.text_width_scale * configured_width_scale
            + characters * canvas_spacing(&state.letter_spacing)
            + spaces * canvas_spacing(&state.word_spacing);
    }
    if !monospace
        && text
            .chars()
            .any(|character| !((' '..='~').contains(&character)))
        && let Some(width) = mixed_script_advance_width(&state.font, text, font_size)
    {
        let characters = text.chars().count() as f64;
        let spaces = text
            .chars()
            .filter(|character| character.is_whitespace())
            .count() as f64;
        return width * canvas.text_width_scale * configured_width_scale
            + characters * canvas_spacing(&state.letter_spacing)
            + spaces * canvas_spacing(&state.word_spacing);
    }
    measured_text_width(text, state, monospace)
        * canvas.text_width_scale
        * configured_width_scale
        * font_scale
}

fn shaped_metrics_for_state(
    scope: &v8::PinScope<'_, '_>,
    text: &str,
    state: &CanvasState,
) -> Option<crate::font_shaping::ShapeMetrics> {
    let direction = if state.direction == "rtl" {
        rustybuzz::Direction::RightToLeft
    } else {
        rustybuzz::Direction::LeftToRight
    };
    let mut metrics = crate::font_shaping::metrics_with_features(
        scope,
        text,
        &state.font,
        direction,
        state.font_kerning != "none",
        &state.font_variant_caps,
        &state.font_stretch,
    )?;
    apply_shaped_width_scale(scope, &state.font, &mut metrics);
    Some(metrics)
}

pub(crate) fn shaped_font_metrics(
    scope: &v8::PinScope<'_, '_>,
    text: &str,
    font: &str,
    rtl: bool,
) -> Option<crate::font_shaping::ShapeMetrics> {
    let mut metrics = crate::font_shaping::dom_metrics(
        scope,
        text,
        font,
        if rtl {
            rustybuzz::Direction::RightToLeft
        } else {
            rustybuzz::Direction::LeftToRight
        },
    )?;
    apply_shaped_width_scale(scope, font, &mut metrics);
    Some(metrics)
}

fn apply_shaped_width_scale(
    scope: &v8::PinScope<'_, '_>,
    font: &str,
    metrics: &mut crate::font_shaping::ShapeMetrics,
) {
    let canvas = &crate::fingerprint::edge(scope).rendering.canvas;
    let (configured, _) = canvas_font_metrics(scope, font);
    let scale = canvas.text_width_scale * configured;
    metrics.advance *= scale;
    metrics.actual_left *= scale;
    metrics.actual_right *= scale;
}

fn mixed_script_advance_width(font: &str, text: &str, font_size: f64) -> Option<f64> {
    let mut width = 0.0;
    let mut ascii_run = String::new();
    let flush_ascii = |run: &mut String, width: &mut f64| -> Option<()> {
        if !run.is_empty() {
            *width += super::font_metric_tables::ascii_advance_width(font, run, font_size)?;
            run.clear();
        }
        Some(())
    };
    for grapheme in text.graphemes(true) {
        if grapheme
            .chars()
            .all(|character| (' '..='~').contains(&character))
        {
            ascii_run.push_str(grapheme);
            continue;
        }
        flush_ascii(&mut ascii_run, &mut width)?;
        width += grapheme_advance_10(font, grapheme) * font_size / 10.0;
    }
    flush_ascii(&mut ascii_run, &mut width)?;
    Some(width)
}

fn grapheme_advance_10(font: &str, grapheme: &str) -> f64 {
    let codepoints = grapheme
        .chars()
        .map(|character| character as u32)
        .collect::<Vec<_>>();
    if codepoints.len() == 2
        && codepoints
            .iter()
            .all(|codepoint| (0x1F1E6..=0x1F1FF).contains(codepoint))
    {
        // Segoe UI Emoji shapes a pair of regional indicators as one flag.
        return 10.478_515_625;
    }
    if codepoints.contains(&0x200D)
        && codepoints
            .iter()
            .any(|codepoint| (0x1F000..=0x1FAFF).contains(codepoint))
    {
        // The common family/person ZWJ presentation captured from Edge 150.
        return 12.529_296_875;
    }
    let visible = grapheme
        .chars()
        .filter(|character| {
            !is_combining_mark(*character)
                && *character != '\u{200D}'
                && !matches!(*character as u32, 0x1F3FB..=0x1F3FF)
        })
        .collect::<String>();
    if font.to_ascii_lowercase().contains("arial")
        && let Some(character) = visible.chars().next()
        && visible.chars().count() == 1
        && let Some(width) = arial_hebrew_advance_10(character)
    {
        return width;
    }
    if visible
        .chars()
        .all(|character| (' '..='~').contains(&character))
        && let Some(width) = super::font_metric_tables::ascii_advance_width(font, &visible, 10.0)
    {
        return width;
    }
    visible.chars().map(generic_character_advance_10).sum()
}

fn arial_hebrew_advance_10(character: char) -> Option<f64> {
    // Arial advances captured from Edge 150/DirectWrite at 16 CSS px and
    // normalized to the 10px reference size used by this text metric path.
    const ADVANCES: [f64; 27] = [
        5.629_882_812_5,
        5.419_921_875,
        3.989_257_812_5,
        5.083_007_812_5,
        6.020_507_812_5,
        2.465_820_312_5,
        3.823_242_187_5,
        5.986_328_125,
        5.898_437_5,
        2.465_820_312_5,
        5.092_773_437_5,
        4.609_375,
        4.628_906_25,
        5.986_328_125,
        6.010_742_187_5,
        2.465_820_312_5,
        3.525_390_625,
        5.742_187_5,
        5.292_968_75,
        5.664_062_5,
        5.463_867_187_5,
        4.614_257_812_5,
        4.785_156_25,
        5.498_046_875,
        5.092_773_437_5,
        6.943_359_375,
        6.425_781_25,
    ];
    let codepoint = character as u32;
    (0x05D0..=0x05EA)
        .contains(&codepoint)
        .then(|| ADVANCES[(codepoint - 0x05D0) as usize])
}

pub(crate) fn canvas_font_size(font: &str) -> f64 {
    font.split_ascii_whitespace()
        .find_map(|part| {
            let pixels = part.strip_suffix("px")?;
            pixels
                .parse::<f64>()
                .ok()
                .filter(|value| *value >= 0.0)
                // DirectWrite's Canvas text path truncates fractional CSS
                // font sizes to 1/100px for glyph scaling. Edge therefore
                // measures 13.3333px with the same advances as 13.33px.
                .map(|value| (value * 100.0).floor() / 100.0)
        })
        .unwrap_or(10.0)
}

fn canvas_spacing(value: &str) -> f64 {
    value
        .trim()
        .strip_suffix("px")
        .and_then(|pixels| pixels.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(0.0)
}

fn canvas_font_metrics(scope: &v8::PinScope<'_, '_>, font: &str) -> (f64, bool) {
    let lower = font.to_ascii_lowercase();
    let fonts = &crate::fingerprint::edge(scope).fonts;
    fonts
        .metrics
        .iter()
        .filter(|metric| {
            fonts
                .families
                .iter()
                .any(|family| family.eq_ignore_ascii_case(&metric.family))
        })
        .filter(|metric| lower.contains(&metric.family.to_ascii_lowercase()))
        .max_by_key(|metric| metric.family.len())
        .map(|metric| (metric.width_scale, metric.monospace))
        .unwrap_or_else(|| (1.0, lower.contains("monospace")))
}

fn measured_text_width(text: &str, state: &CanvasState, monospace: bool) -> f64 {
    let mut width = 0.0;
    let mut characters = 0_usize;
    let mut spaces = 0_usize;
    for character in text.chars() {
        characters += 1;
        if character.is_whitespace() {
            spaces += 1;
        }
        width += if monospace {
            5.0
        } else if !((' '..='~').contains(&character)) {
            generic_character_advance_10(character)
        } else if character.is_whitespace() {
            2.239_990_234_375
        } else if character == 'a' {
            5.56
        } else if character == 'b' {
            6.249_982_299_804_688
        } else if character == 'c' {
            5.0
        } else if character == 'd' {
            6.034_194_946_289_062
        } else if matches!(character, 'i' | 'l' | 'I' | '1' | '!' | '|' | '.' | ',') {
            2.089_998_779_296_875
        } else if matches!(character, 'M' | 'W' | 'm' | 'w' | '@' | '%') {
            9.44
        } else if character as u32 >= 0x2E80 {
            10.0
        } else {
            5.71
        };
    }
    width
        + characters as f64 * canvas_spacing(&state.letter_spacing)
        + spaces as f64 * canvas_spacing(&state.word_spacing)
}

fn generic_character_advance_10(character: char) -> f64 {
    if is_combining_mark(character) || character == '\u{200D}' {
        0.0
    } else if matches!(character as u32, 0x1F3FB..=0x1F3FF) {
        0.0
    } else if is_emoji_presentation(character) {
        // Edge 150 on Windows falls back from Arial/Times to Segoe UI
        // Emoji. Its common supplementary pictographs advance 21.96875 CSS
        // px at 16px.
        13.730_468_75
    } else if is_full_width_fallback(character) {
        10.0
    } else if character == '\u{0645}' {
        // Segoe UI's isolated ARABIC LETTER MEEM at the 10px reference size.
        3.378_906_25
    } else if character.is_whitespace() {
        2.239_990_234_375
    } else {
        5.71
    }
}

fn is_combining_mark(character: char) -> bool {
    matches!(character as u32,
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF |
        0x20D0..=0x20FF | 0xFE00..=0xFE0F | 0xFE20..=0xFE2F |
        0xE0100..=0xE01EF
    )
}

fn is_full_width_fallback(character: char) -> bool {
    matches!(character as u32,
        0x2E80..=0xA4CF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF |
        0xFE10..=0xFE19 | 0xFE30..=0xFE6F | 0xFF01..=0xFF60 |
        0xFFE0..=0xFFE6 | 0x20000..=0x3FFFD
    )
}

fn is_emoji_presentation(character: char) -> bool {
    matches!(character as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x26FF | 0x2700..=0x27BF
    )
}

fn trailing_glyph_inset(text: &str, monospace: bool) -> f64 {
    if monospace {
        return if text.is_empty() { 0.0 } else { 0.5 };
    }
    match text.chars().next_back() {
        Some('b') => 0.179_992_675_781_25,
        Some('i' | 'l' | 'I' | '1' | '!' | '|' | '.' | ',') => 0.25,
        Some(_) => 0.199_996_948_242_187_5,
        None => 0.0,
    }
}

fn aligned_text_bounds(
    width: f64,
    glyph_left: f64,
    glyph_right: f64,
    state: &CanvasState,
) -> (f64, f64) {
    let alignment = match state.text_align.as_str() {
        "start" if state.direction == "rtl" => "right",
        "end" if state.direction == "rtl" => "left",
        "start" => "left",
        "end" => "right",
        value => value,
    };
    match alignment {
        "center" => (width / 2.0 - glyph_left, glyph_right - width / 2.0),
        "right" => (width - glyph_left, glyph_right - width),
        _ => (glyph_left, glyph_right),
    }
}
fn get_context_attributes(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    define(
        scope,
        output,
        "alpha",
        v8::Boolean::new(scope, record.alpha).into(),
    );
    define(
        scope,
        output,
        "colorSpace",
        v8::String::new(scope, &record.color_space).unwrap().into(),
    );
    define(
        scope,
        output,
        "colorType",
        v8::String::new(scope, &record.color_type).unwrap().into(),
    );
    define(
        scope,
        output,
        "desynchronized",
        v8::Boolean::new(scope, record.desynchronized).into(),
    );
    define(
        scope,
        output,
        "willReadFrequently",
        v8::Boolean::new(scope, record.will_read_frequently).into(),
    );
    r.set(output.into())
}
fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<OffscreenCanvasRenderingContext2DStore>() {
        store.constructors.remove(&realm_id);
    }
}
