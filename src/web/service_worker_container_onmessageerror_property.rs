pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "onmessageerror", get, set)
}
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::service_worker_container::get_handler(
        scope,
        arguments,
        super::service_worker_container::HandlerKind::MessageError,
        result,
    )
}
fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::service_worker_container::set_handler(
        scope,
        arguments,
        super::service_worker_container::HandlerKind::MessageError,
    )
}
