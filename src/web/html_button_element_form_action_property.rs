use super::html_button_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "formAction",
        get_form_action,
        set_form_action,
    )
}

fn get_form_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.form_action);
}

fn set_form_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| {
        x.form_action = if v.starts_with('/') {
            String::new()
        } else if v.is_empty() {
            "about:blank".to_owned()
        } else {
            v
        }
    });
}
