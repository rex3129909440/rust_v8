use super::math_ml_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "style", get_style, set_style)
}

pub(crate) fn get_style(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| &x.style);
}

pub(crate) fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let style = v8::Local::new(scope, &record.style);
    if let Some(key) = v8::String::new(scope, "cssText")
        && let Some(value) = arguments.get(0).to_string(scope)
    {
        let _ = style.set(scope, key.into(), value.into());
    }
}
