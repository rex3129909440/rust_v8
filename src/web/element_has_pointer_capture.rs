pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "hasPointerCapture",
        1,
        has_pointer_capture,
    )
}

fn has_pointer_capture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let pointer_id = arguments.get(0).int32_value(scope).unwrap_or(0);
    match super::element::has_pointer_capture_state(scope, arguments.this(), pointer_id) {
        Some(captured) => result.set(v8::Boolean::new(scope, captured).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
