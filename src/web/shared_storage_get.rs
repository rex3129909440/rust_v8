pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "get", 1, get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::shared_storage::has_record(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "SharedStorage", "get", result);
        return;
    }
    if arguments.length() < 1 {
        let message =
            "Failed to execute 'get' on 'SharedStorage': 1 argument required, but only 0 present.";
        if let Some(promise) = crate::webidl::rejected_type_error_promise(scope, message) {
            result.set(promise.into());
        }
        return;
    }
    // Chromium 140 exposes get() on Window.sharedStorage, but a normal
    // top-level document is not a fenced-frame worklet context.
    let exception = super::dom_exception::create(
        scope,
        "Cannot call get() outside of a fenced frame.".to_owned(),
        "OperationError".to_owned(),
    )
    .map(Into::into)
    .unwrap_or_else(|_| v8::undefined(scope).into());
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
        result.set(promise.into());
    }
}
