pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "hasAttributeNS", 2, has_attribute_ns)
}

fn has_attribute_ns(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(attributes) = super::element::attributes_snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let namespace = if arguments.get(0).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, arguments.get(0)))
    };
    let local_name = crate::webidl::value_to_string(scope, arguments.get(1));
    let present = attributes.into_iter().any(|attribute| {
        attribute.name.rsplit(':').next().unwrap_or(&attribute.name) == local_name
            && attribute.namespace_uri == namespace
    });
    result.set(v8::Boolean::new(scope, present).into());
}
