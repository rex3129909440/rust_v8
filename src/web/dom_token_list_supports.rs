pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "supports", 1, supports)
}
fn supports(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(token) = super::dom_token_list::validate_token(scope, arguments.get(0)) else {
        return;
    };
    let supported = matches!(
        token.as_str(),
        "alternate"
            | "author"
            | "bookmark"
            | "external"
            | "help"
            | "license"
            | "next"
            | "nofollow"
            | "noopener"
            | "noreferrer"
            | "opener"
            | "prev"
            | "search"
            | "tag"
    );
    result.set(v8::Boolean::new(scope, supported).into())
}
