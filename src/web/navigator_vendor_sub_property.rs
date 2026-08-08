pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "vendorSub", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !super::navigator::valid_this(scope, arguments.this()) {
        return;
    }
    let value = crate::fingerprint::navigator(scope).vendor_sub.clone();
    super::navigator::return_string(scope, &value, result);
}
