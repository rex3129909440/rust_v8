use super::html_script_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "type", get_string, set_string)
}

fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::reflected_string(scope, a.this(), "type").unwrap_or_default();
    if let Some(v) = v8::String::new(scope, &value) {
        r.set(v.into())
    }
}

fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::element::set_reflected_string(scope, a.this(), "type", v);
}
