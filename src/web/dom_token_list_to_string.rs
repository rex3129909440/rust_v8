pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)
}

fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::dom_token_list::string_value(scope, arguments.this()) {
        Some(text) => {
            if let Some(value) = v8::String::new(scope, &text) {
                result.set(value.into());
            }
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
