pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "oncanmakepayment", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::get_service_handler(
        scope,
        super::worker_global_scope::ServiceHandlerKind::CanMakePayment,
        result,
    )
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::set_service_handler(
        scope,
        super::worker_global_scope::ServiceHandlerKind::CanMakePayment,
        arguments.get(0),
    )
}
