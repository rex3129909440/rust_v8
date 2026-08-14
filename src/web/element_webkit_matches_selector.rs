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
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'webkitMatchesSelector' on 'Element': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(selector) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'webkitMatchesSelector' on 'Element'",
    ) else {
        return;
    };
    match super::dom_selector::matches_selector(
        scope,
        arguments.this(),
        &selector,
        arguments.this(),
    ) {
        Ok(value) => result.set(v8::Boolean::new(scope, value).into()),
        Err(_) => super::dom_selector::throw_api_error(
            scope,
            "webkitMatchesSelector",
            "Element",
            &selector,
        ),
    }
}
