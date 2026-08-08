use super::window_controls_overlay_geometry_change_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "titlebarAreaRect",
        get_titlebar_area_rect,
    )
}

fn get_titlebar_area_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(rect) = record.titlebar_area_rect {
        result.set(v8::Local::new(scope, &rect).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
