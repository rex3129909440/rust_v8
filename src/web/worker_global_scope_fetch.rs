pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "fetch", 1, call)
}
fn call<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    result: v8::ReturnValue<'s>,
) {
    super::fetch_global::execute(scope, arguments, result)
}
