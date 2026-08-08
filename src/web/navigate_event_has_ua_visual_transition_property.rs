use super::navigate_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "hasUAVisualTransition",
        get_has_ua_visual_transition,
    )
}

fn get_has_ua_visual_transition(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(scope, arguments, result, |record| {
        record.has_ua_visual_transition
    });
}
