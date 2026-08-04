pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "deleteContents", 0, delete_contents)
}

fn delete_contents(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Err(message) = super::range_contents::delete_contents(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, &message);
    }
}
