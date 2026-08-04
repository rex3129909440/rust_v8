pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "deviceMemory", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::navigator::valid_this(scope, arguments.this()) {
        let value = crate::fingerprint::navigator(scope).device_memory_gb;
        result.set(v8::Number::new(scope, value).into());
    }
}
