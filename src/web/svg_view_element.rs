use std::collections::HashMap;

pub(crate) const ZOOM_AND_PAN_UNKNOWN: i32 = 0;
pub(crate) const ZOOM_AND_PAN_DISABLE: i32 = 1;
pub(crate) const ZOOM_AND_PAN_MAGNIFY: i32 = 2;

#[derive(Default)]
pub(crate) struct SvgViewElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) view_box: v8::Global<v8::Object>,
    pub(crate) preserve_aspect_ratio: v8::Global<v8::Object>,
    pub(crate) zoom_and_pan: i32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgViewElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGViewElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgViewElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGViewElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_view_element_view_box_property::define(scope, prototype)?;
    super::svg_view_element_preserve_aspect_ratio_property::define(scope, prototype)?;
    super::svg_view_element_zoom_and_pan_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgViewElementStore>()
        .ok_or_else(|| "SVGViewElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_ZOOMANDPAN_UNKNOWN",
        ZOOM_AND_PAN_UNKNOWN,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_ZOOMANDPAN_DISABLE",
        ZOOM_AND_PAN_DISABLE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_ZOOMANDPAN_MAGNIFY",
        ZOOM_AND_PAN_MAGNIFY,
    )
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(scope, constructor, "view", owner)?;
    let view_box = super::svg_animated_rect::create(
        scope,
        super::svg_rect::RectValue {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    )?;
    let preserve_aspect_ratio = super::svg_animated_preserve_aspect_ratio::create(
        scope,
        super::svg_preserve_aspect_ratio::PreserveAspectRatioValue {
            align: 6,
            meet_or_slice: 1,
        },
    )?;
    let view_box = v8::Global::new(scope, view_box);
    let preserve_aspect_ratio = v8::Global::new(scope, preserve_aspect_ratio);
    scope
        .get_slot_mut::<SvgViewElementStore>()
        .ok_or_else(|| "SVGViewElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            Record {
                view_box,
                preserve_aspect_ratio,
                zoom_and_pan: ZOOM_AND_PAN_MAGNIFY,
            },
        );
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGViewElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgViewElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_view_box(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.view_box).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_preserve_aspect_ratio(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.preserve_aspect_ratio).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_zoom_and_pan(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.zoom_and_pan).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_zoom_and_pan(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .int32_value(scope)
        .unwrap_or(ZOOM_AND_PAN_UNKNOWN);
    if value != ZOOM_AND_PAN_DISABLE && value != ZOOM_AND_PAN_MAGNIFY {
        crate::webidl::throw_type_error(scope, "Invalid SVG zoomAndPan value");
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<SvgViewElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.zoom_and_pan = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
