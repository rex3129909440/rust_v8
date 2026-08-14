use super::html_video_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "requestPictureInPicture",
        0,
        request_picture_in_picture,
    )
}

fn request_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "HTMLVideoElement",
            "requestPictureInPicture",
            result,
        );
        return;
    };
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    if record.disable_picture_in_picture {
        let message = v8::String::new(scope, "Picture-in-Picture is disabled")
            .map(v8::Local::<v8::Value>::from)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let _ = resolver.reject(scope, message);
    } else if let Ok(window) =
        super::picture_in_picture_window::create(scope, record.width as i32, record.height as i32)
    {
        let _ = resolver.resolve(scope, window.into());
    }
    result.set(resolver.get_promise(scope).into());
}
