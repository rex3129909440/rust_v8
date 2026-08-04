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
    if let Some(value) =
        super::element::cached_reflected_value(scope, arguments.this(), "customElementRegistry")
    {
        result.set(v8::Local::new(scope, &value));
        return;
    }
    match super::custom_element_registry::create(scope) {
        Ok(registry) => {
            super::element::set_reflected_value(
                scope,
                arguments.this(),
                "customElementRegistry",
                Some(registry.into()),
            );
            result.set(registry.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
