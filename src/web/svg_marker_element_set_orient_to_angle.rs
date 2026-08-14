use super::svg_marker_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setOrientToAngle", 1, set_orient_to_angle)
}

fn set_orient_to_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(angle) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "setOrientToAngle requires an SVGAngle");
        return;
    };
    let Some(angle) = super::svg_angle::snapshot(scope, angle) else {
        crate::webidl::throw_type_error(scope, "setOrientToAngle requires an SVGAngle");
        return;
    };
    let orient_type = v8::Local::new(scope, &record.orient_type);
    let orient_angle = v8::Local::new(scope, &record.orient_angle);
    super::svg_animated_enumeration::set(scope, orient_type, ORIENT_ANGLE as u32);
    if let Err(error) = super::svg_animated_angle::set(scope, orient_angle, angle) {
        crate::webidl::throw_type_error(scope, &error);
    }
}
