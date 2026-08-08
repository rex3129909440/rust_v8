use super::rtc_peer_connection_ice_error_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "errorText", get_error_text)
}

fn get_error_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => string_result(scope, &record.error_text, result),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
