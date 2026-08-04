pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getAttributeNS", 2, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let namespace = if arguments.get(0).is_null() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, arguments.get(0));
        (!value.is_empty()).then_some(value)
    };
    let local_name = crate::webidl::value_to_string(scope, arguments.get(1));
    let value =
        super::element::attributes_snapshot(scope, arguments.this()).and_then(|attributes| {
            attributes.into_iter().find(|attribute| {
                attribute.name.rsplit(':').next().unwrap_or(&attribute.name) == local_name
                    && attribute.namespace_uri == namespace
            })
        });
    match value {
        Some(attribute) => {
            if let Some(value) = v8::String::new(scope, &attribute.value) {
                result.set(value.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}
