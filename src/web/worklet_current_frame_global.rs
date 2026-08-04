pub(crate) fn install(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "currentFrame", get_current_frame)
}

fn get_current_frame(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let frame = super::worklet::current_frame(scope).unwrap_or_default();
    result.set(v8::Number::new(scope, frame as f64).into());
}
