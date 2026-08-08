use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgPointStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PointValue>,
}

#[derive(Clone, Copy)]
pub(crate) struct PointValue {
    pub x: f64,
    pub y: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgPointStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGPoint", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgPointStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGPoint",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "x", get_x, set_x)?;
    crate::webidl::define_accessor(scope, prototype, "y", get_y, set_y)?;
    crate::webidl::define_method(scope, prototype, "matrixTransform", 1, matrix_transform)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgPointStore>()
        .ok_or_else(|| "SVGPoint state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: PointValue,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGPoint".to_owned());
    }
    scope
        .get_slot_mut::<SvgPointStore>()
        .ok_or_else(|| "SVGPoint state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), value);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'SVGPoint': Illegal constructor");
}

pub(crate) fn value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PointValue> {
    scope
        .get_slot::<SvgPointStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut PointValue),
) {
    if let Some(value) = scope
        .get_slot_mut::<SvgPointStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Number::new(scope, value.x).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |point| point.x = value);
}

fn get_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Number::new(scope, value.y).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |point| point.y = value);
}

fn matrix_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(point) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(matrix) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "matrixTransform requires an SVGMatrix");
        return;
    };
    let Some(matrix) = super::svg_matrix::value(scope, matrix) else {
        crate::webidl::throw_type_error(scope, "matrixTransform requires an SVGMatrix");
        return;
    };
    match create(
        scope,
        PointValue {
            x: matrix.a * point.x + matrix.c * point.y + matrix.e,
            y: matrix.b * point.x + matrix.d * point.y + matrix.f,
        },
    ) {
        Ok(point) => result.set(point.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
