use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DomMatrixReadOnlyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, [f64; 16]>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomMatrixReadOnlyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMMatrixReadOnly", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<DomMatrixReadOnlyStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMMatrixReadOnly",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    define_component(scope, prototype, "a")?;
    define_component(scope, prototype, "b")?;
    define_component(scope, prototype, "c")?;
    define_component(scope, prototype, "d")?;
    define_component(scope, prototype, "e")?;
    define_component(scope, prototype, "f")?;
    define_component(scope, prototype, "m11")?;
    define_component(scope, prototype, "m12")?;
    define_component(scope, prototype, "m13")?;
    define_component(scope, prototype, "m14")?;
    define_component(scope, prototype, "m21")?;
    define_component(scope, prototype, "m22")?;
    define_component(scope, prototype, "m23")?;
    define_component(scope, prototype, "m24")?;
    define_component(scope, prototype, "m31")?;
    define_component(scope, prototype, "m32")?;
    define_component(scope, prototype, "m33")?;
    define_component(scope, prototype, "m34")?;
    define_component(scope, prototype, "m41")?;
    define_component(scope, prototype, "m42")?;
    define_component(scope, prototype, "m43")?;
    define_component(scope, prototype, "m44")?;
    crate::webidl::define_readonly_accessor(scope, prototype, "is2D", get_is_2d)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "isIdentity", get_is_identity)?;
    crate::webidl::define_method(scope, prototype, "flipX", 0, flip_x)?;
    crate::webidl::define_method(scope, prototype, "flipY", 0, flip_y)?;
    crate::webidl::define_method(scope, prototype, "inverse", 0, inverse)?;
    crate::webidl::define_method(scope, prototype, "multiply", 0, multiply)?;
    crate::webidl::define_method(scope, prototype, "rotate", 0, rotate)?;
    crate::webidl::define_method(scope, prototype, "rotateAxisAngle", 0, rotate_axis_angle)?;
    crate::webidl::define_method(scope, prototype, "rotateFromVector", 0, rotate_from_vector)?;
    crate::webidl::define_method(scope, prototype, "scale", 0, scale)?;
    crate::webidl::define_method(scope, prototype, "scale3d", 0, scale_3d)?;
    crate::webidl::define_method(scope, prototype, "scaleNonUniform", 0, scale_non_uniform)?;
    crate::webidl::define_method(scope, prototype, "skewX", 0, skew_x)?;
    crate::webidl::define_method(scope, prototype, "skewY", 0, skew_y)?;
    crate::webidl::define_method(scope, prototype, "toFloat32Array", 0, to_float_32_array)?;
    crate::webidl::define_method(scope, prototype, "toFloat64Array", 0, to_float_64_array)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_method(scope, prototype, "transformPoint", 0, transform_point)?;
    crate::webidl::define_method(scope, prototype, "translate", 0, translate)?;
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
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomMatrixReadOnlyStore>()
        .ok_or_else(|| "DOMMatrixReadOnly state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn identity() -> [f64; 16] {
    super::dom_matrix::identity()
}

pub(crate) fn create_from_matrix<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    matrix: [f64; 16],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DOMMatrixReadOnly".to_owned());
    }
    attach(scope, object, matrix);
    Ok(object)
}

fn return_created_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    matrix: [f64; 16],
    mut result: v8::ReturnValue<'_>,
) {
    match create_from_matrix(scope, matrix) {
        Ok(matrix) => result.set(matrix.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn matrix_from_typed_array(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Object>,
    length: usize,
    method: &str,
) -> Result<[f64; 16], String> {
    if length != 6 && length != 16 {
        let three_dimensional_phrase = if method == "fromFloat32Array" {
            "16 elements a for 3D matrix"
        } else {
            "16 elements for a 3D matrix"
        };
        return Err(format!(
            "Failed to execute '{method}' on 'DOMMatrixReadOnly': The sequence must contain 6 elements for a 2D matrix or {three_dimensional_phrase}."
        ));
    }
    let mut values = Vec::with_capacity(length);
    for index in 0..length {
        values.push(
            array
                .get_index(scope, index as u32)
                .and_then(|value| value.number_value(scope))
                .unwrap_or(f64::NAN),
        );
    }
    if length == 6 {
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
            .map_err(|_| "DOMMatrixReadOnly sequence length is invalid".to_owned())
    }
}

fn from_float_32_array(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fromFloat32Array' on 'DOMMatrixReadOnly': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(array) = v8::Local::<v8::Float32Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fromFloat32Array' on 'DOMMatrixReadOnly': parameter 1 is not of type 'Float32Array'.",
        );
        return;
    };
    match matrix_from_typed_array(scope, array.into(), array.length(), "fromFloat32Array") {
        Ok(matrix) => return_created_matrix(scope, matrix, result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn from_float_64_array(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fromFloat64Array' on 'DOMMatrixReadOnly': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(array) = v8::Local::<v8::Float64Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fromFloat64Array' on 'DOMMatrixReadOnly': parameter 1 is not of type 'Float64Array'.",
        );
        return;
    };
    match matrix_from_typed_array(scope, array.into(), array.length(), "fromFloat64Array") {
        Ok(matrix) => return_created_matrix(scope, matrix, result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn from_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let matrix = if arguments.get(0).is_undefined() {
        identity()
    } else {
        match super::dom_matrix::matrix_from_value(scope, arguments.get(0)) {
            Ok(matrix) => matrix,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    };
    return_created_matrix(scope, matrix, result);
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "DOMMatrixReadOnly must be constructed with new");
        return;
    }
    let matrix = if arguments.get(0).is_undefined() {
        identity()
    } else {
        match super::dom_matrix::matrix_from_value(scope, arguments.get(0)) {
            Ok(matrix) => matrix,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    };
    attach(scope, arguments.this(), matrix);
    result.set(arguments.this().into());
}

fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, matrix: [f64; 16]) {
    if let Some(store) = scope.get_slot_mut::<DomMatrixReadOnlyStore>() {
        store
            .records
            .insert(object.get_identity_hash().get(), matrix);
    }
}

pub(crate) fn own_matrix_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    scope
        .get_slot::<DomMatrixReadOnlyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn matrix_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    own_matrix_snapshot(scope, object).or_else(|| super::dom_matrix::matrix_snapshot(scope, object))
}

fn define_component(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let data = crate::webidl::string(scope, name)?;
    let getter = crate::webidl::create_function_with_data(
        scope,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        get_component,
        data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, getter, &format!("{owner}.get {name}"));
    }
    let mut descriptor =
        v8::PropertyDescriptor::new_from_get_set(getter.into(), v8::undefined(scope).into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, name)?;
    if prototype.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define DOMMatrixReadOnly.{name}"))
    }
}

fn component_index(name: &str) -> usize {
    match name {
        "a" | "m11" => 0,
        "b" | "m12" => 1,
        "m13" => 2,
        "m14" => 3,
        "c" | "m21" => 4,
        "d" | "m22" => 5,
        "m23" => 6,
        "m24" => 7,
        "m31" => 8,
        "m32" => 9,
        "m33" => 10,
        "m34" => 11,
        "e" | "m41" => 12,
        "f" | "m42" => 13,
        "m43" => 14,
        _ => 15,
    }
}

fn get_component(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(matrix) = matrix_snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(
        scope,
        crate::trace::native_callback_data(scope, &arguments),
    );
    result.set(v8::Number::new(scope, matrix[component_index(&name)]).into());
}

fn is_2d(matrix: &[f64; 16]) -> bool {
    matrix[2] == 0.0
        && matrix[3] == 0.0
        && matrix[6] == 0.0
        && matrix[7] == 0.0
        && matrix[8] == 0.0
        && matrix[9] == 0.0
        && matrix[10] == 1.0
        && matrix[11] == 0.0
        && matrix[14] == 0.0
        && matrix[15] == 1.0
}

fn get_is_2d(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(matrix) = matrix_snapshot(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, is_2d(&matrix)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_is_identity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(matrix) = matrix_snapshot(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, matrix == identity()).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn optional_number(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    index: i32,
    default_value: f64,
) -> f64 {
    if arguments.get(index).is_undefined() {
        default_value
    } else {
        arguments.get(index).number_value(scope).unwrap_or(f64::NAN)
    }
}

fn return_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    matrix: [f64; 16],
    mut result: v8::ReturnValue<'_>,
) {
    match super::dom_matrix::create_from_matrix(scope, matrix) {
        Ok(matrix) => result.set(matrix.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn current_or_throw(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let matrix = matrix_snapshot(scope, object);
    if matrix.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    matrix
}

fn flip_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let mut flip = identity();
    flip[0] = -1.0;
    flip[1] = -0.0;
    flip[2] = -0.0;
    flip[3] = -0.0;
    return_matrix(s, super::dom_matrix::multiply(matrix, flip), r)
}

fn flip_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let mut flip = identity();
    flip[5] = -1.0;
    flip[4] = -0.0;
    flip[6] = -0.0;
    flip[7] = -0.0;
    return_matrix(s, super::dom_matrix::multiply(matrix, flip), r)
}

fn inverse(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    return_matrix(
        s,
        super::dom_matrix::invert(matrix).unwrap_or([f64::NAN; 16]),
        r,
    )
}

fn multiply(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(left) = current_or_throw(s, a.this()) else {
        return;
    };
    let right = match super::dom_matrix::matrix_from_value(s, a.get(0)) {
        Ok(matrix) => matrix,
        Err(message) => {
            crate::webidl::throw_type_error(s, &message);
            return;
        }
    };
    return_matrix(s, super::dom_matrix::multiply(left, right), r)
}

fn rotate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let angle = if a.get(1).is_undefined() && a.get(2).is_undefined() {
        optional_number(s, &a, 0, 0.0)
    } else {
        optional_number(s, &a, 2, 0.0)
    };
    return_matrix(
        s,
        super::dom_matrix::multiply(matrix, super::dom_matrix::rotation_z(angle)),
        r,
    )
}

fn axis_rotation(x: f64, y: f64, z: f64, degrees: f64) -> [f64; 16] {
    let length = (x * x + y * y + z * z).sqrt();
    if length == 0.0 {
        return identity();
    }
    let (x, y, z) = (x / length, y / length, z / length);
    let (sin, cos) = degrees.to_radians().sin_cos();
    let sin = super::dom_matrix::snap_trigonometric_zero(sin);
    let cos = super::dom_matrix::snap_trigonometric_zero(cos);
    let t = 1.0 - cos;
    [
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
    ]
}

fn rotate_axis_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let rotation = axis_rotation(
        optional_number(s, &a, 0, 0.0),
        optional_number(s, &a, 1, 0.0),
        optional_number(s, &a, 2, 0.0),
        optional_number(s, &a, 3, 0.0),
    );
    return_matrix(s, super::dom_matrix::multiply(matrix, rotation), r)
}

fn rotate_from_vector(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let x = optional_number(s, &a, 0, 0.0);
    let y = optional_number(s, &a, 1, 0.0);
    let degrees = y.atan2(x).to_degrees();
    return_matrix(
        s,
        super::dom_matrix::multiply(matrix, super::dom_matrix::rotation_z(degrees)),
        r,
    )
}

fn translation(x: f64, y: f64, z: f64) -> [f64; 16] {
    let mut matrix = identity();
    matrix[12] = x;
    matrix[13] = y;
    matrix[14] = z;
    matrix
}

fn scaling(x: f64, y: f64, z: f64) -> [f64; 16] {
    let mut matrix = identity();
    matrix[0] = x;
    matrix[5] = y;
    matrix[10] = z;
    matrix
}

fn scale(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(mut matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let x = optional_number(s, &a, 0, 1.0);
    let y = optional_number(s, &a, 1, x);
    let z = optional_number(s, &a, 2, 1.0);
    let ox = optional_number(s, &a, 3, 0.0);
    let oy = optional_number(s, &a, 4, 0.0);
    let oz = optional_number(s, &a, 5, 0.0);
    matrix = super::dom_matrix::multiply(matrix, translation(ox, oy, oz));
    matrix = super::dom_matrix::multiply(matrix, scaling(x, y, z));
    matrix = super::dom_matrix::multiply(matrix, translation(-ox, -oy, -oz));
    return_matrix(s, matrix, r)
}

fn scale_3d(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(mut matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let factor = optional_number(s, &a, 0, 1.0);
    let ox = optional_number(s, &a, 1, 0.0);
    let oy = optional_number(s, &a, 2, 0.0);
    let oz = optional_number(s, &a, 3, 0.0);
    matrix = super::dom_matrix::multiply(matrix, translation(ox, oy, oz));
    matrix = super::dom_matrix::multiply(matrix, scaling(factor, factor, factor));
    matrix = super::dom_matrix::multiply(matrix, translation(-ox, -oy, -oz));
    return_matrix(s, matrix, r)
}

fn scale_non_uniform(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let x = optional_number(s, &a, 0, 1.0);
    let y = optional_number(s, &a, 1, 1.0);
    return_matrix(
        s,
        super::dom_matrix::multiply(matrix, scaling(x, y, 1.0)),
        r,
    )
}

fn skew_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let mut skew = identity();
    skew[4] = optional_number(s, &a, 0, 0.0).to_radians().tan();
    return_matrix(s, super::dom_matrix::multiply(matrix, skew), r)
}

fn skew_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let mut skew = identity();
    skew[1] = optional_number(s, &a, 0, 0.0).to_radians().tan();
    return_matrix(s, super::dom_matrix::multiply(matrix, skew), r)
}

fn typed_array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    matrix: [f64; 16],
    float32: bool,
) -> Option<v8::Local<'s, v8::Value>> {
    if float32 {
        let bytes = matrix
            .iter()
            .flat_map(|value| (*value as f32).to_ne_bytes())
            .collect::<Vec<_>>();
        let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
        let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
        v8::Float32Array::new(scope, buffer, 0, 16).map(Into::into)
    } else {
        let bytes = matrix
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect::<Vec<_>>();
        let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
        let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
        v8::Float64Array::new(scope, buffer, 0, 16).map(Into::into)
    }
}

fn to_float_32_array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(matrix) = current_or_throw(s, a.this())
        && let Some(array) = typed_array(s, matrix, true)
    {
        r.set(array)
    }
}
fn to_float_64_array(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(matrix) = current_or_throw(s, a.this())
        && let Some(array) = typed_array(s, matrix, false)
    {
        r.set(array)
    }
}

fn define_number(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str, v: f64) {
    if let Some(k) = v8::String::new(s, n) {
        let _ = o.create_data_property(s, k.into(), v8::Number::new(s, v).into());
    }
}

fn to_json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(m) = current_or_throw(s, a.this()) else {
        return;
    };
    let o = v8::Object::new(s);
    define_number(s, o, "a", m[0]);
    define_number(s, o, "b", m[1]);
    define_number(s, o, "c", m[4]);
    define_number(s, o, "d", m[5]);
    define_number(s, o, "e", m[12]);
    define_number(s, o, "f", m[13]);
    define_number(s, o, "m11", m[0]);
    define_number(s, o, "m12", m[1]);
    define_number(s, o, "m13", m[2]);
    define_number(s, o, "m14", m[3]);
    define_number(s, o, "m21", m[4]);
    define_number(s, o, "m22", m[5]);
    define_number(s, o, "m23", m[6]);
    define_number(s, o, "m24", m[7]);
    define_number(s, o, "m31", m[8]);
    define_number(s, o, "m32", m[9]);
    define_number(s, o, "m33", m[10]);
    define_number(s, o, "m34", m[11]);
    define_number(s, o, "m41", m[12]);
    define_number(s, o, "m42", m[13]);
    define_number(s, o, "m43", m[14]);
    define_number(s, o, "m44", m[15]);
    if let Some(k) = v8::String::new(s, "is2D") {
        let _ = o.create_data_property(s, k.into(), v8::Boolean::new(s, is_2d(&m)).into());
    }
    if let Some(k) = v8::String::new(s, "isIdentity") {
        let _ = o.create_data_property(s, k.into(), v8::Boolean::new(s, m == identity()).into());
    }
    r.set(o.into())
}

fn transform_point(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(m) = current_or_throw(s, a.this()) else {
        return;
    };
    let p = super::dom_point_read_only::from_value(s, a.get(0));
    let value = super::dom_point_read_only::PointRecord {
        x: p.x * m[0] + p.y * m[4] + p.z * m[8] + p.w * m[12],
        y: p.x * m[1] + p.y * m[5] + p.z * m[9] + p.w * m[13],
        z: p.x * m[2] + p.y * m[6] + p.z * m[10] + p.w * m[14],
        w: p.x * m[3] + p.y * m[7] + p.z * m[11] + p.w * m[15],
    };
    match super::dom_point::create(s, value) {
        Ok(point) => r.set(point.into()),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}

fn translate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(matrix) = current_or_throw(s, a.this()) else {
        return;
    };
    let value = translation(
        optional_number(s, &a, 0, 0.0),
        optional_number(s, &a, 1, 0.0),
        optional_number(s, &a, 2, 0.0),
    );
    return_matrix(s, super::dom_matrix::multiply(matrix, value), r)
}

fn to_string(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(m) = current_or_throw(s, a.this()) else {
        return;
    };
    let text = if is_2d(&m) {
        format!(
            "matrix({}, {}, {}, {}, {}, {})",
            m[0], m[1], m[4], m[5], m[12], m[13]
        )
    } else {
        let values = m
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("matrix3d({values})")
    };
    if let Some(text) = v8::String::new(s, &text) {
        r.set(text.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DomMatrixReadOnlyStore>() {
        store.constructor.remove(realm_id);
    }
}
