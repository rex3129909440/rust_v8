use super::html_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "crossOrigin",
        get_cross_origin,
        set_cross_origin,
    )
}

fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::element::attribute_value(scope, arguments.this(), "crossorigin") {
        None => result.set(v8::null(scope).into()),
        Some(value) => {
            let value = if value.eq_ignore_ascii_case("use-credentials") {
                "use-credentials"
            } else {
                "anonymous"
            };
            if let Some(value) = v8::String::new(scope, value) {
                result.set(value.into());
            }
        }
    }
}

fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.get(0).is_null() {
        super::element::remove_attribute_value(scope, arguments.this(), "crossorigin");
    } else {
        let value = crate::webidl::value_to_string(scope, arguments.get(0));
        super::element::set_reflected_string(scope, arguments.this(), "crossorigin", value);
    }
}
