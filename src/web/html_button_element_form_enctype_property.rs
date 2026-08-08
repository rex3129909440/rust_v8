use super::html_button_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "formEnctype",
        get_form_enctype,
        set_form_enctype,
    )
}

fn get_form_enctype(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.form_enctype);
}

fn set_form_enctype(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| {
        x.form_enctype = match v.to_ascii_lowercase().as_str() {
            "multipart/form-data" => "multipart/form-data".to_owned(),
            "text/plain" => "text/plain".to_owned(),
            _ => "application/x-www-form-urlencoded".to_owned(),
        }
    });
}
