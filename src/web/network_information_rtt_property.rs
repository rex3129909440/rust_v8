pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "rtt", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = super::network_information::record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.rtt).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
