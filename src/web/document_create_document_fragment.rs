pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createDocumentFragment",
        0,
        create_document_fragment,
    )
}

fn create_document_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::document_fragment::create(scope) {
        Ok(fragment) => {
            super::node::set_owner_document(scope, fragment, arguments.this());
            result.set(fragment.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
