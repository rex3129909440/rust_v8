use super::audio_processing_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "playbackTime", get_playback_time)
}

fn get_playback_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Number::new(s, v.playback_time).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
