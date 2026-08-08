use super::html_input_element::*;

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
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(s, a.this(), "formenctype")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let value = match value.as_str() {
        "multipart/form-data" => "multipart/form-data",
        "text/plain" => "text/plain",
        _ => "application/x-www-form-urlencoded",
    };
    if let Some(value) = v8::String::new(s, value) {
        let mut r = r;
        r.set(value.into());
    }
}

fn set_form_enctype(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(s, a, "formenctype");
}
