pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "exitPointerLock", 0, exit_pointer_lock)
}

fn exit_pointer_lock(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::document_method_support::ensure(scope, arguments.this()) {
        super::document::clear_value(scope, arguments.this(), "pointerLockElement");
    }
}
