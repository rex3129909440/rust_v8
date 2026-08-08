pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "scroll", 0, scroll)
}

fn scroll(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let (left, top) = super::element_method_support::scroll_coordinates(scope, &arguments);
    if !super::element::set_scroll_position(scope, arguments.this(), left, top, false) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
