pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "remove", 0, remove)
}
fn remove(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mut tokens = Vec::new();
    for index in 0..arguments.length() {
        let Some(token) = super::dom_token_list::validate_token(scope, arguments.get(index)) else {
            return;
        };
        tokens.push(token)
    }
    if !super::dom_token_list::update(scope, arguments.this(), |values| {
        values.retain(|value| !tokens.contains(value))
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else {
        super::dom_token_list::commit_binding(scope, arguments.this());
    }
}
