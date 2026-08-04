use super::html_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "decoding", get_decoding, set_decoding)
}

fn get_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(scope, arguments.this(), "decoding")
        .unwrap_or_else(|| "auto".to_owned())
        .to_ascii_lowercase();
    let value = if matches!(value.as_str(), "sync" | "async" | "auto") {
        value
    } else {
        "auto".to_owned()
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    } else {
        super::element::set_reflected_string(scope, arguments.this(), "decoding", value);
    }
}
