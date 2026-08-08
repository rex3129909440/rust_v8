use super::svg_graphics_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "requiredExtensions",
        get_required_extensions,
    )
}

fn get_required_extensions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_object(scope, &record.required_extensions, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
