pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "namespaceURI", get_namespace_uri)
}

fn get_namespace_uri(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::attr::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.namespace_uri {
        Some(namespace) => {
            if let Some(value) = v8::String::new(scope, &namespace) {
                result.set(value.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}
