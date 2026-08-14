pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "isDefaultNamespace", 1, call)
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
    let namespace = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, arguments.get(0));
        (!value.is_empty()).then_some(value)
    };
    let actual = super::node::locate_namespace_uri(scope, arguments.this(), None);
    result.set(v8::Boolean::new(scope, actual == namespace).into());
}
