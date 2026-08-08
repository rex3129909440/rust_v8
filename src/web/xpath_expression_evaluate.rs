use super::xpath_expression::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "evaluate", 1, evaluate)
}

fn evaluate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "XPathExpression.evaluate requires a context node");
        return;
    }
    let id = arguments.this().get_identity_hash().get();
    let expression = scope
        .get_slot::<XPathExpressionStore>()
        .and_then(|store| store.records.get(&id))
        .cloned();
    let Some(expression) = expression else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let requested_type = if arguments.get(1).is_undefined() {
        super::xpath_result::ANY_TYPE
    } else {
        arguments
            .get(1)
            .int32_value(scope)
            .unwrap_or(super::xpath_result::ANY_TYPE)
    };
    match evaluate_source(scope, &expression, arguments.get(0), requested_type) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn call_argument<'a>(expression: &'a str, name: &str) -> Option<&'a str> {
    let inner = expression.strip_prefix(name)?.strip_prefix('(')?;
    inner.strip_suffix(')').map(str::trim)
}
