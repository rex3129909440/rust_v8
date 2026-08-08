use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "spellcheck", get_value, set_value)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let enabled = super::element::attribute_value(scope, arguments.this(), "spellcheck")
        .is_none_or(|value| !value.eq_ignore_ascii_case("false"));
    result.set(v8::Boolean::new(scope, enabled).into());
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = if arguments.get(0).boolean_value(scope) {
        "true"
    } else {
        "false"
    };
    super::element::set_attribute_full(
        scope,
        arguments.this(),
        "spellcheck".to_owned(),
        value.to_owned(),
        None,
    );
}
