pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "toggle", 1, toggle)
}
fn toggle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'toggle' on 'DOMTokenList': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(token) = super::dom_token_list::validate_token(scope, arguments.get(0), "toggle")
    else {
        return;
    };
    let force = if arguments.length() > 1 {
        Some(arguments.get(1).boolean_value(scope))
    } else {
        None
    };
    let mut present = false;
    if !super::dom_token_list::update(scope, arguments.this(), |values| {
        let old = values.contains(&token);
        let desired = force.unwrap_or(!old);
        if desired && !old {
            values.push(token.clone())
        } else if !desired && old {
            values.retain(|v| v != &token)
        }
        present = desired
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::dom_token_list::commit_binding(scope, arguments.this());
    result.set(v8::Boolean::new(scope, present).into())
}
