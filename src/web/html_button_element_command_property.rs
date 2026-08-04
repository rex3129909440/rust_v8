use super::html_button_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "command", get_command, set_command)
}

fn get_command(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.command)
}

fn set_command(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    let valid = value.starts_with("--")
        || matches!(
            value.as_str(),
            "show-popover"
                | "hide-popover"
                | "toggle-popover"
                | "show-modal"
                | "close"
                | "request-close"
        );
    update(scope, a.this(), |x| {
        x.command = if valid { value } else { String::new() }
    })
}
