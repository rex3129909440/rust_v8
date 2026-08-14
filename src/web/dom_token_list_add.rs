pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "add", 0, add)
}
fn add(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let mut tokens = Vec::new();
    for index in 0..arguments.length() {
        let Some(token) = super::dom_token_list::validate_token(scope, arguments.get(index), "add")
        else {
            return;
        };
        tokens.push(token)
    }
    if !super::dom_token_list::update(scope, arguments.this(), |values| {
        for token in tokens {
            if !values.contains(&token) {
                values.push(token)
            }
        }
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else {
        super::dom_token_list::commit_binding(scope, arguments.this());
    }
}
