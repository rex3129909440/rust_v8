use super::html_fenced_frame_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "canLoadOpaqueURL", 0, can_load_opaque_url)
}

fn can_load_opaque_url(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Boolean::new(scope, false).into());
}
