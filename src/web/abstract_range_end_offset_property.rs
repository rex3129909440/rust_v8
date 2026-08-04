pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "endOffset", get_end_offset)
}

fn get_end_offset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::abstract_range::record(scope, arguments.this()) {
        Some(record) => result.set(v8::Integer::new_from_unsigned(scope, record.end_offset).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
