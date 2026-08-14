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
    let mode = if let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        let mode_key = match crate::webidl::string(scope, "mode") {
            Ok(key) => key,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        options
            .get(scope, mode_key.into())
            .filter(|value| !value.is_undefined())
            .map(|value| crate::webidl::value_to_string(scope, value))
            .unwrap_or_else(|| "readwrite".to_owned())
    } else {
        "readwrite".to_owned()
    };
    if !matches!(
        mode.as_str(),
        "read-only" | "readwrite" | "readwrite-unsafe"
    ) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'createSyncAccessHandle' on 'FileSystemFileHandle': The provided value is not a valid enum value of type FileSystemSyncAccessHandleMode.",
        );
        return;
    }
    let bytes = match super::file_system_file_handle::shared_bytes(scope, arguments.this()) {
        Ok(bytes) => bytes,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let handle = match super::file_system_sync_access_handle::create(scope, bytes, mode) {
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
