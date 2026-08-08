pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)
}

fn get_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::get_handler(
        scope,
        super::worker_global_scope::HandlerKind::Error,
        result,
    );
}

fn set_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::set_handler(
        scope,
        super::worker_global_scope::HandlerKind::Error,
        arguments.get(0),
    );
}
