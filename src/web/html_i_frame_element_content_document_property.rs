use super::html_i_frame_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "contentDocument",
        get_content_document,
    )
}

fn get_content_document(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match content_document(s, a.this()) {
        Ok(Some(document)) => r.set(document.into()),
        Ok(None) => r.set(v8::null(s).into()),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
