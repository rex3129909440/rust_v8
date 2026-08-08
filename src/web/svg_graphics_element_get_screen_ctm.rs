use super::svg_graphics_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getScreenCTM", 0, get_ctm)
}

fn get_ctm(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::svg_matrix::create(scope, super::svg_matrix::MatrixValue::identity()) {
        Ok(matrix) => result.set(matrix.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
