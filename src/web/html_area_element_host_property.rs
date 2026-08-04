use super::html_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "host", get_host, set_host)
}

fn get_host(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| match (u.host_str(), u.port()) {
                (Some(h), Some(p)) => format!("{h}:{p}"),
                (Some(h), None) => h.to_owned(),
                _ => String::new(),
            })
            .unwrap_or_default()
    })
}

fn set_host(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            if let Ok(q) = ::url::Url::parse(&format!("{}://{v}/", u.scheme())) {
                let _ = u.set_host(q.host_str());
                let _ = u.set_port(q.port());
                x.href = u.as_str().to_owned();
            }
        }
    })
}
