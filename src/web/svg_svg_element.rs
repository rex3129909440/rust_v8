use std::collections::{HashMap, HashSet};

pub(crate) const ZOOM_AND_PAN_UNKNOWN: i32 = 0;
pub(crate) const ZOOM_AND_PAN_DISABLE: i32 = 1;
pub(crate) const ZOOM_AND_PAN_MAGNIFY: i32 = 2;

#[derive(Default)]
pub(crate) struct SvgSvgElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) current_scale: f64,
    pub(crate) current_translate: v8::Global<v8::Object>,
    pub(crate) view_box: v8::Global<v8::Object>,
    pub(crate) preserve_aspect_ratio: v8::Global<v8::Object>,
    pub(crate) zoom_and_pan: i32,
    pub(crate) animations_paused: bool,
    pub(crate) current_time: f64,
    pub(crate) next_redraw_handle: u32,
    pub(crate) suspended_redraws: HashSet<u32>,
    pub(crate) redraw_count: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgSvgElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGSVGElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgSvgElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGSVGElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_svg_element_x_property::define(scope, prototype)?;
    super::svg_svg_element_y_property::define(scope, prototype)?;
    super::svg_svg_element_width_property::define(scope, prototype)?;
    super::svg_svg_element_height_property::define(scope, prototype)?;
    super::svg_svg_element_current_scale_property::define(scope, prototype)?;
    super::svg_svg_element_current_translate_property::define(scope, prototype)?;
    super::svg_svg_element_view_box_property::define(scope, prototype)?;
    super::svg_svg_element_preserve_aspect_ratio_property::define(scope, prototype)?;
    super::svg_svg_element_zoom_and_pan_property::define(scope, prototype)?;
    define_constants(scope, prototype)?;
    super::svg_svg_element_animations_paused::define(scope, prototype)?;
    super::svg_svg_element_check_enclosure::define(scope, prototype)?;
    super::svg_svg_element_check_intersection::define(scope, prototype)?;
    super::svg_svg_element_create_svg_angle::define(scope, prototype)?;
    super::svg_svg_element_create_svg_length::define(scope, prototype)?;
    super::svg_svg_element_create_svg_matrix::define(scope, prototype)?;
    super::svg_svg_element_create_svg_number::define(scope, prototype)?;
    super::svg_svg_element_create_svg_point::define(scope, prototype)?;
    super::svg_svg_element_create_svg_rect::define(scope, prototype)?;
    super::svg_svg_element_create_svg_transform::define(scope, prototype)?;
    super::svg_svg_element_create_svg_transform_from_matrix::define(scope, prototype)?;
    super::svg_svg_element_deselect_all::define(scope, prototype)?;
    super::svg_svg_element_force_redraw::define(scope, prototype)?;
    super::svg_svg_element_get_current_time::define(scope, prototype)?;
    super::svg_svg_element_get_element_by_id::define(scope, prototype)?;
    super::svg_svg_element_get_enclosure_list::define(scope, prototype)?;
    super::svg_svg_element_get_intersection_list::define(scope, prototype)?;
    super::svg_svg_element_pause_animations::define(scope, prototype)?;
    super::svg_svg_element_set_current_time::define(scope, prototype)?;
    super::svg_svg_element_suspend_redraw::define(scope, prototype)?;
    super::svg_svg_element_unpause_animations::define(scope, prototype)?;
    super::svg_svg_element_unsuspend_redraw::define(scope, prototype)?;
    super::svg_svg_element_unsuspend_redraw_all::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let parent = super::svg_graphics_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgSvgElementStore>()
        .ok_or_else(|| "SVGSVGElement state was not prepared".to_owned())?
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
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object =
        super::svg_graphics_element::create_with_constructor(scope, constructor, "svg", None)?;
    let x = super::svg_animated_length::create(scope, 0.0)?;
    let y = super::svg_animated_length::create(scope, 0.0)?;
    let width = super::svg_animated_length::create(scope, 300.0)?;
    let height = super::svg_animated_length::create(scope, 150.0)?;
    let current_translate =
        super::svg_point::create(scope, super::svg_point::PointValue { x: 0.0, y: 0.0 })?;
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
        x: v8::Global::new(scope, x),
        y: v8::Global::new(scope, y),
        width: v8::Global::new(scope, width),
        height: v8::Global::new(scope, height),
        current_scale: 1.0,
        current_translate: v8::Global::new(scope, current_translate),
        view_box: v8::Global::new(scope, view_box),
        preserve_aspect_ratio: v8::Global::new(scope, preserve_aspect_ratio),
        zoom_and_pan: ZOOM_AND_PAN_MAGNIFY,
        animations_paused: false,
        current_time: 0.0,
        next_redraw_handle: 1,
        suspended_redraws: HashSet::new(),
        redraw_count: 0,
    };
    scope
        .get_slot_mut::<SvgSvgElementStore>()
        .ok_or_else(|| "SVGSVGElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGSVGElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgSvgElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(record) = scope
        .get_slot_mut::<SvgSvgElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_object(
    scope: &v8::PinScope<'_, '_>,
    value: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, value).into());
}

pub(crate) fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.width, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.height, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_current_translate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.current_translate, r)
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

pub(crate) fn get_current_scale(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.current_scale).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_current_scale(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(1.0);
    update(scope, arguments.this(), |record| {
        record.current_scale = value
    });
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
    let value = arguments.get(0).int32_value(scope).unwrap_or(0);
    if value != ZOOM_AND_PAN_DISABLE && value != ZOOM_AND_PAN_MAGNIFY {
        crate::webidl::throw_type_error(scope, "Invalid SVG zoomAndPan value");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.zoom_and_pan = value
    });
}

pub(crate) fn animations_paused(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.animations_paused).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn check_enclosure(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let valid = record(scope, arguments.this()).is_some()
        && arguments.get(0).is_object()
        && arguments.get(1).is_object();
    result.set(v8::Boolean::new(scope, valid).into());
}

pub(crate) fn check_intersection(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    check_enclosure(scope, arguments, result);
}

pub(crate) fn return_created(
    scope: &mut v8::PinScope<'_, '_>,
    value: Result<v8::Local<'_, v8::Object>, String>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Ok(value) => result.set(value.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn create_svg_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::svg_angle::create(scope);
    return_created(scope, value, result);
}

pub(crate) fn create_svg_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::svg_length::create_single(
        scope,
        super::svg_length::LengthSnapshot {
            unit: 1,
            specified: 0.0,
        },
    );
    return_created(scope, value, result);
}

pub(crate) fn create_svg_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::svg_matrix::create(scope, super::svg_matrix::MatrixValue::identity());
    return_created(scope, value, result);
}

pub(crate) fn create_svg_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::svg_number::create(scope, 0.0);
    return_created(scope, value, result);
}

pub(crate) fn create_svg_point(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::svg_point::create(scope, super::svg_point::PointValue { x: 0.0, y: 0.0 });
    return_created(scope, value, result);
}

pub(crate) fn create_svg_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::svg_rect::create_pair(
        scope,
        super::svg_rect::RectValue {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    ) {
        Ok((rect, _)) => result.set(rect.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn create_svg_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::svg_transform::create_identity(scope);
    return_created(scope, value, result);
}

pub(crate) fn create_svg_transform_from_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let matrix = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|matrix| super::svg_matrix::value(scope, matrix));
    let Some(matrix) = matrix else {
        crate::webidl::throw_type_error(scope, "An SVGMatrix is required");
        return;
    };
    let value = super::svg_transform::create(
        scope,
        super::svg_transform::TransformValue {
            kind: 1,
            matrix,
            angle: 0.0,
        },
    );
    return_created(scope, value, result);
}

pub(crate) fn deselect_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn force_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.redraw_count = record.redraw_count.saturating_add(1)
    });
}

pub(crate) fn get_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.current_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn element_has_id(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    expected: &str,
) -> bool {
    super::element::record(scope, element).is_some_and(|record| {
        record
            .attributes
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case("id") && value == expected)
    })
}

pub(crate) fn get_element_by_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let expected = crate::webidl::value_to_string(scope, arguments.get(0));
    let mut pending = super::node::children(scope, arguments.this());
    while let Some(candidate) = pending.pop() {
        if element_has_id(scope, candidate, &expected) {
            result.set(candidate.into());
            return;
        }
        pending.extend(super::node::children(scope, candidate));
    }
    result.set(v8::null(scope).into());
}

pub(crate) fn return_descendants(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    let mut pending = super::node::children(scope, root);
    let mut elements = Vec::new();
    while let Some(candidate) = pending.pop() {
        if super::element::record(scope, candidate).is_some() {
            elements.push(v8::Global::new(scope, candidate));
        }
        pending.extend(super::node::children(scope, candidate));
    }
    let array = v8::Array::new(scope, elements.len() as i32);
    for (index, element) in elements.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Local::new(scope, element).into());
    }
    result.set(array.into());
}

pub(crate) fn get_enclosure_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        return_descendants(scope, arguments.this(), result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_intersection_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_enclosure_list(scope, arguments, result);
}

pub(crate) fn pause_animations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.animations_paused = true
    });
}

pub(crate) fn set_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(0.0);
    update(scope, arguments.this(), |record| {
        record.current_time = value.max(0.0)
    });
}

pub(crate) fn suspend_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let handle = current.next_redraw_handle;
    update(scope, arguments.this(), |record| {
        record.next_redraw_handle = record.next_redraw_handle.saturating_add(1);
        record.suspended_redraws.insert(handle);
    });
    result.set(v8::Integer::new_from_unsigned(scope, handle).into());
}

pub(crate) fn unpause_animations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.animations_paused = false
    });
}

pub(crate) fn unsuspend_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handle = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| {
        record.suspended_redraws.remove(&handle);
    });
}

pub(crate) fn unsuspend_redraw_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.suspended_redraws.clear()
    });
}
