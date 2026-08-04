use std::collections::HashMap;

pub(crate) const MARKER_UNITS_UNKNOWN: i32 = 0;
pub(crate) const MARKER_UNITS_USER_SPACE: i32 = 1;
pub(crate) const MARKER_UNITS_STROKE_WIDTH: i32 = 2;
pub(crate) const ORIENT_UNKNOWN: i32 = 0;
pub(crate) const ORIENT_AUTO: i32 = 1;
pub(crate) const ORIENT_ANGLE: i32 = 2;

#[derive(Default)]
pub(crate) struct SvgMarkerElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) ref_x: v8::Global<v8::Object>,
    pub(crate) ref_y: v8::Global<v8::Object>,
    pub(crate) marker_units: v8::Global<v8::Object>,
    pub(crate) marker_width: v8::Global<v8::Object>,
    pub(crate) marker_height: v8::Global<v8::Object>,
    pub(crate) orient_type: v8::Global<v8::Object>,
    pub(crate) orient_angle: v8::Global<v8::Object>,
    pub(crate) view_box: v8::Global<v8::Object>,
    pub(crate) preserve_aspect_ratio: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgMarkerElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGMarkerElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgMarkerElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGMarkerElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_marker_element_ref_x_property::define(scope, prototype)?;
    super::svg_marker_element_ref_y_property::define(scope, prototype)?;
    super::svg_marker_element_marker_units_property::define(scope, prototype)?;
    super::svg_marker_element_marker_width_property::define(scope, prototype)?;
    super::svg_marker_element_marker_height_property::define(scope, prototype)?;
    super::svg_marker_element_orient_type_property::define(scope, prototype)?;
    super::svg_marker_element_orient_angle_property::define(scope, prototype)?;
    super::svg_marker_element_view_box_property::define(scope, prototype)?;
    super::svg_marker_element_preserve_aspect_ratio_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    super::svg_marker_element_set_orient_to_angle::define(scope, prototype)?;
    super::svg_marker_element_set_orient_to_auto::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgMarkerElementStore>()
        .ok_or_else(|| "SVGMarkerElement state was not prepared".to_owned())?
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
        "SVG_MARKERUNITS_UNKNOWN",
        MARKER_UNITS_UNKNOWN,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_MARKERUNITS_USERSPACEONUSE",
        MARKER_UNITS_USER_SPACE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_MARKERUNITS_STROKEWIDTH",
        MARKER_UNITS_STROKE_WIDTH,
    )?;
    crate::webidl::define_constant(scope, object, "SVG_MARKER_ORIENT_UNKNOWN", ORIENT_UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "SVG_MARKER_ORIENT_AUTO", ORIENT_AUTO)?;
    crate::webidl::define_constant(scope, object, "SVG_MARKER_ORIENT_ANGLE", ORIENT_ANGLE)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(scope, constructor, "marker", owner)?;
    let ref_x = super::svg_animated_length::create(scope, 0.0)?;
    let ref_y = super::svg_animated_length::create(scope, 0.0)?;
    let marker_units =
        super::svg_animated_enumeration::create(scope, MARKER_UNITS_STROKE_WIDTH as u32)?;
    let marker_width = super::svg_animated_length::create(scope, 3.0)?;
    let marker_height = super::svg_animated_length::create(scope, 3.0)?;
    let orient_type = super::svg_animated_enumeration::create(scope, ORIENT_ANGLE as u32)?;
    let orient_angle = super::svg_animated_angle::create(
        scope,
        super::svg_angle::AngleSnapshot {
            unit: 1,
            specified: 0.0,
        },
    )?;
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
    let record = Record {
        ref_x: v8::Global::new(scope, ref_x),
        ref_y: v8::Global::new(scope, ref_y),
        marker_units: v8::Global::new(scope, marker_units),
        marker_width: v8::Global::new(scope, marker_width),
        marker_height: v8::Global::new(scope, marker_height),
        orient_type: v8::Global::new(scope, orient_type),
        orient_angle: v8::Global::new(scope, orient_angle),
        view_box: v8::Global::new(scope, view_box),
        preserve_aspect_ratio: v8::Global::new(scope, preserve_aspect_ratio),
    };
    scope
        .get_slot_mut::<SvgMarkerElementStore>()
        .ok_or_else(|| "SVGMarkerElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGMarkerElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgMarkerElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_object(
    scope: &v8::PinScope<'_, '_>,
    value: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, value).into());
}

pub(crate) fn get_ref_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.ref_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_ref_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.ref_y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_marker_units(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.marker_units, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_marker_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.marker_width, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_marker_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.marker_height, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_orient_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.orient_type, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_orient_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.orient_angle, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_view_box(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.view_box, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_preserve_aspect_ratio(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.preserve_aspect_ratio, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn set_orient_to_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(angle) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "setOrientToAngle requires an SVGAngle");
        return;
    };
    let Some(angle) = super::svg_angle::snapshot(scope, angle) else {
        crate::webidl::throw_type_error(scope, "setOrientToAngle requires an SVGAngle");
        return;
    };
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let orient_type = v8::Local::new(scope, &record.orient_type);
    let orient_angle = v8::Local::new(scope, &record.orient_angle);
    super::svg_animated_enumeration::set(scope, orient_type, ORIENT_ANGLE as u32);
    if let Err(error) = super::svg_animated_angle::set(scope, orient_angle, angle) {
        crate::webidl::throw_type_error(scope, &error);
    }
}

pub(crate) fn set_orient_to_auto(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let orient_type = v8::Local::new(scope, &record.orient_type);
    super::svg_animated_enumeration::set(scope, orient_type, ORIENT_AUTO as u32);
}
