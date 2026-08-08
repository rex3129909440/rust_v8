use super::html_input_element::*;

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
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(s, a.this(), "formmethod")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let value = match value.as_str() {
        "post" => "post",
        "dialog" => "dialog",
        _ => "get",
    };
    if let Some(value) = v8::String::new(s, value) {
        let mut r = r;
        r.set(value.into());
    }
}

fn set_form_method(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(s, a, "formmethod");
}
