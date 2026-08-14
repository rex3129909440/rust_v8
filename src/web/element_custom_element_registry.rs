pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "customElementRegistry",
        get_custom_element_registry,
    )
}

fn get_custom_element_registry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(document) = super::node::owner_document(scope, arguments.this())
        && let Some(registry) =
            super::custom_element_registry::registry_for_document(scope, document)
    {
        result.set(registry.into());
    } else {
        result.set(v8::null(scope).into());
    }
}
