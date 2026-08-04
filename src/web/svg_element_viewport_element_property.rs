use super::svg_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "viewportElement",
        get_viewport_element,
    )
}

pub(crate) fn get_viewport_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_object(scope, record.viewport_element, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
