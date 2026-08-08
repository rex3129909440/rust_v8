use super::html_button_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "popoverTargetAction",
        get_popover_target_action,
        set_popover_target_action,
    )
}

fn get_popover_target_action(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.popover_target_action)
}

fn set_popover_target_action(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0)).to_ascii_lowercase();
    let value = match value.as_str() {
        "show" => "show",
        "hide" => "hide",
        _ => "toggle",
    }
    .to_owned();
    update(scope, a.this(), |x| x.popover_target_action = value)
}
