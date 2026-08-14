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
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'querySelectorAll' on 'Element': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(selector) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'querySelectorAll' on 'Element'",
    ) else {
        return;
    };
    let matches = match super::dom_selector::query_selector_all(scope, arguments.this(), &selector)
    {
        Ok(matches) => matches,
        Err(_) => {
            super::dom_selector::throw_api_error(scope, "querySelectorAll", "Element", &selector);
            return;
        }
    };
    match super::node_list::create(scope, matches) {
        Ok(list) => result.set(list.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
