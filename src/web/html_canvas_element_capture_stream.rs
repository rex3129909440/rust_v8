use super::html_canvas_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "captureStream", 0, capture_stream)
}

fn capture_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.transferred {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Cannot capture a canvas after control has been transferred",
        );
        return;
    }
    if !arguments.get(0).is_undefined() && arguments.get(0).number_value(scope).unwrap_or(0.0) < 0.0
    {
        throw_dom_exception(
            scope,
            "NotSupportedError",
            "The frame rate cannot be negative",
        );
        return;
    }
    let track = match super::canvas_capture_media_stream_track::create(scope, arguments.this()) {
        Ok(track) => track,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    match super::media_stream::create_with_tracks(scope, &[track]) {
        Ok(stream) => result.set(stream.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
