use super::svg_geometry_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getPointAtLength", 1, get_point_at_length)
}

fn get_point_at_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let distance = arguments
        .get(0)
        .number_value(scope)
        .unwrap_or(0.0)
        .clamp(0.0, record.total_length);
    match super::svg_point::create(
        scope,
        super::svg_point::PointValue {
            x: distance,
            y: 0.0,
        },
    ) {
        Ok(point) => result.set(point.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
