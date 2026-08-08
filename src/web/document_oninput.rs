pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "oninput", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::document::handler_value(scope, arguments.this(), "oninput") {
        Some(value) => result.set(v8::Local::new(scope, &value)),
        None => result.set(v8::null(scope).into()),
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document::set_handler(scope, arguments.this(), "oninput", arguments.get(0)) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
