pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setAttributeNS", 3, call)
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
    let qualified_name = crate::webidl::value_to_string(scope, arguments.get(1));
    if let Err((name, message)) =
        super::document::validate_qualified_name(namespace.as_deref(), &qualified_name, true)
    {
        super::node::throw_dom_exception(scope, name, message);
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(2));
    super::element::set_attribute_full(scope, arguments.this(), qualified_name, value, namespace);
}
