pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "querySelectorAll", 1, query_selector_all)
}

fn query_selector_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let selector = crate::webidl::value_to_string(scope, arguments.get(0));
    let matches = match super::dom_selector::query_selector_all(scope, arguments.this(), &selector)
    {
        Ok(matches) => matches,
        Err(message) => {
            super::node::throw_dom_exception(scope, "SyntaxError", &message);
            return;
        }
    };
    match super::node_list::create(scope, matches) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
