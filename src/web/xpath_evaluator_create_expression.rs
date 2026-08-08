use super::xpath_evaluator::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createExpression", 1, create_expression)
}

fn create_expression(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_instance(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "createExpression requires an expression");
        return;
    }
    let expression = crate::webidl::value_to_string(scope, arguments.get(0));
    match super::xpath_expression::create(scope, expression) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
