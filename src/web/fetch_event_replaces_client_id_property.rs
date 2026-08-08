pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "replacesClientId", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::fetch_event::record(scope, arguments.this()) {
        Some(record) => {
            if let Some(value) = v8::String::new(scope, &record.replaces_client_id) {
                result.set(value.into());
            }
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
