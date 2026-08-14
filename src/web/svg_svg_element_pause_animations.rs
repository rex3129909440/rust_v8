use super::svg_svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "pauseAnimations", 0, pause_animations)
}

fn pause_animations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let current_time = current_time(scope, arguments.this());
    update(scope, arguments.this(), |record| {
        if let Some(current_time) = current_time {
            record.current_time = current_time;
        }
        record.timeline_started_ms = None;
        record.animations_paused = true;
    });
}
