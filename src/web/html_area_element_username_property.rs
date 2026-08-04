use super::html_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "username", get_username, set_username)
}

fn get_username(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| u.username().to_owned())
            .unwrap_or_default()
    })
}

fn set_username(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            let _ = u.set_username(&v);
            x.href = u.as_str().to_owned()
        }
    })
}
