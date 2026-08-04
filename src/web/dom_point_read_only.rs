use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(crate) struct PointRecord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Default)]
pub(crate) struct DomPointReadOnlyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PointRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomPointReadOnlyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMPointReadOnly", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<DomPointReadOnlyStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMPointReadOnly",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "x", get_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "y", get_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "z", get_z)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "w", get_w)?;
    crate::webidl::define_method(scope, prototype, "matrixTransform", 0, matrix_transform)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomPointReadOnlyStore>()
        .ok_or_else(|| "DOMPointReadOnly state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "DOMPointReadOnly must be constructed with new");
        return;
    }
    let value = |index: i32, default_value: f64| {
        if arguments.get(index).is_undefined() {
            default_value
        } else {
            arguments.get(index).number_value(scope).unwrap_or(f64::NAN)
        }
    };
    attach(
        scope,
        arguments.this(),
        PointRecord {
            x: value(0, 0.0),
            y: value(1, 0.0),
            z: value(2, 0.0),
            w: value(3, 1.0),
        },
    );
    result.set(arguments.this().into());
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    point: PointRecord,
) {
    if let Some(store) = scope.get_slot_mut::<DomPointReadOnlyStore>() {
        store
            .records
            .insert(object.get_identity_hash().get(), point);
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PointRecord> {
    scope
        .get_slot::<DomPointReadOnlyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut PointRecord),
) -> bool {
    let Some(point) = scope
        .get_slot_mut::<DomPointReadOnlyStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    change(point);
    true
}

pub(crate) fn from_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> PointRecord {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return PointRecord {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        };
    };
    let property = |name: &str, default_value: f64| {
        v8::String::new(scope, name)
            .and_then(|key| object.get(scope, key.into()))
            .filter(|value| !value.is_undefined())
            .and_then(|value| value.number_value(scope))
            .unwrap_or(default_value)
    };
    PointRecord {
        x: property("x", 0.0),
        y: property("y", 0.0),
        z: property("z", 0.0),
        w: property("w", 1.0),
    }
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(PointRecord) -> f64,
) {
    if let Some(point) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(point)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.x)
}
fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.y)
}
fn get_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.z)
}
fn get_w(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.w)
}

fn matrix_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(point) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matrix = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|matrix| super::dom_matrix_read_only::matrix_snapshot(scope, matrix))
        .unwrap_or_else(super::dom_matrix_read_only::identity);
    let transformed = PointRecord {
        x: point.x * matrix[0] + point.y * matrix[4] + point.z * matrix[8] + point.w * matrix[12],
        y: point.x * matrix[1] + point.y * matrix[5] + point.z * matrix[9] + point.w * matrix[13],
        z: point.x * matrix[2] + point.y * matrix[6] + point.z * matrix[10] + point.w * matrix[14],
        w: point.x * matrix[3] + point.y * matrix[7] + point.z * matrix[11] + point.w * matrix[15],
    };
    match super::dom_point::create(scope, transformed) {
        Ok(point) => result.set(point.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(point) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    define_number(scope, object, "x", point.x);
    define_number(scope, object, "y", point.y);
    define_number(scope, object, "z", point.z);
    define_number(scope, object, "w", point.w);
    result.set(object.into());
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DomPointReadOnlyStore>() {
        store.constructor.remove(realm_id);
    }
}
