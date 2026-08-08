use super::html_video_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getVideoPlaybackQuality",
        0,
        get_video_playback_quality,
    )
}

fn get_video_playback_quality(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match super::video_playback_quality::create(
        scope,
        0.0,
        record.decoded_frames,
        record.dropped_frames,
        0,
    ) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
