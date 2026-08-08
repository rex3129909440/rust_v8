use super::html_video_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "cancelVideoFrameCallback",
        1,
        cancel_video_frame_callback,
    )
}

fn cancel_video_frame_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| {
        record.callbacks.remove(&id);
    });
}
