pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "preloadResponse", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::fetch_event::record(scope, arguments.this()) {
        Some(record) => result.set(v8::Local::new(scope, &record.preload_response).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
