pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "requestStorageAccessFor",
        1,
        request_storage_access_for,
    )
}

fn request_storage_access_for(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let origin = crate::webidl::value_to_string(scope, arguments.get(0));
    let trustworthy = ::url::Url::parse(&origin)
        .ok()
        .is_some_and(|url| url.scheme() == "https");
    if !trustworthy {
        let Some(message) = v8::String::new(scope, "The requested origin is not trustworthy")
        else {
            return;
        };
        let exception = v8::Exception::type_error(scope, message);
        match super::writable_stream::rejected_promise(scope, exception) {
            Ok(promise) => result.set(promise.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    let undefined = v8::undefined(scope);
    match super::document_method_support::resolved(scope, undefined.into()) {
        Ok(promise) => result.set(promise.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
