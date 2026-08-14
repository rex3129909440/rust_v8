pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "requestFullscreen", 0, request_fullscreen)
}

fn request_fullscreen(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "Element",
            "requestFullscreen",
            result,
        );
        return;
    }
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    if let Some(document) = super::element_method_support::owner_document(scope, arguments.this()) {
        super::document::set_object_value(scope, document, "fullscreenElement", arguments.this());
    }
    match super::element_method_support::resolved_undefined(scope) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
