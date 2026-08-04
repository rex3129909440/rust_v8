pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "brands", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = super::navigator_ua_data::record(scope, arguments.this()) {
        result.set(super::navigator_ua_data::brands_array(scope, &record, false).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
