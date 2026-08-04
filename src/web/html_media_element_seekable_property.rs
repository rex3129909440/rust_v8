use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "seekable", get_seekable)
}

fn get_seekable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let ranges = if record.duration.is_finite() && record.duration > 0.0 {
            vec![(0.0, record.duration)]
        } else {
            Vec::new()
        };
        empty_ranges(scope, result, ranges);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
