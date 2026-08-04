use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "played", get_played)
}

fn get_played(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let ranges = if record.has_played && record.current_time > 0.0 {
            vec![(0.0, record.current_time)]
        } else {
            Vec::new()
        };
        empty_ranges(scope, result, ranges);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
