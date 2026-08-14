pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "lookupNamespaceURI", 1, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::node::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let prefix = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, arguments.get(0)))
    };
    match super::node::locate_namespace_uri(scope, arguments.this(), prefix.as_deref()) {
        Some(namespace) => {
            if let Some(namespace) = v8::String::new(scope, &namespace) {
                result.set(namespace.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}
