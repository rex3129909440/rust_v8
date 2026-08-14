use std::collections::HashMap;

#[derive(Clone)]
struct CssScaleRecord {
    x: v8::Global<v8::Object>,
    y: v8::Global<v8::Object>,
    z: v8::Global<v8::Object>,
    original_2d: bool,
}

#[derive(Default)]
pub(crate) struct CssScaleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssScaleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssScaleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSScale", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssScaleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSScale",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "x", get_x, set_x)?;
    crate::webidl::define_accessor(scope, prototype, "y", get_y, set_y)?;
    crate::webidl::define_accessor(scope, prototype, "z", get_z, set_z)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_transform_component::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssScaleStore>()
        .ok_or_else(|| "CSSScale state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn scale_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Object>> {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value)
        && super::css_unit_value::record(scope, object).is_some()
    {
        return Some(v8::Global::new(scope, object));
    }
    if let Some(message) = crate::webidl::number_conversion_error(value) {
        crate::webidl::throw_type_error(scope, &message);
        return None;
    }
    let Some(number) = value.number_value(scope) else {
        return None;
    };
    if !number.is_finite() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSScale': The provided double value is non-finite.",
        );
        return None;
    }
    super::css_unit_value::create(scope, number, "number")
        .ok()
        .map(|value| v8::Global::new(scope, value))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "CSSScale requires x and y");
        return;
    }
    let Some(x) = scale_value(scope, arguments.get(0)) else {
        return;
    };
    let Some(y) = scale_value(scope, arguments.get(1)) else {
        return;
    };
    let z = if arguments.get(2).is_undefined() {
        match super::css_unit_value::create(scope, 1.0, "number") {
            Ok(value) => v8::Global::new(scope, value),
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        }
    } else {
        let Some(z) = scale_value(scope, arguments.get(2)) else {
            return;
        };
        z
    };
    let is_2d = super::css_unit_value::record(scope, v8::Local::new(scope, &z))
        .is_some_and(|value| value.value == 1.0);
    scope
        .get_slot_mut::<CssScaleStore>()
        .expect("CSSScale state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CssScaleRecord {
                x,
                y,
                z,
                original_2d: is_2d,
            },
        );
    super::css_transform_component::attach(scope, arguments.this(), is_2d);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssScaleRecord> {
    scope
        .get_slot::<CssScaleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_component(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    component: u8,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, object) {
        let value = match component {
            0 => record.x,
            1 => record.y,
            _ => record.z,
        };
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_component(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    component: u8,
) {
    let Some(value) = scale_value(scope, value) else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<CssScaleStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        match component {
            0 => record.x = value,
            1 => record.y = value,
            _ => record.z = value,
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
}

fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 0, r);
}
fn set_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), a.get(0), 0);
}
fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 1, r);
}
fn set_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), a.get(0), 1);
}
fn get_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_component(s, a.this(), 2, r);
}
fn set_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_component(s, a.this(), a.get(0), 2);
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let x = super::css_unit_value::record(scope, v8::Local::new(scope, &record.x))?;
    let y = super::css_unit_value::record(scope, v8::Local::new(scope, &record.y))?;
    let z = super::css_unit_value::record(scope, v8::Local::new(scope, &record.z))?;
    Some(if record.original_2d {
        format!(
            "scale({}, {})",
            super::css_unit_value::serialize(&x),
            super::css_unit_value::serialize(&y)
        )
    } else {
        format!(
            "scale3d({}, {}, {})",
            super::css_unit_value::serialize(&x),
            super::css_unit_value::serialize(&y),
            super::css_unit_value::serialize(&z)
        )
    })
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let record = record(scope, object)?;
    let x = super::css_unit_value::record(scope, v8::Local::new(scope, &record.x))?;
    let y = super::css_unit_value::record(scope, v8::Local::new(scope, &record.y))?;
    let z = super::css_unit_value::record(scope, v8::Local::new(scope, &record.z))?;
    let mut matrix = super::dom_matrix::identity();
    matrix[0] = x.value;
    matrix[5] = y.value;
    matrix[10] = z.value;
    Some(matrix)
}
