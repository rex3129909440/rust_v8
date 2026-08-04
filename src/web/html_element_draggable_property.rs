use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "draggable", get_value, set_value)
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
    let value = super::element::attribute_value(scope, arguments.this(), "draggable");
    let enabled = match value.as_deref() {
        Some(value) if value.eq_ignore_ascii_case("true") => true,
        Some(_) => false,
        None => super::element::record(scope, arguments.this()).is_some_and(|record| {
            record.tag_name.eq_ignore_ascii_case("img")
                || (record.tag_name.eq_ignore_ascii_case("a")
                    && super::element::attribute_value(scope, arguments.this(), "href").is_some())
        }),
    };
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
        "draggable".to_owned(),
        value.to_owned(),
        None,
    );
}
