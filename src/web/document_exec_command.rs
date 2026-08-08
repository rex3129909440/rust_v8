pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "execCommand", 1, exec_command)
}

fn exec_command(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let command = crate::webidl::value_to_string(scope, arguments.get(0));
    let supported = super::document_method_support::command_supported(&command);
    if supported {
        super::document::set_string_value(scope, arguments.this(), "lastCommand", &command);
        let value = crate::webidl::value_to_string(scope, arguments.get(2));
        super::document::set_string_value(scope, arguments.this(), "lastCommandValue", &value);
    }
    result.set(v8::Boolean::new(scope, supported).into());
}
