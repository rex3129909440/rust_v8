use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgGeometryElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) path_length: v8::Global<v8::Object>,
    pub(crate) total_length: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgGeometryElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGGeometryElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgGeometryElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGGeometryElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_geometry_element_path_length_property::define(scope, prototype)?;
    super::svg_geometry_element_get_point_at_length::define(scope, prototype)?;
    super::svg_geometry_element_get_total_length::define(scope, prototype)?;
    super::svg_geometry_element_is_point_in_fill::define(scope, prototype)?;
    super::svg_geometry_element_is_point_in_stroke::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_graphics_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgGeometryElementStore>()
        .ok_or_else(|| "SVGGeometryElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_with_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    constructor: v8::Local<'s, v8::Function>,
    tag_name: &str,
    owner: Option<v8::Local<'s, v8::Object>>,
    total_length: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object =
        super::svg_graphics_element::create_with_constructor(scope, constructor, tag_name, owner)?;
    let path_length = super::svg_animated_number::create(scope, 0.0)?;
    let path_length = v8::Global::new(scope, path_length);
    scope
        .get_slot_mut::<SvgGeometryElementStore>()
        .ok_or_else(|| "SVGGeometryElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            Record {
                path_length,
                total_length,
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
        "Failed to construct 'SVGGeometryElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgGeometryElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_path_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.path_length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_total_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.total_length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_point_at_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let distance = arguments
        .get(0)
        .number_value(scope)
        .unwrap_or(0.0)
        .clamp(0.0, record.total_length);
    match super::svg_point::create(
        scope,
        super::svg_point::PointValue {
            x: distance,
            y: 0.0,
        },
    ) {
        Ok(point) => result.set(point.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn point_coordinates(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<(f64, f64)> {
    let point = v8::Local::<v8::Object>::try_from(value).ok()?;
    let x_key = v8::String::new(scope, "x")?;
    let y_key = v8::String::new(scope, "y")?;
    let x = point.get(scope, x_key.into())?.number_value(scope)?;
    let y = point.get(scope, y_key.into())?.number_value(scope)?;
    Some((x, y))
}

pub(crate) fn is_point_in_fill(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let inside = point_coordinates(scope, arguments.get(0))
        .is_some_and(|(x, y)| x >= 0.0 && x <= record.total_length && y.abs() <= 0.5);
    result.set(v8::Boolean::new(scope, inside).into());
}

pub(crate) fn is_point_in_stroke(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let inside = point_coordinates(scope, arguments.get(0))
        .is_some_and(|(x, y)| x >= 0.0 && x <= record.total_length && y.abs() <= 1.0);
    result.set(v8::Boolean::new(scope, inside).into());
}
