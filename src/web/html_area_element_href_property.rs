use super::html_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "href", get_href, set_href)
}

fn get_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.href.clone())
}

fn set_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        x.url = ::url::Url::parse(&v).ok();
        x.href = x.url.as_ref().map(|u| u.as_str().to_owned()).unwrap_or(v)
    })
}
