use super::svg_component_transfer_function_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "amplitude", get_amplitude)
}

fn get_amplitude(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.amplitude, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
