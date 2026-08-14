pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "hasUnpartitionedCookieAccess",
        0,
        has_unpartitioned_cookie_access,
    )
}

fn has_unpartitioned_cookie_access(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "Document",
            "hasUnpartitionedCookieAccess",
            result,
        );
        return;
    }
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let allowed = v8::Boolean::new(scope, true);
    match super::document_method_support::resolved(scope, allowed.into()) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
