pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "releasePointerCapture",
        1,
        release_pointer_capture,
    )
}

fn release_pointer_capture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let pointer_id = arguments.get(0).int32_value(scope).unwrap_or(0);
    if !super::element::set_pointer_capture_state(scope, arguments.this(), pointer_id, false) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
