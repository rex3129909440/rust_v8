use super::html_video_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onenterpictureinpicture",
        get_on_enter_picture_in_picture,
        set_on_enter_picture_in_picture,
    )
}

fn get_on_enter_picture_in_picture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| &x.on_enter_picture_in_picture);
}

fn set_on_enter_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    update(scope, arguments.this(), |x| {
        x.on_enter_picture_in_picture = handler
    });
}
