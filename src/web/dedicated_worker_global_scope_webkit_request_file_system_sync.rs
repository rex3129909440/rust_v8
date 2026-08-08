pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, object, "webkitRequestFileSystemSync", 2, call)
}
fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(origin) = super::webkit_request_file_system::secure_origin(scope) else {
        crate::webidl::throw_type_error(scope, "The worker origin cannot access a file system");
        return;
    };
    let file_system_type = arguments.get(0).int32_value(scope).unwrap_or(0);
    if file_system_type != 0 && file_system_type != 1 {
        crate::webidl::throw_type_error(scope, "The requested file system type is not supported");
        return;
    }
    match super::webkit_request_file_system::get_or_create_file_system(
        scope,
        &origin,
        file_system_type,
    ) {
        Ok(file_system) => result.set(file_system.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
