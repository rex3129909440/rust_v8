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
    if !super::document_fragment::valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'querySelector' on 'DocumentFragment': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(selector) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'querySelector' on 'DocumentFragment'",
    ) else {
        return;
    };
    match super::dom_selector::query_selector_all(scope, arguments.this(), &selector) {
        Ok(values) => match values.first() {
            Some(element) => result.set((*element).into()),
            None => result.set(v8::null(scope).into()),
        },
        Err(_) => super::dom_selector::throw_api_error(
            scope,
            "querySelector",
            "DocumentFragment",
            &selector,
        ),
    }
}
