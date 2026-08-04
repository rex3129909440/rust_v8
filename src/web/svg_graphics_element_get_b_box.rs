use super::svg_graphics_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getBBox", 0, get_bbox)
}

fn get_bbox(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::svg_rect::create_pair(
        scope,
        super::svg_rect::RectValue {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    ) {
        Ok((rect, _)) => result.set(rect.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}
