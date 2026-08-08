pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    _: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    crate::webidl::create_function(scope, "close", 0, v8::ConstructorBehavior::Throw, close)
}

fn close(
    _: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
}
