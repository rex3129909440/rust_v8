use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct XPathExpressionStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XPathExpressionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XPathExpression", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<XPathExpressionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XPathExpression",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::xpath_expression_evaluate::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XPathExpressionStore>()
        .ok_or_else(|| "XPathExpression state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    expression: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if expression.trim().is_empty() {
        return Err("The XPath expression is empty".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XPathExpression".to_owned());
    }
    scope
        .get_slot_mut::<XPathExpressionStore>()
        .ok_or_else(|| "XPathExpression state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), expression);
    Ok(object)
}

pub(crate) fn evaluate_source<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    expression: &str,
    context: v8::Local<'_, v8::Value>,
    requested_type: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let expression = expression.trim();
    let payload = if let Some(inner) =
        super::xpath_expression_evaluate::call_argument(expression, "string")
    {
        super::xpath_result::XPathPayload::String(unquote(inner))
    } else if let Some(inner) =
        super::xpath_expression_evaluate::call_argument(expression, "boolean")
    {
        let value = parse_number_expression(inner)
            .map(|number| number != 0.0 && !number.is_nan())
            .unwrap_or_else(|| !unquote(inner).is_empty());
        super::xpath_result::XPathPayload::Boolean(value)
    } else if let Some(inner) = super::xpath_expression_evaluate::call_argument(expression, "count")
    {
        let nodes = select_nodes(scope, inner, context)?;
        super::xpath_result::XPathPayload::Number(nodes.len() as f64)
    } else if let Some(number) = parse_number_expression(expression) {
        super::xpath_result::XPathPayload::Number(number)
    } else {
        super::xpath_result::XPathPayload::Nodes(select_nodes(scope, expression, context)?)
    };
    super::xpath_result::create(scope, requested_type, payload)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'XPathExpression': Illegal constructor",
    );
}

pub(crate) fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_owned();
        }
    }
    value.to_owned()
}

pub(crate) fn parse_number_expression(expression: &str) -> Option<f64> {
    let expression = expression.trim();
    if let Ok(number) = expression.parse::<f64>() {
        return Some(number);
    }
    let (left, right) = expression.split_once('+')?;
    Some(left.trim().parse::<f64>().ok()? + right.trim().parse::<f64>().ok()?)
}

pub(crate) fn select_nodes(
    scope: &mut v8::PinScope<'_, '_>,
    expression: &str,
    context: v8::Local<'_, v8::Value>,
) -> Result<Vec<v8::Global<v8::Object>>, String> {
    let context_object = v8::Local::<v8::Object>::try_from(context)
        .map_err(|_| "XPath context is not a Node".to_owned())?;
    let expression = expression.trim();
    if expression == "." {
        return Ok(vec![v8::Global::new(scope, context_object)]);
    }
    let selector = xpath_to_selector(expression)?;
    let method_key = v8::String::new(scope, "querySelectorAll")
        .ok_or_else(|| "cannot create querySelectorAll key".to_owned())?;
    let method = context_object
        .get(scope, method_key.into())
        .ok_or_else(|| "XPath context cannot be queried".to_owned())?;
    let method = v8::Local::<v8::Function>::try_from(method)
        .map_err(|_| "XPath context does not provide querySelectorAll".to_owned())?;
    let selector = v8::String::new(scope, &selector)
        .ok_or_else(|| "XPath selector is too large".to_owned())?;
    let value = method
        .call(scope, context_object.into(), &[selector.into()])
        .ok_or_else(|| "XPath node query failed".to_owned())?;
    collect_array_like(scope, value)
}

pub(crate) fn xpath_to_selector(expression: &str) -> Result<String, String> {
    let path = expression
        .strip_prefix(".//")
        .or_else(|| expression.strip_prefix("//"))
        .ok_or_else(|| "This XPath form is not supported by the offline evaluator".to_owned())?;
    let element = path.split('/').next().unwrap_or(path).trim();
    if element.is_empty() {
        return Err("XPath node test is empty".to_owned());
    }
    if element == "*" {
        return Ok("*".to_owned());
    }
    if element
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        Ok(element.to_owned())
    } else {
        Err("This XPath node test is not supported by the offline evaluator".to_owned())
    }
}

pub(crate) fn collect_array_like(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Vec<v8::Global<v8::Object>>, String> {
    let object = v8::Local::<v8::Object>::try_from(value)
        .map_err(|_| "querySelectorAll did not return an object".to_owned())?;
    let length_key =
        v8::String::new(scope, "length").ok_or_else(|| "cannot create length key".to_owned())?;
    let length = object
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let mut nodes = Vec::with_capacity(length as usize);
    for index in 0..length {
        let Some(value) = object.get_index(scope, index) else {
            continue;
        };
        if let Ok(node) = v8::Local::<v8::Object>::try_from(value) {
            nodes.push(v8::Global::new(scope, node));
        }
    }
    Ok(nodes)
}
