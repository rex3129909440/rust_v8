use super::html_button_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "formNoValidate",
        get_form_no_validate,
        set_form_no_validate,
    )
}

fn get_form_no_validate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, x.form_no_validate).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_form_no_validate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    update(scope, a.this(), |x| x.form_no_validate = value);
}
