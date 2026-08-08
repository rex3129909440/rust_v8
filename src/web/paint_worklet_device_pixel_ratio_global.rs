pub(crate) fn install(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "devicePixelRatio",
        get_device_pixel_ratio,
    )
}

fn get_device_pixel_ratio(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = super::worklet::current_device_pixel_ratio(scope).unwrap_or(1.0);
    result.set(v8::Number::new(scope, value).into());
}
