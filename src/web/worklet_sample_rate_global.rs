pub(crate) fn install(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "sampleRate", get_sample_rate)
}

fn get_sample_rate(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let sample_rate = super::worklet::current_sample_rate(scope).unwrap_or(48_000.0);
    result.set(v8::Number::new(scope, sample_rate).into());
}
