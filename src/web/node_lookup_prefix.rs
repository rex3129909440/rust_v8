pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "lookupPrefix", 1, call)
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
    if arguments.get(0).is_null() {
        result.set(v8::null(scope).into());
        return;
    }
    let namespace = crate::webidl::value_to_string(scope, arguments.get(0));
    if namespace.is_empty() {
        result.set(v8::null(scope).into());
        return;
    }
    match super::node::locate_prefix(scope, arguments.this(), &namespace) {
        Some(prefix) => {
            if let Some(prefix) = v8::String::new(scope, &prefix) {
                result.set(prefix.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}
