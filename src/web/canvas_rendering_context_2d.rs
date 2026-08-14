#[derive(Default)]
pub(crate) struct CanvasRenderingContext2DStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CanvasRenderingContext2DStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CanvasRenderingContext2D", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CanvasRenderingContext2DStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CanvasRenderingContext2D",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    define_surface(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CanvasRenderingContext2DStore>()
        .ok_or_else(|| "CanvasRenderingContext2D state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn copy_property(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    source: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let descriptor = source
        .get_own_property_descriptor(scope, key.into())
        .ok_or_else(|| format!("missing canvas descriptor {name}"))?;
    let object_key = crate::webidl::string(scope, "Object")?;
    let object = scope
        .get_current_context()
        .global(scope)
        .get(scope, object_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "Object constructor is unavailable".to_owned())?;
    let define_key = crate::webidl::string(scope, "defineProperty")?;
    let define = object
        .get(scope, define_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "Object.defineProperty is unavailable".to_owned())?;
    if define
        .call(
            scope,
            object.into(),
            &[target.into(), key.into(), descriptor],
        )
        .is_some()
    {
        Ok(())
    } else {
        Err(format!("cannot copy canvas descriptor {name}"))
    }
}

fn define_surface(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let source_constructor =
        super::offscreen_canvas_rendering_context_2d::ensure_constructor(scope)?;
    let source = crate::webidl::prototype(scope, source_constructor)?;
    copy_property(scope, prototype, source, "canvas")?;
    copy_property(scope, prototype, source, "lang")?;
    copy_property(scope, prototype, source, "font")?;
    copy_property(scope, prototype, source, "textAlign")?;
    copy_property(scope, prototype, source, "textBaseline")?;
    copy_property(scope, prototype, source, "direction")?;
    copy_property(scope, prototype, source, "fontKerning")?;
    copy_property(scope, prototype, source, "fontStretch")?;
    copy_property(scope, prototype, source, "fontVariantCaps")?;
    copy_property(scope, prototype, source, "letterSpacing")?;
    copy_property(scope, prototype, source, "textRendering")?;
    copy_property(scope, prototype, source, "wordSpacing")?;
    copy_property(scope, prototype, source, "globalCompositeOperation")?;
    copy_property(scope, prototype, source, "filter")?;
    copy_property(scope, prototype, source, "imageSmoothingQuality")?;
    copy_property(scope, prototype, source, "strokeStyle")?;
    copy_property(scope, prototype, source, "fillStyle")?;
    copy_property(scope, prototype, source, "shadowColor")?;
    copy_property(scope, prototype, source, "lineCap")?;
    copy_property(scope, prototype, source, "lineJoin")?;
    copy_property(scope, prototype, source, "globalAlpha")?;
    copy_property(scope, prototype, source, "imageSmoothingEnabled")?;
    copy_property(scope, prototype, source, "shadowOffsetX")?;
    copy_property(scope, prototype, source, "shadowOffsetY")?;
    copy_property(scope, prototype, source, "shadowBlur")?;
    copy_property(scope, prototype, source, "lineWidth")?;
    copy_property(scope, prototype, source, "miterLimit")?;
    copy_property(scope, prototype, source, "lineDashOffset")?;
    copy_property(scope, prototype, source, "clip")?;
    copy_property(scope, prototype, source, "createConicGradient")?;
    copy_property(scope, prototype, source, "createImageData")?;
    copy_property(scope, prototype, source, "createLinearGradient")?;
    copy_property(scope, prototype, source, "createPattern")?;
    copy_property(scope, prototype, source, "createRadialGradient")?;
    crate::webidl::define_method(
        scope,
        prototype,
        "drawFocusIfNeeded",
        1,
        draw_focus_if_needed,
    )?;
    copy_property(scope, prototype, source, "drawImage")?;
    copy_property(scope, prototype, source, "fill")?;
    copy_property(scope, prototype, source, "fillText")?;
    copy_property(scope, prototype, source, "getContextAttributes")?;
    copy_property(scope, prototype, source, "getImageData")?;
    copy_property(scope, prototype, source, "getLineDash")?;
    copy_property(scope, prototype, source, "getTransform")?;
    copy_property(scope, prototype, source, "isContextLost")?;
    copy_property(scope, prototype, source, "isPointInPath")?;
    copy_property(scope, prototype, source, "isPointInStroke")?;
    copy_property(scope, prototype, source, "measureText")?;
    copy_property(scope, prototype, source, "reset")?;
    copy_property(scope, prototype, source, "roundRect")?;
    copy_property(scope, prototype, source, "setLineDash")?;
    copy_property(scope, prototype, source, "strokeText")?;
    copy_property(scope, prototype, source, "arc")?;
    copy_property(scope, prototype, source, "arcTo")?;
    copy_property(scope, prototype, source, "beginPath")?;
    copy_property(scope, prototype, source, "bezierCurveTo")?;
    copy_property(scope, prototype, source, "clearRect")?;
    copy_property(scope, prototype, source, "closePath")?;
    copy_property(scope, prototype, source, "ellipse")?;
    copy_property(scope, prototype, source, "fillRect")?;
    copy_property(scope, prototype, source, "lineTo")?;
    copy_property(scope, prototype, source, "moveTo")?;
    copy_property(scope, prototype, source, "putImageData")?;
    copy_property(scope, prototype, source, "quadraticCurveTo")?;
    copy_property(scope, prototype, source, "rect")?;
    copy_property(scope, prototype, source, "resetTransform")?;
    copy_property(scope, prototype, source, "restore")?;
    copy_property(scope, prototype, source, "rotate")?;
    copy_property(scope, prototype, source, "save")?;
    copy_property(scope, prototype, source, "scale")?;
    copy_property(scope, prototype, source, "setTransform")?;
    copy_property(scope, prototype, source, "stroke")?;
    copy_property(scope, prototype, source, "strokeRect")?;
    copy_property(scope, prototype, source, "transform")?;
    copy_property(scope, prototype, source, "translate")
}

fn draw_focus_if_needed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::offscreen_canvas_rendering_context_2d::pixel_snapshot(scope, arguments.this())
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 || !arguments.get(0).is_object() {
        crate::webidl::throw_type_error(scope, "drawFocusIfNeeded requires an Element");
        return;
    }
    result.set(v8::undefined(scope).into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    canvas: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let context = super::offscreen_canvas_rendering_context_2d::create(scope, canvas, options)?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    if crate::webidl::set_platform_prototype(scope, context, prototype.into()) != Some(true) {
        return Err("cannot create CanvasRenderingContext2D".to_owned());
    }
    Ok(context)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CanvasRenderingContext2D': Illegal constructor",
    );
}
