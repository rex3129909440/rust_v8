pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "cloneNode", 0, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::node::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::node::clone_object(
        scope,
        arguments.this(),
        arguments.get(0).boolean_value(scope),
    ) {
        Ok(clone) => result.set(clone.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
