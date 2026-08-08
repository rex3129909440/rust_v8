use std::collections::HashMap;

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
    set_string(s, a, |v| &mut v.font, |v| !v.trim().is_empty())
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
    set_string(s, a, |v| &mut v.font_stretch, any)
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
    set_string(s, a, |v| &mut v.font_variant_caps, any)
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
    let lower = color.to_ascii_lowercase();
    let rgb = if lower.starts_with('#') && lower.len() == 7 {
        [
            u8::from_str_radix(&lower[1..3], 16).unwrap_or(0),
            u8::from_str_radix(&lower[3..5], 16).unwrap_or(0),
            u8::from_str_radix(&lower[5..7], 16).unwrap_or(0),
        ]
    } else {
        match lower.as_str() {
            "red" => [255, 0, 0],
            "blue" => [0, 0, 255],
            "white" => [255, 255, 255],
            "green" => [0, 128, 0],
            _ => [0, 0, 0],
        }
    };
    [rgb[0], rgb[1], rgb[2], (alpha * 255.0).round() as u8]
}
fn paint_rect(record: &mut ContextRecord, x: f64, y: f64, w: f64, h: f64, color: [u8; 4]) {
    let x0 = x.floor().max(0.0) as u32;
    let y0 = y.floor().max(0.0) as u32;
    let x1 = (x + w).ceil().max(0.0).min(record.width as f64) as u32;
    let y1 = (y + h).ceil().max(0.0).min(record.height as f64) as u32;
    for py in y0.min(record.height)..y1 {
        for px in x0.min(record.width)..x1 {
            let i = (py as usize * record.width as usize + px as usize) * 4;
            if i + 3 < record.pixels.len() {
                record.pixels[i..i + 4].copy_from_slice(&color)
            }
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
        paint_rect(record, v[0], v[1], v[2], v[3], [0, 0, 0, 0])
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
        let l = record.state.line_width;
        paint_rect(record, v[0], v[1], v[2], l, c);
        paint_rect(record, v[0], v[1] + v[3] - l, v[2], l, c);
        paint_rect(record, v[0], v[1], l, v[3], c);
        paint_rect(record, v[0] + v[2] - l, v[1], l, v[3], c)
    })
}
fn fill(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn stroke(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
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
    update(scope, a.this(), |record| {
        let color = parse_color(&record.state.fill_style, record.state.global_alpha);
        paint_rect(
            record,
            x,
            y - 10.0,
            text.chars().count() as f64 * 6.0,
            10.0,
            color,
        )
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
    update(scope, a.this(), |record| {
        let color = parse_color(&record.state.stroke_style, record.state.global_alpha);
        paint_rect(
            record,
            x,
            y - 10.0,
            text.chars().count() as f64 * 6.0,
            10.0,
            color,
        )
    })
}

fn get_image_data(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
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
    let (configured_width_scale, monospace) = canvas_font_metrics(scope, &record.state.font);
    let width = measured_text_width(&text, &record.state, monospace)
        * canvas.text_width_scale
        * configured_width_scale
        * font_scale;
    let has_ink = !text.is_empty();
    let glyph_left = if monospace && text.starts_with('W') {
        0.5 * font_scale
    } else {
        canvas.actual_bounding_box_left * font_scale
    };
    let glyph_right = width * canvas.actual_bounding_box_right_scale
        - trailing_glyph_inset(&text, monospace) * font_scale;
    let (alignment_left, alignment_right) =
        aligned_text_bounds(width, glyph_left, glyph_right, &record.state);
    let font_size = canvas_font_size(&record.state.font);
    let font_ascent = if monospace {
        font_size * 0.85
    } else {
        canvas.font_bounding_box_ascent * font_scale
    };
    let font_descent = if monospace {
        font_size * 0.15
    } else {
        canvas.font_bounding_box_descent * font_scale
    };
    let actual_ascent = if monospace {
        font_size * 0.65
    } else {
        canvas.actual_bounding_box_ascent * font_scale
    };
    let actual_descent = if monospace {
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
        actual_bounding_box_ascent: if has_ink { actual_ascent } else { 0.0 },
        actual_bounding_box_descent: if has_ink { actual_descent } else { 0.0 },
        hanging_baseline: if monospace {
            font_size * 0.68
        } else {
            canvas.hanging_baseline * font_scale
        },
        alphabetic_baseline: canvas.alphabetic_baseline * font_scale,
        ideographic_baseline: if monospace {
            -font_size * 0.15
        } else {
            canvas.ideographic_baseline * font_scale
        },
    };
    if let Ok(value) = super::text_metrics::create(scope, metrics) {
        r.set(value.into())
    }
}

fn canvas_font_size(font: &str) -> f64 {
    font.split_ascii_whitespace()
        .find_map(|part| {
            let pixels = part.strip_suffix("px")?;
            pixels.parse::<f64>().ok().filter(|value| *value > 0.0)
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
