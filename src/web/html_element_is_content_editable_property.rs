use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "isContentEditable",
        get_is_content_editable,
    )
}

pub(crate) fn get_is_content_editable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let editable = record
        .strings
        .get("contentEditable")
        .is_some_and(|value| value == "true" || value == "plaintext-only");
    result.set(v8::Boolean::new(scope, editable).into());
}
