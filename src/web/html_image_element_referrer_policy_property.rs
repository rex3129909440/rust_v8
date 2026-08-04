use super::html_image_element::*;

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
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(scope, arguments.this(), "referrerpolicy")
        .unwrap_or_default();
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
    if let Some(value) = v8::String::new(scope, &normalized) {
        result.set(value.into());
    }
}

fn set_referrer_policy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else {
        super::element::set_reflected_string(scope, arguments.this(), "referrerpolicy", value);
    }
}
