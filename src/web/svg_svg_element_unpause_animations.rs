use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "unpauseAnimations", 0, unpause_animations)
}

fn unpause_animations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    update(scope, arguments.this(), |record| {
        if record.animations_paused {
            record.timeline_started_ms = Some(now);
        }
        record.animations_paused = false;
    });
}
