use super::svg_geometry_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "isPointInStroke", 0, is_point_in_stroke)
}

fn is_point_in_stroke(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let inside = point_coordinates(scope, arguments.get(0))
        .is_some_and(|(x, y)| x >= 0.0 && x <= record.total_length && y.abs() <= 1.0);
    result.set(v8::Boolean::new(scope, inside).into());
}
