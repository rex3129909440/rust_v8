use super::html_geolocation_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "invalidReason", get_invalid_reason)
}

fn get_invalid_reason(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        return_text(s, "", r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
