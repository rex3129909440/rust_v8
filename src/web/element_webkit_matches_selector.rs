pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "webkitMatchesSelector", 1, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let selector = crate::webidl::value_to_string(scope, arguments.get(0));
    match super::dom_selector::matches_selector(
        scope,
        arguments.this(),
        &selector,
        arguments.this(),
    ) {
        Ok(value) => result.set(v8::Boolean::new(scope, value).into()),
        Err(message) => super::element::throw_selector_error(scope, message),
    }
}
