use super::svg_fe_specular_lighting_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "surfaceScale", get_surface_scale)
}

fn get_surface_scale(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.surface_scale, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
