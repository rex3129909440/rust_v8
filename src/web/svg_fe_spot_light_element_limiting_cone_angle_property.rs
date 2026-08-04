use super::svg_fe_spot_light_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "limitingConeAngle",
        get_limiting_cone_angle,
    )
}

fn get_limiting_cone_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        ret(s, &v.limiting_cone_angle, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
