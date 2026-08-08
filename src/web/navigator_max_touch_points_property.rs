pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "maxTouchPoints", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::navigator::valid_this(scope, arguments.this()) {
        let value = crate::fingerprint::navigator(scope).max_touch_points;
        result.set(v8::Integer::new_from_unsigned(scope, value).into());
    }
}
