use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "hidden", get_value, set_value)
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
    match super::element::attribute_value(scope, arguments.this(), "hidden") {
        Some(value) if value.eq_ignore_ascii_case("until-found") => {
            if let Some(value) = v8::String::new(scope, "until-found") {
                result.set(value.into());
            }
        }
        Some(_) => result.set(v8::Boolean::new(scope, true).into()),
        None => result.set(v8::Boolean::new(scope, false).into()),
    }
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
    let input = arguments.get(0);
    if input.is_string()
        && crate::webidl::value_to_string(scope, input).eq_ignore_ascii_case("until-found")
    {
        super::element::set_attribute_full(
            scope,
            arguments.this(),
            "hidden".to_owned(),
            "until-found".to_owned(),
            None,
        );
    } else if input.boolean_value(scope) {
        super::element::set_attribute_full(
            scope,
            arguments.this(),
            "hidden".to_owned(),
            String::new(),
            None,
        );
    } else {
        super::element::remove_attribute_full(scope, arguments.this(), None, "hidden");
    }
}
