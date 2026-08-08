pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "baseOffset", get_base_offset)
}

fn get_base_offset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::selection::record(scope, arguments.this()) {
        Some(record) => {
            result.set(v8::Integer::new_from_unsigned(scope, record.anchor_offset).into())
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
