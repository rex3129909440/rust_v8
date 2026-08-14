pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "browsingTopics", 0, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "Document",
            "browsingTopics",
            result,
        );
        return;
    }
    let topics = v8::Array::new(scope, 0);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, topics.into()) {
        result.set(promise.into());
    }
}
