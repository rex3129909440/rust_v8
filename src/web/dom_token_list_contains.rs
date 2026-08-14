pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "contains", 1, contains)
}
fn contains(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'contains' on 'DOMTokenList': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(token) = super::dom_token_list::validate_token(scope, arguments.get(0), "contains")
    else {
        return;
    };
    if let Some(values) = super::dom_token_list::list(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, values.contains(&token)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
