use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "size", get_size, set_size)
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else {
        let value = super::element::attribute_value(scope, a.this(), "size")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        r.set(v8::Integer::new_from_unsigned(scope, value).into())
    }
}

fn set_size(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(scope).unwrap_or(0);
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else {
        super::element::set_reflected_string(scope, a.this(), "size", v.to_string());
    }
}
