pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createAttributeNS",
        2,
        create_attribute_ns,
    )
}

fn create_attribute_ns(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let namespace = if arguments.get(0).is_null() {
        None
    } else {
        let namespace = crate::webidl::value_to_string(scope, arguments.get(0));
        (!namespace.is_empty()).then_some(namespace)
    };
    let qualified_name = crate::webidl::value_to_string(scope, arguments.get(1));
    if let Err((name, message)) =
        super::document::validate_qualified_name(namespace.as_deref(), &qualified_name, true)
    {
        super::node::throw_dom_exception(scope, name, message);
        return;
    }
    match super::attr::create(scope, qualified_name, String::new(), namespace, None) {
        Ok(attribute) => {
            super::node::set_owner_document(scope, attribute, arguments.this());
            result.set(attribute.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
