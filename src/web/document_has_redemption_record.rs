pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "hasRedemptionRecord", 1, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let issuer = crate::webidl::value_to_string(scope, arguments.get(0));
    let trustworthy = ::url::Url::parse(&issuer).ok().is_some_and(|url| {
        url.scheme() == "https"
            || (url.scheme() == "http"
                && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1")))
    });
    if !trustworthy {
        let message = "Failed to execute 'hasRedemptionRecord' on 'Document': hasRedemptionRecord: Private Token issuer origins must be both HTTP(S) and secure (\"potentially trustworthy\").";
        let Some(message) = v8::String::new(scope, message) else {
            return;
        };
        let exception = v8::Exception::type_error(scope, message);
        if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
            result.set(promise.into());
        }
        return;
    }
    let absent = v8::Boolean::new(scope, false);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, absent.into()) {
        result.set(promise.into());
    }
}
