use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DomMatrixStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, [f64; 16]>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomMatrixStore::default());
}

#[allow(dead_code)]
pub(crate) fn install_standard_name(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMMatrix", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<DomMatrixStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }

    let constructor = crate::webidl::create_function(
        scope,
        "DOMMatrix",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "a", get_a, set_a)?;
    crate::webidl::define_accessor(scope, prototype, "b", get_b, set_b)?;
    crate::webidl::define_accessor(scope, prototype, "c", get_c, set_c)?;
    crate::webidl::define_accessor(scope, prototype, "d", get_d, set_d)?;
    crate::webidl::define_accessor(scope, prototype, "e", get_e, set_e)?;
    crate::webidl::define_accessor(scope, prototype, "f", get_f, set_f)?;
    crate::webidl::define_accessor(scope, prototype, "m11", get_m11, set_m11)?;
    crate::webidl::define_accessor(scope, prototype, "m12", get_m12, set_m12)?;
    crate::webidl::define_accessor(scope, prototype, "m13", get_m13, set_m13)?;
    crate::webidl::define_accessor(scope, prototype, "m14", get_m14, set_m14)?;
    crate::webidl::define_accessor(scope, prototype, "m21", get_m21, set_m21)?;
    crate::webidl::define_accessor(scope, prototype, "m22", get_m22, set_m22)?;
    crate::webidl::define_accessor(scope, prototype, "m23", get_m23, set_m23)?;
    crate::webidl::define_accessor(scope, prototype, "m24", get_m24, set_m24)?;
    crate::webidl::define_accessor(scope, prototype, "m31", get_m31, set_m31)?;
    crate::webidl::define_accessor(scope, prototype, "m32", get_m32, set_m32)?;
    crate::webidl::define_accessor(scope, prototype, "m33", get_m33, set_m33)?;
    crate::webidl::define_accessor(scope, prototype, "m34", get_m34, set_m34)?;
    crate::webidl::define_accessor(scope, prototype, "m41", get_m41, set_m41)?;
    crate::webidl::define_accessor(scope, prototype, "m42", get_m42, set_m42)?;
    crate::webidl::define_accessor(scope, prototype, "m43", get_m43, set_m43)?;
    crate::webidl::define_accessor(scope, prototype, "m44", get_m44, set_m44)?;
    crate::webidl::define_method(scope, prototype, "invertSelf", 0, invert_self)?;
    crate::webidl::define_method(scope, prototype, "multiplySelf", 0, multiply_self)?;
    crate::webidl::define_method(scope, prototype, "preMultiplySelf", 0, pre_multiply_self)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "rotateAxisAngleSelf",
        0,
        rotate_axis_angle_self,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "rotateFromVectorSelf",
        0,
        rotate_from_vector_self,
    )?;
    crate::webidl::define_method(scope, prototype, "rotateSelf", 0, rotate_self)?;
    crate::webidl::define_method(scope, prototype, "scale3dSelf", 0, scale_3d_self)?;
    crate::webidl::define_method(scope, prototype, "scaleSelf", 0, scale_self)?;
    crate::webidl::define_method(scope, prototype, "skewXSelf", 0, skew_x_self)?;
    crate::webidl::define_method(scope, prototype, "skewYSelf", 0, skew_y_self)?;
    crate::webidl::define_method(scope, prototype, "translateSelf", 0, translate_self)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "fromFloat32Array",
        1,
        from_float_32_array,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "fromFloat64Array",
        1,
        from_float_64_array,
    )?;
    crate::webidl::define_method(scope, constructor.into(), "fromMatrix", 0, from_matrix)?;

    crate::webidl::define_method(scope, prototype, "setMatrixValue", 1, set_matrix_value)?;
    let parent = super::dom_matrix_read_only::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);

    scope
        .get_slot_mut::<DomMatrixStore>()
        .ok_or_else(|| "DOMMatrix state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn identity() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub(crate) fn create_2d<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: [f64; 6],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DOMMatrix".to_owned());
    }
    let mut matrix = identity();
    matrix[0] = values[0];
    matrix[1] = values[1];
    matrix[4] = values[2];
    matrix[5] = values[3];
    matrix[12] = values[4];
    matrix[13] = values[5];
    scope
        .get_slot_mut::<DomMatrixStore>()
        .ok_or_else(|| "DOMMatrix state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), matrix);
    Ok(object)
}

pub(crate) fn create_from_matrix<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    matrix: [f64; 16],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DOMMatrix".to_owned());
    }
    scope
        .get_slot_mut::<DomMatrixStore>()
        .ok_or_else(|| "DOMMatrix state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), matrix);
    Ok(object)
}

pub(crate) fn matrix_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    scope
        .get_slot::<DomMatrixStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'DOMMatrix': Please use the 'new' operator",
        );
        return;
    }
    let matrix = if arguments.get(0).is_undefined() {
        identity()
    } else {
        match matrix_from_value(scope, arguments.get(0)) {
            Ok(matrix) => matrix,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    };
    let object = arguments.this();
    scope
        .get_slot_mut::<DomMatrixStore>()
        .expect("DOMMatrix state")
        .records
        .insert(object.get_identity_hash().get(), matrix);
    result.set(object.into());
}

pub(crate) fn matrix_from_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<[f64; 16], String> {
    if value.is_string() || value.is_string_object() {
        return parse_css_matrix(&crate::webidl::value_to_string(scope, value));
    }
    let object = v8::Local::<v8::Object>::try_from(value)
        .map_err(|_| "DOMMatrix initializer must be a string or sequence".to_owned())?;
    if let Some(matrix) = scope
        .get_slot::<DomMatrixStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .copied()
    {
        return Ok(matrix);
    }
    if let Some(matrix) = super::dom_matrix_read_only::own_matrix_snapshot(scope, object) {
        return Ok(matrix);
    }
    let length_key = v8::String::new(scope, "length").unwrap();
    let length = object
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    if length != 6 && length != 16 {
        return matrix_from_object(scope, object);
    }
    let mut values = Vec::with_capacity(length as usize);
    for index in 0..length {
        let value = object
            .get_index(scope, index)
            .and_then(|value| value.number_value(scope))
            .ok_or_else(|| "DOMMatrix sequence contains a non-number".to_owned())?;
        values.push(value);
    }
    if values.len() == 6 {
        let mut matrix = identity();
        matrix[0] = values[0];
        matrix[1] = values[1];
        matrix[4] = values[2];
        matrix[5] = values[3];
        matrix[12] = values[4];
        matrix[13] = values[5];
        Ok(matrix)
    } else {
        values
            .try_into()
            .map_err(|_| "DOMMatrix sequence length is invalid".to_owned())
    }
}

fn matrix_from_object(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<[f64; 16], String> {
    let mut matrix = identity();
    matrix[0] = object_number(scope, object, "m11", 1.0);
    matrix[1] = object_number(scope, object, "m12", 0.0);
    matrix[2] = object_number(scope, object, "m13", 0.0);
    matrix[3] = object_number(scope, object, "m14", 0.0);
    matrix[4] = object_number(scope, object, "m21", 0.0);
    matrix[5] = object_number(scope, object, "m22", 1.0);
    matrix[6] = object_number(scope, object, "m23", 0.0);
    matrix[7] = object_number(scope, object, "m24", 0.0);
    matrix[8] = object_number(scope, object, "m31", 0.0);
    matrix[9] = object_number(scope, object, "m32", 0.0);
    matrix[10] = object_number(scope, object, "m33", 1.0);
    matrix[11] = object_number(scope, object, "m34", 0.0);
    matrix[12] = object_number(scope, object, "m41", 0.0);
    matrix[13] = object_number(scope, object, "m42", 0.0);
    matrix[14] = object_number(scope, object, "m43", 0.0);
    matrix[15] = object_number(scope, object, "m44", 1.0);
    Ok(matrix)
}

fn object_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    default: f64,
) -> f64 {
    v8::String::new(scope, name)
        .and_then(|key| object.get(scope, key.into()))
        .and_then(|value| value.number_value(scope))
        .unwrap_or(default)
}

fn parse_css_matrix(value: &str) -> Result<[f64; 16], String> {
    let value = value.trim();
    if value == "none" || value.is_empty() {
        return Ok(identity());
    }
    if let Some(values) = value
        .strip_prefix("matrix(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let numbers: Result<Vec<f64>, _> = values
            .split(',')
            .map(|value| value.trim().parse())
            .collect();
        let numbers = numbers.map_err(|_| "Invalid CSS matrix".to_owned())?;
        if numbers.len() != 6 {
            return Err("CSS matrix() requires six values".to_owned());
        }
        let mut matrix = identity();
        matrix[0] = numbers[0];
        matrix[1] = numbers[1];
        matrix[4] = numbers[2];
        matrix[5] = numbers[3];
        matrix[12] = numbers[4];
        matrix[13] = numbers[5];
        return Ok(matrix);
    }
    Err("Invalid CSS matrix".to_owned())
}

fn get_component(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    index: usize,
    result: &mut v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<DomMatrixStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .map(|matrix| matrix[index])
    {
        result.set(v8::Number::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_component(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    index: usize,
    value: v8::Local<'_, v8::Value>,
) {
    let value = value.number_value(scope).unwrap_or(f64::NAN);
    if let Some(matrix) = scope
        .get_slot_mut::<DomMatrixStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        matrix[index] = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_a(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 0, &mut r);
}
fn set_a(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 0, a.get(0));
}
fn get_b(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 1, &mut r);
}
fn set_b(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 1, a.get(0));
}
fn get_c(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 4, &mut r);
}
fn set_c(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 4, a.get(0));
}
fn get_d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 5, &mut r);
}
fn set_d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 5, a.get(0));
}
fn get_e(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 12, &mut r);
}
fn set_e(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 12, a.get(0));
}
fn get_f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 13, &mut r);
}
fn set_f(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 13, a.get(0));
}
fn get_m11(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 0, &mut r);
}
fn set_m11(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 0, a.get(0));
}
fn get_m12(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 1, &mut r);
}
fn set_m12(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 1, a.get(0));
}
fn get_m13(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 2, &mut r);
}
fn set_m13(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 2, a.get(0));
}
fn get_m14(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 3, &mut r);
}
fn set_m14(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 3, a.get(0));
}
fn get_m21(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 4, &mut r);
}
fn set_m21(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 4, a.get(0));
}
fn get_m22(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 5, &mut r);
}
fn set_m22(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 5, a.get(0));
}
fn get_m23(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 6, &mut r);
}
fn set_m23(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 6, a.get(0));
}
fn get_m24(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 7, &mut r);
}
fn set_m24(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 7, a.get(0));
}
fn get_m31(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 8, &mut r);
}
fn set_m31(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 8, a.get(0));
}
fn get_m32(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 9, &mut r);
}
fn set_m32(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 9, a.get(0));
}
fn get_m33(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 10, &mut r);
}
fn set_m33(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 10, a.get(0));
}
fn get_m34(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 11, &mut r);
}
fn set_m34(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 11, a.get(0));
}
fn get_m41(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 12, &mut r);
}
fn set_m41(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 12, a.get(0));
}
fn get_m42(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 13, &mut r);
}
fn set_m42(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 13, a.get(0));
}
fn get_m43(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 14, &mut r);
}
fn set_m43(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 14, a.get(0));
}
fn get_m44(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 15, &mut r);
}
fn set_m44(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), 15, a.get(0));
}

fn return_this(mut result: v8::ReturnValue<'_>, object: v8::Local<'_, v8::Object>) {
    result.set(object.into());
}

fn optional_number(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    index: i32,
    default: f64,
) -> f64 {
    let value = arguments.get(index);
    if value.is_undefined() {
        default
    } else {
        value.number_value(scope).unwrap_or(f64::NAN)
    }
}

fn with_matrix_mut(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut [f64; 16]),
) -> bool {
    if let Some(matrix) = scope
        .get_slot_mut::<DomMatrixStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        operation(matrix);
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn multiply(left: [f64; 16], right: [f64; 16]) -> [f64; 16] {
    let mut output = [0.0; 16];
    for row in 0..4 {
        for column in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

pub(crate) fn invert(matrix: [f64; 16]) -> Option<[f64; 16]> {
    let mut augmented = [[0.0; 8]; 4];
    for row in 0..4 {
        for column in 0..4 {
            augmented[row][column] = matrix[column * 4 + row];
        }
        augmented[row][row + 4] = 1.0;
    }
    for column in 0..4 {
        let pivot = (column..4).max_by(|left, right| {
            augmented[*left][column]
                .abs()
                .total_cmp(&augmented[*right][column].abs())
        })?;
        if augmented[pivot][column].abs() < f64::EPSILON {
            return None;
        }
        augmented.swap(column, pivot);
        let divisor = augmented[column][column];
        for value in &mut augmented[column] {
            *value /= divisor;
        }
        for row in 0..4 {
            if row == column {
                continue;
            }
            let factor = augmented[row][column];
            for index in 0..8 {
                augmented[row][index] -= factor * augmented[column][index];
            }
        }
    }
    let mut output = [0.0; 16];
    for row in 0..4 {
        for column in 0..4 {
            output[column * 4 + row] = augmented[row][column + 4];
        }
    }
    Some(output)
}

fn invert_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let object = a.this();
    with_matrix_mut(s, object, |matrix| {
        *matrix = invert(*matrix).unwrap_or([f64::NAN; 16])
    });
    return_this(r, object);
}

fn multiply_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let right = match matrix_from_value(s, a.get(0)) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(s, &message);
            return;
        }
    };
    let object = a.this();
    with_matrix_mut(s, object, |matrix| *matrix = multiply(*matrix, right));
    return_this(r, object);
}

fn pre_multiply_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let left = match matrix_from_value(s, a.get(0)) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(s, &message);
            return;
        }
    };
    let object = a.this();
    with_matrix_mut(s, object, |matrix| *matrix = multiply(left, *matrix));
    return_this(r, object);
}

pub(crate) fn rotation_z(degrees: f64) -> [f64; 16] {
    let angle = degrees.to_radians();
    let (sin, cos) = angle.sin_cos();
    let mut matrix = identity();
    matrix[0] = cos;
    matrix[1] = sin;
    matrix[4] = -sin;
    matrix[5] = cos;
    matrix
}

fn rotate_axis_angle_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let x = optional_number(s, &a, 0, 0.0);
    let y = optional_number(s, &a, 1, 0.0);
    let z = optional_number(s, &a, 2, 0.0);
    let angle = optional_number(s, &a, 3, 0.0);
    let length = (x * x + y * y + z * z).sqrt();
    let object = a.this();
    if length > 0.0 {
        let (x, y, z) = (x / length, y / length, z / length);
        let radians = angle.to_radians();
        let (sin, cos) = radians.sin_cos();
        let t = 1.0 - cos;
        let rotation = [
            t * x * x + cos,
            t * x * y + sin * z,
            t * x * z - sin * y,
            0.0,
            t * x * y - sin * z,
            t * y * y + cos,
            t * y * z + sin * x,
            0.0,
            t * x * z + sin * y,
            t * y * z - sin * x,
            t * z * z + cos,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ];
        with_matrix_mut(s, object, |matrix| *matrix = multiply(*matrix, rotation));
    }
    return_this(r, object);
}

fn rotate_from_vector_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let x = optional_number(s, &a, 0, 0.0);
    let y = optional_number(s, &a, 1, 0.0);
    let rotation = rotation_z(y.atan2(x).to_degrees());
    let object = a.this();
    with_matrix_mut(s, object, |matrix| *matrix = multiply(*matrix, rotation));
    return_this(r, object);
}

fn rotate_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let angle = optional_number(s, &a, 0, 0.0);
    let object = a.this();
    with_matrix_mut(s, object, |matrix| {
        *matrix = multiply(*matrix, rotation_z(angle))
    });
    return_this(r, object);
}

fn scale_3d_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let scale = optional_number(s, &a, 0, 1.0);
    let origin_x = optional_number(s, &a, 1, 0.0);
    let origin_y = optional_number(s, &a, 2, 0.0);
    let origin_z = optional_number(s, &a, 3, 0.0);
    apply_scale(
        s,
        a.this(),
        scale,
        scale,
        scale,
        origin_x,
        origin_y,
        origin_z,
    );
    return_this(r, a.this());
}

fn scale_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let x = optional_number(s, &a, 0, 1.0);
    let y = optional_number(s, &a, 1, x);
    let z = optional_number(s, &a, 2, 1.0);
    let origin_x = optional_number(s, &a, 3, 0.0);
    let origin_y = optional_number(s, &a, 4, 0.0);
    let origin_z = optional_number(s, &a, 5, 0.0);
    apply_scale(s, a.this(), x, y, z, origin_x, origin_y, origin_z);
    return_this(r, a.this());
}

fn apply_scale(
    s: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    x: f64,
    y: f64,
    z: f64,
    ox: f64,
    oy: f64,
    oz: f64,
) {
    let mut scale = identity();
    scale[0] = x;
    scale[5] = y;
    scale[10] = z;
    scale[12] = ox - ox * x;
    scale[13] = oy - oy * y;
    scale[14] = oz - oz * z;
    with_matrix_mut(s, object, |matrix| *matrix = multiply(*matrix, scale));
}

fn skew_x_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let mut transform = identity();
    transform[4] = optional_number(s, &a, 0, 0.0).to_radians().tan();
    let object = a.this();
    with_matrix_mut(s, object, |matrix| *matrix = multiply(*matrix, transform));
    return_this(r, object);
}

fn skew_y_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let mut transform = identity();
    transform[1] = optional_number(s, &a, 0, 0.0).to_radians().tan();
    let object = a.this();
    with_matrix_mut(s, object, |matrix| *matrix = multiply(*matrix, transform));
    return_this(r, object);
}

fn translate_self(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let mut transform = identity();
    transform[12] = optional_number(s, &a, 0, 0.0);
    transform[13] = optional_number(s, &a, 1, 0.0);
    transform[14] = optional_number(s, &a, 2, 0.0);
    let object = a.this();
    with_matrix_mut(s, object, |matrix| *matrix = multiply(*matrix, transform));
    return_this(r, object);
}

fn set_matrix_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    let matrix = match parse_css_matrix(&value) {
        Ok(matrix) => matrix,
        Err(message) => {
            crate::webidl::throw_type_error(s, &message);
            return;
        }
    };
    let object = a.this();
    with_matrix_mut(s, object, |target| *target = matrix);
    return_this(r, object);
}

fn return_created_matrix(
    s: &mut v8::PinScope<'_, '_>,
    matrix: [f64; 16],
    mut r: v8::ReturnValue<'_>,
) {
    let constructor = s
        .get_slot::<DomMatrixStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(s)))
        .cloned();
    let Some(constructor) = constructor else {
        return;
    };
    let constructor = v8::Local::new(s, &constructor);
    let Ok(prototype) = crate::webidl::prototype(s, constructor) else {
        return;
    };
    let object = v8::Object::new(s);
    let _ = crate::webidl::set_platform_prototype(s, object, prototype.into());
    s.get_slot_mut::<DomMatrixStore>()
        .expect("DOMMatrix state")
        .records
        .insert(object.get_identity_hash().get(), matrix);
    r.set(object.into());
}

fn from_float_32_array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match matrix_from_value(s, a.get(0)) {
        Ok(matrix) => return_created_matrix(s, matrix, r),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
fn from_float_64_array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    match matrix_from_value(s, a.get(0)) {
        Ok(matrix) => return_created_matrix(s, matrix, r),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
fn from_matrix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let matrix = if a.get(0).is_undefined() {
        identity()
    } else {
        match matrix_from_value(s, a.get(0)) {
            Ok(matrix) => matrix,
            Err(message) => {
                crate::webidl::throw_type_error(s, &message);
                return;
            }
        }
    };
    return_created_matrix(s, matrix, r);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DomMatrixStore>() {
        store.constructor.remove(realm_id);
    }
}
