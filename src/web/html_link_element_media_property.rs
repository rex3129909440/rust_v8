use super::html_link_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "media", get_media, set_media)
}

fn get_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::reflected_string(s, a.this(), "media").unwrap_or_default();
    let mut r = r;
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into());
    }
}

fn set_media(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "media", value);
    refresh_connected(s, a.this());
}
