pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, object, "webkitResolveLocalFileSystemURL", 2, call)
}
fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::webkit_resolve_local_file_system_url::execute(scope, arguments, result)
}
