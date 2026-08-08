use super::html_i_frame_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "contentWindow", get_content_window)
}

fn get_content_window(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match content_window(s, a.this()) {
        Ok(Some(window)) => r.set(window.into()),
        Ok(None) => r.set(v8::null(s).into()),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
