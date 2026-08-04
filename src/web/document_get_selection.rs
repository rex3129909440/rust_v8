pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getSelection", 0, get_selection)
}

fn get_selection(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::selection::for_document(scope, arguments.this()) {
        Ok(selection) => result.set(selection.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
