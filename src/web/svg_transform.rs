use std::collections::HashMap;

const UNKNOWN: i32 = 0;
const MATRIX: i32 = 1;
const TRANSLATE: i32 = 2;
const SCALE: i32 = 3;
const ROTATE: i32 = 4;
const SKEWX: i32 = 5;
const SKEWY: i32 = 6;

#[derive(Default)]
pub(crate) struct SvgTransformStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TransformRecord>,
}

#[derive(Clone)]
struct TransformRecord {
    kind: i32,
    matrix: v8::Global<v8::Object>,
    angle: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct TransformValue {
    pub kind: i32,
    pub matrix: super::svg_matrix::MatrixValue,
    pub angle: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgTransformStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGTransform", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgTransformStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGTransform",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "matrix", get_matrix)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "angle", get_angle)?;
    define_constants(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "setMatrix", 1, set_matrix)?;
    crate::webidl::define_method(scope, prototype, "setRotate", 3, set_rotate)?;
    crate::webidl::define_method(scope, prototype, "setScale", 2, set_scale)?;
    crate::webidl::define_method(scope, prototype, "setSkewX", 1, set_skew_x)?;
    crate::webidl::define_method(scope, prototype, "setSkewY", 1, set_skew_y)?;
    crate::webidl::define_method(scope, prototype, "setTranslate", 2, set_translate)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgTransformStore>()
        .ok_or_else(|| "SVGTransform state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "SVG_TRANSFORM_UNKNOWN", UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "SVG_TRANSFORM_MATRIX", MATRIX)?;
    crate::webidl::define_constant(scope, object, "SVG_TRANSFORM_TRANSLATE", TRANSLATE)?;
    crate::webidl::define_constant(scope, object, "SVG_TRANSFORM_SCALE", SCALE)?;
    crate::webidl::define_constant(scope, object, "SVG_TRANSFORM_ROTATE", ROTATE)?;
    crate::webidl::define_constant(scope, object, "SVG_TRANSFORM_SKEWX", SKEWX)?;
    crate::webidl::define_constant(scope, object, "SVG_TRANSFORM_SKEWY", SKEWY)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: TransformValue,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGTransform".to_owned());
    }
    let matrix = super::svg_matrix::create(scope, value.matrix)?;
    let matrix = v8::Global::new(scope, matrix);
    scope
        .get_slot_mut::<SvgTransformStore>()
        .ok_or_else(|| "SVGTransform state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TransformRecord {
                kind: value.kind,
                matrix,
                angle: value.angle,
            },
        );
    Ok(object)
}

pub(crate) fn create_identity<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create(
        scope,
        TransformValue {
            kind: MATRIX,
            matrix: super::svg_matrix::MatrixValue::identity(),
            angle: 0.0,
        },
    )
}

pub(crate) fn value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TransformValue> {
    let record = scope
        .get_slot::<SvgTransformStore>()?
        .records
        .get(&object.get_identity_hash().get())?
        .clone();
    let matrix = super::svg_matrix::value(scope, v8::Local::new(scope, &record.matrix))?;
    Some(TransformValue {
        kind: record.kind,
        matrix,
        angle: record.angle,
    })
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGTransform': Illegal constructor",
    );
}

fn set_record(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: TransformValue,
) {
    let matrix = scope
        .get_slot::<SvgTransformStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .map(|record| record.matrix.clone());
    let Some(matrix) = matrix else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matrix_object = v8::Local::new(scope, &matrix);
    super::svg_matrix::set_value(scope, matrix_object, value.matrix);
    if let Some(record) = scope
        .get_slot_mut::<SvgTransformStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.kind = value.kind;
        record.angle = value.angle;
    }
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, value.kind).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let matrix = scope
        .get_slot::<SvgTransformStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.matrix.clone());
    if let Some(matrix) = matrix {
        result.set(v8::Local::new(scope, &matrix).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Number::new(scope, value.angle).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn number(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    index: i32,
) -> f64 {
    arguments.get(index).number_value(scope).unwrap_or(f64::NAN)
}

fn set_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if value(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(matrix) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "setMatrix requires an SVGMatrix");
        return;
    };
    let Some(matrix) = super::svg_matrix::value(scope, matrix) else {
        crate::webidl::throw_type_error(scope, "setMatrix requires an SVGMatrix");
        return;
    };
    set_record(
        scope,
        arguments.this(),
        TransformValue {
            kind: MATRIX,
            matrix,
            angle: 0.0,
        },
    );
}

fn set_rotate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let angle = number(scope, &arguments, 0);
    let cx = number(scope, &arguments, 1);
    let cy = number(scope, &arguments, 2);
    let radians = angle.to_radians();
    let rotation = super::svg_matrix::MatrixValue {
        a: radians.cos(),
        b: radians.sin(),
        c: -radians.sin(),
        d: radians.cos(),
        e: 0.0,
        f: 0.0,
    };
    let to_center = super::svg_matrix::MatrixValue {
        e: cx,
        f: cy,
        ..super::svg_matrix::MatrixValue::identity()
    };
    let from_center = super::svg_matrix::MatrixValue {
        e: -cx,
        f: -cy,
        ..super::svg_matrix::MatrixValue::identity()
    };
    let matrix =
        super::svg_matrix::product(super::svg_matrix::product(to_center, rotation), from_center);
    set_record(
        scope,
        arguments.this(),
        TransformValue {
            kind: ROTATE,
            matrix,
            angle,
        },
    );
}

fn set_scale(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let x = number(scope, &arguments, 0);
    let y = number(scope, &arguments, 1);
    set_record(
        scope,
        arguments.this(),
        TransformValue {
            kind: SCALE,
            matrix: super::svg_matrix::MatrixValue {
                a: x,
                d: y,
                ..super::svg_matrix::MatrixValue::identity()
            },
            angle: 0.0,
        },
    );
}

fn set_skew_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let angle = number(scope, &arguments, 0);
    set_record(
        scope,
        arguments.this(),
        TransformValue {
            kind: SKEWX,
            matrix: super::svg_matrix::MatrixValue {
                c: angle.to_radians().tan(),
                ..super::svg_matrix::MatrixValue::identity()
            },
            angle,
        },
    );
}

fn set_skew_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let angle = number(scope, &arguments, 0);
    set_record(
        scope,
        arguments.this(),
        TransformValue {
            kind: SKEWY,
            matrix: super::svg_matrix::MatrixValue {
                b: angle.to_radians().tan(),
                ..super::svg_matrix::MatrixValue::identity()
            },
            angle,
        },
    );
}

fn set_translate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let x = number(scope, &arguments, 0);
    let y = number(scope, &arguments, 1);
    set_record(
        scope,
        arguments.this(),
        TransformValue {
            kind: TRANSLATE,
            matrix: super::svg_matrix::MatrixValue {
                e: x,
                f: y,
                ..super::svg_matrix::MatrixValue::identity()
            },
            angle: 0.0,
        },
    );
}
