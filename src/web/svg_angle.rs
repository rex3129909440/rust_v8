use std::collections::HashMap;

const UNKNOWN: i32 = 0;
const UNSPECIFIED: i32 = 1;
const DEG: i32 = 2;
const RAD: i32 = 3;
const GRAD: i32 = 4;

#[derive(Default)]
pub(crate) struct SvgAngleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AngleValue>,
}

#[derive(Clone, Copy)]
struct AngleValue {
    unit: i32,
    specified: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct AngleSnapshot {
    pub unit: i32,
    pub specified: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgAngleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGAngle", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgAngleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGAngle",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "unitType", get_unit)?;
    crate::webidl::define_accessor(scope, prototype, "value", get_value, set_value)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "valueInSpecifiedUnits",
        get_specified,
        set_specified,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "valueAsString",
        get_as_string,
        set_as_string,
    )?;
    define_constants(scope, prototype)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "convertToSpecifiedUnits",
        1,
        convert_to_units,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "newValueSpecifiedUnits",
        2,
        new_value_units,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgAngleStore>()
        .ok_or_else(|| "SVGAngle state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "SVG_ANGLETYPE_UNKNOWN", UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "SVG_ANGLETYPE_UNSPECIFIED", UNSPECIFIED)?;
    crate::webidl::define_constant(scope, object, "SVG_ANGLETYPE_DEG", DEG)?;
    crate::webidl::define_constant(scope, object, "SVG_ANGLETYPE_RAD", RAD)?;
    crate::webidl::define_constant(scope, object, "SVG_ANGLETYPE_GRAD", GRAD)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_from(
        scope,
        AngleSnapshot {
            unit: UNSPECIFIED,
            specified: 0.0,
        },
    )
}

pub(crate) fn create_from<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: AngleSnapshot,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGAngle".to_owned());
    }
    scope
        .get_slot_mut::<SvgAngleStore>()
        .ok_or_else(|| "SVGAngle state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AngleValue {
                unit: value.unit,
                specified: value.specified,
            },
        );
    Ok(object)
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AngleSnapshot> {
    value(scope, object).map(|value| AngleSnapshot {
        unit: value.unit,
        specified: value.specified,
    })
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'SVGAngle': Illegal constructor");
}

fn value(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<AngleValue> {
    scope
        .get_slot::<SvgAngleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut AngleValue),
) {
    if let Some(value) = scope
        .get_slot_mut::<SvgAngleStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn degrees(value: AngleValue) -> f64 {
    match value.unit {
        RAD => value.specified.to_degrees(),
        GRAD => value.specified * 0.9,
        _ => value.specified,
    }
}

fn from_degrees(value: f64, unit: i32) -> f64 {
    match unit {
        RAD => value.to_radians(),
        GRAD => value / 0.9,
        _ => value,
    }
}

fn get_unit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, value.unit).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Number::new(scope, degrees(value)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let number = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |value| {
        value.specified = from_degrees(number, value.unit)
    });
}

fn get_specified(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Number::new(scope, value.specified).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_specified(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let number = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |value| value.specified = number);
}

fn get_as_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = value(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let suffix = match value.unit {
        DEG => "deg",
        RAD => "rad",
        GRAD => "grad",
        _ => "",
    };
    let text = format!("{}{suffix}", value.specified);
    if let Some(text) = v8::String::new(scope, &text) {
        result.set(text.into());
    }
}

fn set_as_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let (number, unit) = if let Some(number) = text.strip_suffix("deg") {
        (number, DEG)
    } else if let Some(number) = text.strip_suffix("rad") {
        (number, RAD)
    } else if let Some(number) = text.strip_suffix("grad") {
        (number, GRAD)
    } else {
        (text.as_str(), UNSPECIFIED)
    };
    let Ok(number) = number.trim().parse::<f64>() else {
        crate::webidl::throw_type_error(scope, "Invalid SVG angle");
        return;
    };
    update(scope, arguments.this(), |value| {
        value.unit = unit;
        value.specified = number;
    });
}

fn convert_to_units(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let unit = arguments.get(0).int32_value(scope).unwrap_or(UNKNOWN);
    if !(UNSPECIFIED..=GRAD).contains(&unit) {
        crate::webidl::throw_type_error(scope, "Invalid SVG angle unit");
        return;
    }
    update(scope, arguments.this(), |value| {
        let absolute = degrees(*value);
        value.unit = unit;
        value.specified = from_degrees(absolute, unit);
    });
}

fn new_value_units(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let unit = arguments.get(0).int32_value(scope).unwrap_or(UNKNOWN);
    let number = arguments.get(1).number_value(scope).unwrap_or(f64::NAN);
    if !(UNSPECIFIED..=GRAD).contains(&unit) {
        crate::webidl::throw_type_error(scope, "Invalid SVG angle unit");
        return;
    }
    update(scope, arguments.this(), |value| {
        value.unit = unit;
        value.specified = number;
    });
}
