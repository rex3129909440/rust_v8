pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "hasFocus", 0, has_focus)
}

fn has_focus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::document_method_support::ensure(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, true).into());
    }
}
