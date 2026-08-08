use super::svg_fe_drop_shadow_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setStdDeviation", 2, set_std_deviation)
}

fn set_std_deviation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let x = a.get(0).number_value(s).unwrap_or(f64::NAN);
    let y = a.get(1).number_value(s).unwrap_or(f64::NAN);
    let xo = v8::Local::new(s, &v.std_deviation_x);
    let yo = v8::Local::new(s, &v.std_deviation_y);
    let _ = super::svg_animated_number::set_for_object(s, xo, x);
    let _ = super::svg_animated_number::set_for_object(s, yo, y);
}
