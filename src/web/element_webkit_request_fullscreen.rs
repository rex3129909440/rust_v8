pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "webkitRequestFullscreen",
        0,
        webkit_request_fullscreen,
    )
}

fn webkit_request_fullscreen(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    if let Some(document) = super::element_method_support::owner_document(scope, arguments.this()) {
        super::document::set_object_value(scope, document, "fullscreenElement", arguments.this());
    }
}
