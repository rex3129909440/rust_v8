use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "zoomAndPan",
        get_zoom_and_pan,
        set_zoom_and_pan,
    )
}

fn get_zoom_and_pan(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.zoom_and_pan).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_zoom_and_pan(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).int32_value(scope).unwrap_or(0);
    if value != ZOOM_AND_PAN_DISABLE && value != ZOOM_AND_PAN_MAGNIFY {
        crate::webidl::throw_type_error(scope, "Invalid SVG zoomAndPan value");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.zoom_and_pan = value
    });
}
