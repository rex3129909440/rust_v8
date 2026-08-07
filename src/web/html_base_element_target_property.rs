use super::html_base_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "target", get_target, set_target)
}

fn get_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        let target = super::element::reflected_string(s, a.this(), "target").unwrap_or_default();
        if let Some(v) = v8::String::new(s, &target) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if record(s, a.this()).is_some() {
        super::element::set_reflected_string(s, a.this(), "target", v);
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
