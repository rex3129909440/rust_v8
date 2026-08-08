#[derive(Default)]
pub(crate) struct CssNumericValueStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssNumericValueStore::default());
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSNumericValue", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssNumericValueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSNumericValue",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "add", 0, add)?;
    crate::webidl::define_method(scope, prototype, "div", 0, div)?;
    crate::webidl::define_method(scope, prototype, "equals", 0, equals)?;
    crate::webidl::define_method(scope, prototype, "max", 0, max)?;
    crate::webidl::define_method(scope, prototype, "min", 0, min)?;
    crate::webidl::define_method(scope, prototype, "mul", 0, mul)?;
    crate::webidl::define_method(scope, prototype, "sub", 0, sub)?;
    crate::webidl::define_method(scope, prototype, "to", 1, convert)?;
    crate::webidl::define_method(scope, prototype, "toSum", 0, to_sum)?;
    crate::webidl::define_method(scope, prototype, "type", 0, numeric_type)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "parse", 1, parse)?;
    let parent = super::css_style_value::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssNumericValueStore>()
        .ok_or_else(|| "CSSNumericValue state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn is_numeric(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    super::css_unit_value::record(scope, object).is_some()
        || super::css_math_value::is_math(scope, object)
}

pub(crate) fn numberish(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Global<v8::Object>, String> {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value)
        && is_numeric(scope, object)
    {
        return Ok(v8::Global::new(scope, object));
    }
    if value.is_number() {
        let number = value
            .number_value(scope)
            .ok_or_else(|| "CSS numeric value must be finite".to_owned())?;
        if !number.is_finite() {
            return Err("CSS numeric value must be finite".to_owned());
        }
        let object = super::css_unit_value::create(scope, number, "number")?;
        return Ok(v8::Global::new(scope, object));
    }
    Err("Value is not a CSS numeric value".to_owned())
}

fn unit_dimension(unit: &str) -> &str {
    match unit {
        "number" => "number",
        "percent" | "%" => "percent",
        "px" | "cm" | "mm" | "q" | "in" | "pc" | "pt" | "em" | "rem" | "ex" | "rex" | "cap"
        | "rcap" | "ch" | "rch" | "ic" | "ric" | "lh" | "rlh" | "vw" | "vh" | "vi" | "vb"
        | "vmin" | "vmax" | "svw" | "svh" | "svi" | "svb" | "svmin" | "svmax" | "lvw" | "lvh"
        | "lvi" | "lvb" | "lvmin" | "lvmax" | "dvw" | "dvh" | "dvi" | "dvb" | "dvmin" | "dvmax"
        | "cqw" | "cqh" | "cqi" | "cqb" | "cqmin" | "cqmax" => "length",
        "deg" | "grad" | "rad" | "turn" => "angle",
        "s" | "ms" => "time",
        "hz" | "khz" => "frequency",
        "dpi" | "dpcm" | "dppx" => "resolution",
        "fr" => "flex",
        other => other,
    }
}

pub(crate) fn compatible(scope: &v8::PinScope<'_, '_>, values: &[v8::Global<v8::Object>]) -> bool {
    let mut dimensions = Vec::<String>::new();
    for value in values {
        let value = v8::Local::new(scope, value);
        let Some(record) = super::css_unit_value::record(scope, value) else {
            continue;
        };
        let dimension = unit_dimension(&record.unit).to_owned();
        if !dimensions.contains(&dimension) {
            dimensions.push(dimension);
        }
    }
    dimensions.len() <= 1
        || dimensions
            .iter()
            .all(|dimension| matches!(dimension.as_str(), "length" | "percent"))
}

fn parse(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let parsed = parse_numeric_object(scope, &value);
    match parsed {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn parse_numeric_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if let Some((number, unit)) = parse_dimension(value) {
        return super::css_unit_value::create(scope, number, canonical_unit(unit));
    }
    if let Ok(number) = value.trim().parse::<f64>()
        && number.is_finite()
    {
        return super::css_unit_value::create(scope, number, "number");
    }
    let normalized = super::css_calculation::normalize_numeric_value(value)
        .ok_or_else(|| "Invalid CSS numeric value".to_owned())?;
    let inner = normalized
        .strip_prefix("calc(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| "Invalid CSS numeric value".to_owned())?;
    let original_is_calc = value.trim().to_ascii_lowercase().starts_with("calc(");
    if !original_is_calc {
        if let Some((number, unit)) = parse_dimension(inner) {
            return super::css_unit_value::create(scope, number, canonical_unit(unit));
        }
        if let Ok(number) = inner.parse::<f64>() {
            return super::css_unit_value::create(scope, number, "number");
        }
    }
    let mut terms = Vec::<v8::Local<'_, v8::Value>>::new();
    for term in split_sum(inner) {
        let object = if let Some((number, unit)) = parse_dimension(term) {
            super::css_unit_value::create(scope, number, canonical_unit(unit))?
        } else if let Ok(number) = term.trim().parse::<f64>() {
            super::css_unit_value::create(scope, number, "number")?
        } else {
            return Err("CSS numeric expression is not representable in Typed OM".to_owned());
        };
        terms.push(object.into());
    }
    construct_named(scope, "CSSMathSum", &terms)
}

fn canonical_unit(unit: &str) -> &str {
    if unit == "%" { "percent" } else { unit }
}

fn split_sum(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut output = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (offset, byte) in bytes.iter().copied().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'+' if depth == 0 => {
                output.push(value[start..offset].trim());
                start = offset + 1;
            }
            _ => {}
        }
    }
    output.push(value[start..].trim());
    output
}

fn construct_named<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    arguments: &[v8::Local<'_, v8::Value>],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let key = v8::String::new(scope, name).ok_or_else(|| format!("cannot create {name}"))?;
    let global = scope.get_current_context().global(scope);
    let constructor = global
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| format!("{name} constructor is unavailable"))?;
    constructor
        .new_instance(scope, arguments)
        .ok_or_else(|| format!("cannot create {name}"))
}

fn parse_dimension(value: &str) -> Option<(f64, &str)> {
    let value = value.trim();
    let split = value
        .char_indices()
        .find(|(_, character)| character.is_ascii_alphabetic() || *character == '%')
        .map(|(index, _)| index)?;
    let number = value[..split].parse::<f64>().ok()?;
    let unit = value[split..].trim();
    (!unit.is_empty() && number.is_finite()).then_some((number, unit))
}

fn operands(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<(
    super::css_unit_value::CssUnitRecord,
    super::css_unit_value::CssUnitRecord,
)> {
    let left = super::css_unit_value::record(scope, arguments.this())?;
    let right_object = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok()?;
    let right = super::css_unit_value::record(scope, right_object)?;
    Some((left, right))
}

fn same_unit(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<(
    super::css_unit_value::CssUnitRecord,
    super::css_unit_value::CssUnitRecord,
)> {
    let (left, right) = operands(scope, arguments)?;
    (left.unit == right.unit).then_some((left, right))
}

fn return_unit(
    scope: &mut v8::PinScope<'_, '_>,
    value: f64,
    unit: &str,
    mut result: v8::ReturnValue<'_>,
) {
    match super::css_unit_value::create(scope, value, unit) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some((left, right)) = same_unit(scope, &arguments) {
        return_unit(scope, left.value + right.value, &left.unit, result);
    } else {
        crate::webidl::throw_type_error(scope, "Incompatible CSS units");
    }
}

fn sub(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some((left, right)) = same_unit(scope, &arguments) {
        return_unit(scope, left.value - right.value, &left.unit, result);
    } else {
        crate::webidl::throw_type_error(scope, "Incompatible CSS units");
    }
}

fn mul(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(left) = super::css_unit_value::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let factor = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if factor.is_finite() {
        return_unit(scope, left.value * factor, &left.unit, result);
    } else {
        crate::webidl::throw_type_error(scope, "Multiplier must be finite");
    }
}

fn div(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(left) = super::css_unit_value::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let divisor = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if divisor.is_finite() && divisor != 0.0 {
        return_unit(scope, left.value / divisor, &left.unit, result);
    } else {
        crate::webidl::throw_type_error(scope, "Divisor must be finite and non-zero");
    }
}

fn min(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some((left, right)) = same_unit(scope, &arguments) {
        return_unit(scope, left.value.min(right.value), &left.unit, result);
    } else {
        crate::webidl::throw_type_error(scope, "Incompatible CSS units");
    }
}

fn max(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some((left, right)) = same_unit(scope, &arguments) {
        return_unit(scope, left.value.max(right.value), &left.unit, result);
    } else {
        crate::webidl::throw_type_error(scope, "Incompatible CSS units");
    }
}

fn equals(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let equal = operands(scope, &arguments)
        .is_some_and(|(left, right)| left.unit == right.unit && left.value == right.value);
    result.set(v8::Boolean::new(scope, equal).into());
}

fn convert(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = super::css_unit_value::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let unit = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    if unit == value.unit {
        return_unit(scope, value.value, &unit, result);
    } else {
        crate::webidl::throw_type_error(scope, "Unit conversion is not available");
    }
}

fn to_sum(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::css_unit_value::record(scope, arguments.this()).is_some() {
        result.set(arguments.this().into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn numeric_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = super::css_unit_value::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    let category = match value.unit.as_str() {
        "deg" | "rad" | "grad" | "turn" => "angle",
        "s" | "ms" => "time",
        "hz" | "khz" => "frequency",
        "dpi" | "dpcm" | "dppx" => "resolution",
        "percent" | "%" => "percent",
        _ => "length",
    };
    if let Some(key) = v8::String::new(scope, category) {
        let _ = object.set(scope, key.into(), v8::Integer::new(scope, 1).into());
    }
    if let Some(key) = v8::String::new(scope, "length") {
        let _ = object.set(scope, key.into(), v8::Integer::new(scope, 1).into());
    }
    result.set(object.into());
}
