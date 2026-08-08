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
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let expression = crate::webidl::value_to_string(scope, arguments.get(0));
    let context = arguments.get(1);
    let requested_type = if arguments.get(3).is_undefined() {
        super::xpath_result::ANY_TYPE
    } else {
        arguments
            .get(3)
            .int32_value(scope)
            .unwrap_or(super::xpath_result::ANY_TYPE)
    };
    match super::xpath_expression::evaluate_source(scope, &expression, context, requested_type) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
