use super::svg_a_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "type", get_type, set_type)
}

fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.link_type, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !update(s, a.this(), |v| v.link_type = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
