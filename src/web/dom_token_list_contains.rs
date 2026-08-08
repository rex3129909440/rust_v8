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
    let Some(token) = super::dom_token_list::validate_token(scope, arguments.get(0)) else {
        return;
    };
    if let Some(values) = super::dom_token_list::list(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, values.contains(&token)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
