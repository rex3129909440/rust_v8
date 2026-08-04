pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "querySelector", 1, query_selector)
}

fn query_selector(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let selector = crate::webidl::value_to_string(scope, arguments.get(0));
    match super::dom_selector::query_selector_all(scope, arguments.this(), &selector) {
        Ok(matches) => match matches.first() {
            Some(element) => result.set((*element).into()),
            None => result.set(v8::null(scope).into()),
        },
        Err(message) => super::node::throw_dom_exception(scope, "SyntaxError", &message),
    }
}
