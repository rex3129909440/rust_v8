use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgMatrixStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MatrixValue>,
}

#[derive(Clone, Copy)]
pub(crate) struct MatrixValue {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl MatrixValue {
    pub(crate) fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgMatrixStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGMatrix", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgMatrixStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGMatrix",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "a", get_a, set_a)?;
    crate::webidl::define_accessor(scope, prototype, "b", get_b, set_b)?;
    crate::webidl::define_accessor(scope, prototype, "c", get_c, set_c)?;
    crate::webidl::define_accessor(scope, prototype, "d", get_d, set_d)?;
    crate::webidl::define_accessor(scope, prototype, "e", get_e, set_e)?;
    crate::webidl::define_accessor(scope, prototype, "f", get_f, set_f)?;
    crate::webidl::define_method(scope, prototype, "flipX", 0, flip_x)?;
    crate::webidl::define_method(scope, prototype, "flipY", 0, flip_y)?;
    crate::webidl::define_method(scope, prototype, "inverse", 0, inverse)?;
    crate::webidl::define_method(scope, prototype, "multiply", 1, multiply)?;
    crate::webidl::define_method(scope, prototype, "rotate", 1, rotate)?;
    crate::webidl::define_method(scope, prototype, "rotateFromVector", 2, rotate_from_vector)?;
    crate::webidl::define_method(scope, prototype, "scale", 1, scale)?;
    crate::webidl::define_method(scope, prototype, "scaleNonUniform", 2, scale_non_uniform)?;
    crate::webidl::define_method(scope, prototype, "skewX", 1, skew_x)?;
    crate::webidl::define_method(scope, prototype, "skewY", 1, skew_y)?;
    crate::webidl::define_method(scope, prototype, "translate", 2, translate)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgMatrixStore>()
        .ok_or_else(|| "SVGMatrix state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: MatrixValue,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGMatrix".to_owned());
    }
    scope
        .get_slot_mut::<SvgMatrixStore>()
        .ok_or_else(|| "SVGMatrix state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), value);
    Ok(object)
}

pub(crate) fn value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MatrixValue> {
    scope
        .get_slot::<SvgMatrixStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: MatrixValue,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<SvgMatrixStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    *record = value;
    true
}

pub(crate) fn product(left: MatrixValue, right: MatrixValue) -> MatrixValue {
    MatrixValue {
        a: left.a * right.a + left.c * right.b,
        b: left.b * right.a + left.d * right.b,
        c: left.a * right.c + left.c * right.d,
        d: left.b * right.c + left.d * right.d,
        e: left.a * right.e + left.c * right.f + left.e,
        f: left.b * right.e + left.d * right.f + left.f,
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGMatrix': Illegal constructor",
    );
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut MatrixValue),
) {
    let Some(record) = scope
        .get_slot_mut::<SvgMatrixStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    change(record);
}

fn number_argument(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    index: i32,
) -> f64 {
    arguments.get(index).number_value(scope).unwrap_or(f64::NAN)
}

fn return_component(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    component: impl FnOnce(MatrixValue) -> f64,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, object) {
        result.set(v8::Number::new(scope, component(value)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_a(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_component(s, a.this(), |m| m.a, r);
}
fn get_b(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_component(s, a.this(), |m| m.b, r);
}
fn get_c(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_component(s, a.this(), |m| m.c, r);
}
fn get_d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_component(s, a.this(), |m| m.d, r);
}
fn get_e(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_component(s, a.this(), |m| m.e, r);
}
fn get_f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_component(s, a.this(), |m| m.f, r);
}
fn set_a(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = number_argument(s, &a, 0);
    update(s, a.this(), |m| m.a = n);
}
fn set_b(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = number_argument(s, &a, 0);
    update(s, a.this(), |m| m.b = n);
}
fn set_c(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = number_argument(s, &a, 0);
    update(s, a.this(), |m| m.c = n);
}
fn set_d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = number_argument(s, &a, 0);
    update(s, a.this(), |m| m.d = n);
}
fn set_e(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = number_argument(s, &a, 0);
    update(s, a.this(), |m| m.e = n);
}
fn set_f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let n = number_argument(s, &a, 0);
    update(s, a.this(), |m| m.f = n);
}

fn return_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    value: MatrixValue,
    mut result: v8::ReturnValue<'_>,
) {
    match create(scope, value) {
        Ok(matrix) => result.set(matrix.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

fn flip_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        return_matrix(
            scope,
            product(
                value,
                MatrixValue {
                    a: -1.0,
                    ..MatrixValue::identity()
                },
            ),
            result,
        );
    }
    crate::webidl::throw_type_error(scope, "Illegal invocation");
}

fn flip_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        return_matrix(
            scope,
            product(
                value,
                MatrixValue {
                    d: -1.0,
                    ..MatrixValue::identity()
                },
            ),
            result,
        );
    }
    crate::webidl::throw_type_error(scope, "Illegal invocation");
}

fn inverse(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let determinant = value.a * value.d - value.b * value.c;
    if determinant == 0.0 {
        crate::webidl::throw_type_error(scope, "Matrix is not invertible");
        return;
    }
    return_matrix(
        scope,
        MatrixValue {
            a: value.d / determinant,
            b: -value.b / determinant,
            c: -value.c / determinant,
            d: value.a / determinant,
            e: (value.c * value.f - value.d * value.e) / determinant,
            f: (value.b * value.e - value.a * value.f) / determinant,
        },
        result,
    );
}

fn multiply(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(left) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(right_object) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "multiply requires an SVGMatrix");
        return;
    };
    let Some(right) = value(scope, right_object) else {
        crate::webidl::throw_type_error(scope, "multiply requires an SVGMatrix");
        return;
    };
    return_matrix(scope, product(left, right), result);
}

fn rotate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let radians = number_argument(scope, &arguments, 0).to_radians();
    return_matrix(
        scope,
        product(
            value,
            MatrixValue {
                a: radians.cos(),
                b: radians.sin(),
                c: -radians.sin(),
                d: radians.cos(),
                e: 0.0,
                f: 0.0,
            },
        ),
        result,
    );
}

fn rotate_from_vector(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let x = number_argument(scope, &arguments, 0);
    let y = number_argument(scope, &arguments, 1);
    if x == 0.0 || y == 0.0 {
        crate::webidl::throw_type_error(scope, "Vector components cannot be zero");
        return;
    }
    let radians = y.atan2(x);
    return_matrix(
        scope,
        product(
            value,
            MatrixValue {
                a: radians.cos(),
                b: radians.sin(),
                c: -radians.sin(),
                d: radians.cos(),
                e: 0.0,
                f: 0.0,
            },
        ),
        result,
    );
}

fn scale(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let amount = number_argument(scope, &arguments, 0);
    return_matrix(
        scope,
        product(
            value,
            MatrixValue {
                a: amount,
                d: amount,
                ..MatrixValue::identity()
            },
        ),
        result,
    );
}

fn scale_non_uniform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let x = number_argument(scope, &arguments, 0);
    let y = number_argument(scope, &arguments, 1);
    return_matrix(
        scope,
        product(
            value,
            MatrixValue {
                a: x,
                d: y,
                ..MatrixValue::identity()
            },
        ),
        result,
    );
}

fn skew_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let tangent = number_argument(scope, &arguments, 0).to_radians().tan();
    return_matrix(
        scope,
        product(
            value,
            MatrixValue {
                c: tangent,
                ..MatrixValue::identity()
            },
        ),
        result,
    );
}

fn skew_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let tangent = number_argument(scope, &arguments, 0).to_radians().tan();
    return_matrix(
        scope,
        product(
            value,
            MatrixValue {
                b: tangent,
                ..MatrixValue::identity()
            },
        ),
        result,
    );
}

fn translate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let x = number_argument(scope, &arguments, 0);
    let y = number_argument(scope, &arguments, 1);
    return_matrix(
        scope,
        product(
            value,
            MatrixValue {
                e: x,
                f: y,
                ..MatrixValue::identity()
            },
        ),
        result,
    );
}
