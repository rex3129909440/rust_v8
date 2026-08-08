pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "removeAttributeNS", 2, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
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
    super::element::remove_attribute_full(
        scope,
        arguments.this(),
        namespace.as_deref(),
        &local_name,
    );
}
