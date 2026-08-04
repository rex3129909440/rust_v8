pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "queryCommandEnabled",
        1,
        query_command_enabled,
    )
}

fn query_command_enabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let command = crate::webidl::value_to_string(scope, arguments.get(0));
    result.set(
        v8::Boolean::new(
            scope,
            super::document_method_support::command_supported(&command),
        )
        .into(),
    );
}
