pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "computedStyleMap", 0, computed_style_map)
}

fn computed_style_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    match super::style_property_map::create(scope) {
        Ok(map) => result.set(map.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
