use super::html_field_set_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "checkValidity", 0, check_validity)
}

fn check_validity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, true).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
