use super::svg_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "decoding", get_decoding, set_decoding)
}

fn get_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        return_string(scope, &value.decoding, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if matches!(value.as_str(), "sync" | "async" | "auto") {
        value
    } else {
        "auto".to_owned()
    };
    update(scope, arguments.this(), |record| record.decoding = value);
}
