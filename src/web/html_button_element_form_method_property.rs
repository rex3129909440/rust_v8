use super::html_button_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "formMethod",
        get_form_method,
        set_form_method,
    )
}

fn get_form_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.form_method);
}

fn set_form_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| {
        x.form_method = match v.to_ascii_lowercase().as_str() {
            "post" => "post".to_owned(),
            "dialog" => "dialog".to_owned(),
            _ => "get".to_owned(),
        }
    });
}
