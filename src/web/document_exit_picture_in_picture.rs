pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "exitPictureInPicture",
        0,
        exit_picture_in_picture,
    )
}

fn exit_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "Document",
            "exitPictureInPicture",
            result,
        );
        return;
    }
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    super::document::clear_value(scope, arguments.this(), "pictureInPictureElement");
    let undefined = v8::undefined(scope);
    match super::document_method_support::resolved(scope, undefined.into()) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
