use super::html_text_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "validationMessage",
        get_validation_message,
    )
}

fn get_validation_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let message = if !record.custom_validity.is_empty() {
        record.custom_validity
    } else if record.booleans.get("required").copied().unwrap_or(false) && record.value.is_empty() {
        "Please fill out this field.".to_owned()
    } else {
        String::new()
    };
    if let Some(value) = v8::String::new(scope, &message) {
        result.set(value.into());
    }
}
