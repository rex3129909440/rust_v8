use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssStyleValueStore {
    constructor: crate::webidl::RealmConstructor,
    serialized: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssStyleValueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSStyleValue", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssStyleValueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSStyleValue",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "parse", 2, parse)?;
    crate::webidl::define_method(scope, constructor.into(), "parseAll", 2, parse_all)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssStyleValueStore>()
        .ok_or_else(|| "CSSStyleValue state was not prepared".to_owned())?
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

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    serialized: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSStyleValue".to_owned());
    }
    scope
        .get_slot_mut::<CssStyleValueStore>()
        .ok_or_else(|| "CSSStyleValue state was not prepared".to_owned())?
        .serialized
        .insert(object.get_identity_hash().get(), serialized);
    Ok(object)
}

fn stored_text(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<String> {
    scope
        .get_slot::<CssStyleValueStore>()?
        .serialized
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    if let Some(record) = super::css_unit_value::record(scope, object) {
        return Some(super::css_unit_value::serialize(&record));
    }
    if let Some(value) = super::css_unparsed_value::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_transform_value::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_position_value::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_math_sum::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_math_product::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_math_negate::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_math_min::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_math_max::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_math_invert::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_math_clamp::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_keyword_value::serialize(scope, object) {
        return Some(value);
    }
    if let Some(value) = super::css_image_value::serialize(scope, object) {
        return Some(value);
    }
    stored_text(scope, object)
}

fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = serialize(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn dimension(value: &str) -> Option<(f64, &str)> {
    let value = value.trim();
    let split = value
        .char_indices()
        .find(|(_, character)| character.is_ascii_alphabetic() || *character == '%')
        .map(|(index, _)| index)?;
    if split == 0 {
        return None;
    }
    let number = value[..split].trim().parse::<f64>().ok()?;
    let unit = value[split..].trim();
    if unit.is_empty() || !number.is_finite() {
        None
    } else {
        Some((number, unit))
    }
}

fn parse_one<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    property: &str,
    value: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("CSS value cannot be empty".to_owned());
    }
    if matches!(
        property.trim().to_ascii_lowercase().as_str(),
        "background-image" | "border-image-source" | "list-style-image" | "mask-image"
    ) && (value.to_ascii_lowercase().starts_with("url(")
        || value.to_ascii_lowercase().contains("gradient("))
    {
        super::css_image_value::create(scope, value.to_owned())
    } else if super::css_calculation::is_root_numeric_math(value)
        && super::css_style_declaration::supports_property(property, value)
    {
        super::css_numeric_value::parse_numeric_object(scope, value)
    } else if let Some((number, unit)) = dimension(value) {
        super::css_unit_value::create(scope, number, &unit.to_ascii_lowercase())
    } else {
        create(scope, value.to_owned())
    }
}

fn parse(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "CSSStyleValue.parse requires a property and value");
        return;
    }
    let property = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    if property.trim().is_empty() {
        crate::webidl::throw_type_error(scope, "CSS property cannot be empty");
        return;
    }
    match parse_one(scope, &property, &value) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn split_commas(value: &str) -> Vec<&str> {
    let mut output = Vec::new();
    let mut start = 0;
    let mut depth = 0_u32;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                output.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    output.push(value[start..].trim());
    output
}

fn parse_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "CSSStyleValue.parseAll requires a property and value",
        );
        return;
    }
    let property = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    if property.trim().is_empty() {
        crate::webidl::throw_type_error(scope, "CSS property cannot be empty");
        return;
    }
    let pieces = split_commas(&value);
    let array = v8::Array::new(scope, pieces.len() as i32);
    for (index, piece) in pieces.into_iter().enumerate() {
        let parsed = match parse_one(scope, &property, piece) {
            Ok(parsed) => parsed,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        let _ = array.set_index(scope, index as u32, parsed.into());
    }
    result.set(array.into());
}
