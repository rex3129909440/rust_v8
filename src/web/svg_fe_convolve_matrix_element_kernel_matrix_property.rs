use super::svg_fe_convolve_matrix_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "kernelMatrix", get_kernel_matrix)
}

fn get_kernel_matrix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.kernel_matrix, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
