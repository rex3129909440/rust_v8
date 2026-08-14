pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "scroll", 0, scroll)
}

fn scroll(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(scope, "Element", "scroll", result);
        return;
    }
    let (left, top) = super::element_method_support::scroll_coordinates(scope, &arguments);
    if !super::element::set_scroll_position(scope, arguments.this(), left, top, false) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(promise) = super::element_method_support::resolved_undefined(scope) {
        result.set(promise.into());
    }
}
