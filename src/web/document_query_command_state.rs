pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "queryCommandState",
        1,
        query_command_state,
    )
}

fn query_command_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::document_method_support::ensure(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, false).into());
    }
}
