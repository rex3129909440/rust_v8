use super::html_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "protocol", get_protocol, set_protocol)
}

fn get_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| format!("{}:", u.scheme()))
            .unwrap_or_else(|| ":".to_owned())
    })
}

fn set_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0))
        .trim_end_matches(':')
        .to_owned();
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            let _ = u.set_scheme(&v);
            x.href = u.as_str().to_owned()
        }
    })
}
