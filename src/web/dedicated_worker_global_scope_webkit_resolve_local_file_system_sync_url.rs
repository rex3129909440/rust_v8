pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        object,
        "webkitResolveLocalFileSystemSyncURL",
        1,
        call,
    )
}
fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(entry) = super::webkit_request_file_system::resolve_entry(scope, &input) {
        result.set(entry.into());
    } else {
        crate::webidl::throw_type_error(scope, "The requested file system URL was not found");
    }
}
