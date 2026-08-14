use std::collections::HashMap;

#[derive(Clone)]
struct CssRotateRecord {
    angle: v8::Global<v8::Object>,
    x: f64,
    y: f64,
    z: f64,
    original_2d: bool,
}

#[derive(Default)]
pub(crate) struct CssRotateStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssRotateRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssRotateStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSRotate", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssRotateStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSRotate",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "angle", get_angle, set_angle)?;
    crate::webidl::define_accessor(scope, prototype, "x", get_x, set_x)?;
    crate::webidl::define_accessor(scope, prototype, "y", get_y, set_y)?;
    crate::webidl::define_accessor(scope, prototype, "z", get_z, set_z)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_transform_component::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssRotateStore>()
        .ok_or_else(|| "CSSRotate state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn angle(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Object>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let record = super::css_unit_value::record(scope, object)?;
    if !matches!(record.unit.as_str(), "deg" | "rad" | "grad" | "turn") {
        crate::webidl::throw_type_error(scope, "CSSRotate requires an angle");
        return None;
    }
    Some(v8::Global::new(scope, object))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CSSRotate requires an angle");
        return;
    }
    if arguments.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSRotate': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let valid_angle = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .is_some_and(|object| super::css_numeric_value::is_numeric(scope, object));
    if !valid_angle {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSRotate': parameter 1 is not of type 'CSSNumericValue'.",
        );
        return;
    }
    let (x, y, z, angle_value) = if arguments.length() >= 4 {
        (
            arguments.get(0).number_value(scope).unwrap_or(f64::NAN),
            arguments.get(1).number_value(scope).unwrap_or(f64::NAN),
            arguments.get(2).number_value(scope).unwrap_or(f64::NAN),
            arguments.get(3),
        )
    } else {
        (0.0, 0.0, 1.0, arguments.get(0))
    };
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        crate::webidl::throw_type_error(scope, "CSSRotate axis must be finite");
        return;
    }
    let Some(angle) = angle(scope, angle_value) else {
        return;
    };
    let original_2d = x == 0.0 && y == 0.0 && z == 1.0;
    scope
        .get_slot_mut::<CssRotateStore>()
        .expect("CSSRotate state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CssRotateRecord {
                angle,
                x,
                y,
                z,
                original_2d,
            },
        );
    super::css_transform_component::attach(scope, arguments.this(), original_2d);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssRotateRecord> {
    scope
        .get_slot::<CssRotateStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.angle).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(angle) = angle(scope, arguments.get(0)) else {
        return;
    };
    if let Some(record) = scope.get_slot_mut::<CssRotateStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.angle = angle;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_axis(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    axis: u8,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, object) {
        let value = match axis {
            0 => record.x,
            1 => record.y,
            _ => record.z,
        };
        result.set(v8::Number::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_axis(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    axis: u8,
) {
    let value = value.number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "CSSRotate axis must be finite");
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<CssRotateStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        match axis {
            0 => record.x = value,
            1 => record.y = value,
            _ => record.z = value,
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_axis(s, a.this(), 0, r);
}
fn set_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_axis(s, a.this(), a.get(0), 0);
}
fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_axis(s, a.this(), 1, r);
}
fn set_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_axis(s, a.this(), a.get(0), 1);
}
fn get_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_axis(s, a.this(), 2, r);
}
fn set_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_axis(s, a.this(), a.get(0), 2);
}

fn degrees(record: &super::css_unit_value::CssUnitRecord) -> f64 {
    match record.unit.as_str() {
        "rad" => record.value.to_degrees(),
        "grad" => record.value * 0.9,
        "turn" => record.value * 360.0,
        _ => record.value,
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let angle = super::css_unit_value::record(scope, v8::Local::new(scope, &record.angle))?;
    Some(if record.original_2d {
        format!("rotate({})", super::css_unit_value::serialize(&angle))
    } else {
        format!(
            "rotate3d({}, {}, {}, {})",
            record.x,
            record.y,
            record.z,
            super::css_unit_value::serialize(&angle)
        )
    })
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let record = record(scope, object)?;
    let angle = super::css_unit_value::record(scope, v8::Local::new(scope, &record.angle))?;
    if record.original_2d {
        Some(super::dom_matrix::rotation_z(degrees(&angle)))
    } else {
        let mut matrix = super::dom_matrix::identity();
        let length = (record.x * record.x + record.y * record.y + record.z * record.z).sqrt();
        if length == 0.0 {
            return Some(matrix);
        }
        let x = record.x / length;
        let y = record.y / length;
        let z = record.z / length;
        let radians = degrees(&angle).to_radians();
        let cosine = radians.cos();
        let sine = radians.sin();
        let one_minus = 1.0 - cosine;
        matrix[0] = cosine + x * x * one_minus;
        matrix[1] = y * x * one_minus + z * sine;
        matrix[2] = z * x * one_minus - y * sine;
        matrix[4] = x * y * one_minus - z * sine;
        matrix[5] = cosine + y * y * one_minus;
        matrix[6] = z * y * one_minus + x * sine;
        matrix[8] = x * z * one_minus + y * sine;
        matrix[9] = y * z * one_minus - x * sine;
        matrix[10] = cosine + z * z * one_minus;
        Some(matrix)
    }
}
