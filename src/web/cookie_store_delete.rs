use super::cookie_store::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let name = requested_name(scope, arguments.get(0)).unwrap_or_default();
    let domain = member(scope, options, "domain")
        .filter(|value| !value.is_null() && !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value));
    let path = member(scope, options, "path")
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value));
    let removed = super::document_cookie::delete_from_cookie_store(
        scope,
        &name,
        domain.as_deref(),
        path.as_deref(),
    )
    .map(|cookie| entry_from_cookie(&cookie));
    notify(scope, arguments.this(), None, removed);
    resolved(scope, v8::undefined(scope).into(), result);
}
