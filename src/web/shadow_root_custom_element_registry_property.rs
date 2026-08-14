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
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::shadow_root::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(registry) = v.registry {
        r.set(v8::Local::new(scope, &registry).into())
    } else if !v.registry_is_null
        && let Some(document) = super::node::owner_document(scope, a.this())
        && let Some(registry) =
            super::custom_element_registry::registry_for_document(scope, document)
    {
        r.set(registry.into())
    } else {
        r.set(v8::null(scope).into())
    }
}
