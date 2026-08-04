use super::xpath_evaluator::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "evaluate", 2, evaluate)
}

fn evaluate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_instance(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "XPathEvaluator.evaluate requires 2 arguments");
        return;
    }
    let expression = crate::webidl::value_to_string(scope, arguments.get(0));
    let requested_type = if arguments.get(3).is_undefined() {
        super::xpath_result::ANY_TYPE
    } else {
        arguments
            .get(3)
            .int32_value(scope)
            .unwrap_or(super::xpath_result::ANY_TYPE)
    };
    match super::xpath_expression::evaluate_source(
        scope,
        &expression,
        arguments.get(1),
        requested_type,
    ) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
