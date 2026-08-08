use super::svg_view_element::*;

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
    let value = arguments
        .get(0)
        .int32_value(scope)
        .unwrap_or(ZOOM_AND_PAN_UNKNOWN);
    if value != ZOOM_AND_PAN_DISABLE && value != ZOOM_AND_PAN_MAGNIFY {
        crate::webidl::throw_type_error(scope, "Invalid SVG zoomAndPan value");
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<SvgViewElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.zoom_and_pan = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
