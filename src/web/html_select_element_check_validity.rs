use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "checkValidity", 0, check_validity)
}

fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, x.disabled || !invalid(scope, a.this())).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
