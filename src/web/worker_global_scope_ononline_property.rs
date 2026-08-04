pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "ononline", get_handler, set_handler)
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::get_handler(
        scope,
        super::worker_global_scope::HandlerKind::Online,
        result,
    );
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::set_handler(
        scope,
        super::worker_global_scope::HandlerKind::Online,
        arguments.get(0),
    );
}
