use super::rtc_peer_connection_ice_error_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "errorCode", get_error_code)
}

fn get_error_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => result.set(v8::Integer::new_from_unsigned(scope, record.error_code).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
