pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::character_data::data_if_character(scope, arguments.this()) {
        Some(data) => result
            .set(v8::Integer::new_from_unsigned(scope, data.encode_utf16().count() as u32).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
