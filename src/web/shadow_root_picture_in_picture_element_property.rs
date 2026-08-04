pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "pictureInPictureElement",
        get_picture_in_picture_element,
    )
}

fn get_picture_in_picture_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::shadow_root::document_scoped_element(
        scope,
        arguments.this(),
        "pictureInPictureElement",
    ) {
        Some(element) => result.set(element.into()),
        None => result.set(v8::null(scope).into()),
    }
}
