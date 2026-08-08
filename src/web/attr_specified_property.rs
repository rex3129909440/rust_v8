pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "specified", get_specified)
}

fn get_specified(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::attr::record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, true).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
