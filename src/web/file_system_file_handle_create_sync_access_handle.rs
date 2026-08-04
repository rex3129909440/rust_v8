pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createSyncAccessHandle",
        0,
        create_sync_access_handle,
    )
}

fn create_sync_access_handle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let bytes = match super::file_system_file_handle::shared_bytes(scope, arguments.this()) {
        Ok(bytes) => bytes,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let handle = match super::file_system_sync_access_handle::create(scope, bytes) {
        Ok(handle) => handle,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    match super::writable_stream::resolved_promise(scope, handle.into()) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
