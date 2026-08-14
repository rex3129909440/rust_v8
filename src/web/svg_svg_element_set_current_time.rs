use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setCurrentTime", 1, set_current_time)
}

fn set_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(0.0);
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    update(scope, arguments.this(), |record| {
        record.current_time = value.max(0.0);
        if !record.animations_paused {
            record.timeline_started_ms = Some(now);
        }
    });
}
