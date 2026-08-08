pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "webkitCancelFullScreen",
        0,
        webkit_cancel_full_screen,
    )
}

fn webkit_cancel_full_screen(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::document_method_support::ensure(scope, arguments.this()) {
        super::document::clear_value(scope, arguments.this(), "fullscreenElement");
    }
}
