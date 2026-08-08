pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "startContainer", get_start_container)
}

fn get_start_container(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::abstract_range::record(scope, arguments.this()) {
        Some(record) => result.set(v8::Local::new(scope, &record.start_container).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
