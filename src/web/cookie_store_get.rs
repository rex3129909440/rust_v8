use super::cookie_store::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "get", 0, get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "CookieStore", "get", result);
        return;
    }
    let name = requested_name(scope, arguments.get(0));
    let found = super::document_cookie::global_snapshot(scope)
        .into_iter()
        .find(|entry| name.as_ref().is_none_or(|name| entry.name == *name))
        .map(|entry| entry_from_cookie(&entry));
    let value = found
        .as_ref()
        .map(|entry| cookie_object(scope, entry).into())
        .unwrap_or_else(|| v8::null(scope).into());
    resolved(scope, value, result);
}
