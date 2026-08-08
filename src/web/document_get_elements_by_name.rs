pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getElementsByName",
        1,
        get_elements_by_name,
    )
}

fn get_elements_by_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let wanted = crate::webidl::value_to_string(scope, arguments.get(0));
    match super::node_list::create_live_named_elements(scope, arguments.this(), wanted) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
