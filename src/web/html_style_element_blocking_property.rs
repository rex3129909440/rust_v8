use super::html_style_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "blocking", get_blocking, set_blocking)
}

fn get_blocking(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &x.blocking).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_blocking(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let blocking = v8::Local::new(scope, &record.blocking);
        super::dom_token_list::set_string_value(scope, blocking, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
