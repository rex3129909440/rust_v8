pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, object, "onrtctransform", get, set)
}
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::get_handler(
        scope,
        super::worker_global_scope::HandlerKind::RtcTransform,
        result,
    )
}
fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::set_handler(
        scope,
        super::worker_global_scope::HandlerKind::RtcTransform,
        arguments.get(0),
    )
}
