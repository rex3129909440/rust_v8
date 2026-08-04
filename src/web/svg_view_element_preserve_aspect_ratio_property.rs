use super::svg_view_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "preserveAspectRatio",
        get_preserve_aspect_ratio,
    )
}

fn get_preserve_aspect_ratio(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.preserve_aspect_ratio).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
