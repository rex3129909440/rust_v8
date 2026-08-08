use super::html_anchor_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "referrerPolicy",
        get_referrer_policy,
        set_referrer_policy,
    )
}

fn get_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(s, a.this(), "referrerpolicy").unwrap_or_default();
    let normalized = if matches!(
        value.to_ascii_lowercase().as_str(),
        "" | "no-referrer"
            | "origin"
            | "no-referrer-when-downgrade"
            | "origin-when-cross-origin"
            | "unsafe-url"
            | "same-origin"
            | "strict-origin"
            | "strict-origin-when-cross-origin"
    ) {
        value.to_ascii_lowercase()
    } else {
        String::new()
    };
    if let Some(value) = v8::String::new(s, &normalized) {
        r.set(value.into());
    }
}

fn set_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(s, a, "referrerpolicy")
}
