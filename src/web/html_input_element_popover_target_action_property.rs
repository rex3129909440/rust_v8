use super::html_input_element::*;

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
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(s, a.this(), "popovertargetaction")
        .unwrap_or_else(|| "toggle".to_owned())
        .to_ascii_lowercase();
    let value = if matches!(value.as_str(), "show" | "hide" | "toggle") {
        value
    } else {
        "toggle".to_owned()
    };
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into());
    }
}

fn set_popover_target_action(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(scope, a, "popovertargetaction");
}
