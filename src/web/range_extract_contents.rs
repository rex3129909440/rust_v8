pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "extractContents", 0, extract_contents)
}

fn extract_contents(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::range_contents::extract_contents(scope, arguments.this()) {
        Ok(fragment) => result.set(fragment.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
