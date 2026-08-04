use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "preload", get_preload, set_preload)
}

fn get_preload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.preload);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_preload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if matches!(value.as_str(), "none" | "metadata" | "auto" | "") {
        value
    } else {
        "metadata".to_owned()
    };
    update(scope, arguments.this(), |record| record.preload = value);
}
