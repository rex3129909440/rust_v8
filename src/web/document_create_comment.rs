pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createComment", 1, create_comment)
}

fn create_comment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let data = crate::webidl::value_to_string(scope, arguments.get(0));
    match super::comment::create(scope, data) {
        Ok(comment) => {
            super::node::set_owner_document(scope, comment, arguments.this());
            result.set(comment.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
