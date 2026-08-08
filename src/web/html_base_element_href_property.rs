use super::html_base_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "href", get_href, set_href)
}

fn get_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        if let Some(v) = v8::String::new(s, &resolved_href(s, a.this())) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if record(s, a.this()).is_some() {
        super::element::set_reflected_string(s, a.this(), "href", v);
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
