use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "buffered", get_buffered)
}

fn get_buffered(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        empty_ranges(scope, result, Vec::new());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
